using MessagePack;
using MessagePack.Formatters;
using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;

namespace CriticalPoint {
    public struct TmplID {
        private ulong _id;

        public static readonly TmplID INVALID = new TmplID { _id = ulong.MaxValue };

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<ulong> tmpl_id_create([MarshalAs(UnmanagedType.LPStr)] string str);

        // public TmplID() => this._id = ulong.MaxValue;

        internal TmplID(ulong id) => this._id = id;

        public TmplID(string str) => this._id = tmpl_id_create(str).Unwrap();

        public static TmplID FromNullableString(string? str) {
            if (str != null) {
                return new TmplID(str);
            } else {
                return INVALID;
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        [return: MarshalAs(UnmanagedType.U1)]
        private static extern unsafe bool tmpl_id_is_valid(ulong cid);

        public bool IsValid { get => tmpl_id_is_valid(this._id); }

        public bool IsInvalid { get => !tmpl_id_is_valid(this._id); }

        public static bool operator ==(TmplID lhs, TmplID rhs) => lhs._id == rhs._id;
        public static bool operator !=(TmplID lhs, TmplID rhs) => lhs._id != rhs._id;

        public override bool Equals(object? obj) => obj is TmplID id && _id == id._id;

        public static explicit operator ulong(TmplID id) => id._id;

        public override int GetHashCode() => _id.GetHashCode();

        public override string ToString() => _id.ToString();

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<IntPtr> tmpl_id_to_string(ulong cid);

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe void tmpl_id_free_string(IntPtr cstr);

        public string TryRead() {
            StringBuilder sb = new StringBuilder(256);
            IntPtr cptr = tmpl_id_to_string(_id).Unwrap();
            string str = Marshal.PtrToStringAnsi(cptr) ?? "";
            tmpl_id_free_string(cptr);
            return str;
        }
    }

    public class TmplIDFormatter : IMessagePackFormatter<TmplID> {
        public void Serialize(ref MessagePackWriter writer, TmplID id, MessagePackSerializerOptions options) {
            writer.Write(id.TryRead());
        }

        public TmplID Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            return TmplID.FromNullableString(reader.ReadString());
        }
    }

    public class TmplIDLevelFormatter : IMessagePackFormatter<TmplIDLevel> {
        public void Serialize(ref MessagePackWriter writer, TmplIDLevel idLevel, MessagePackSerializerOptions options) {
            writer.WriteArrayHeader(2);
            writer.Write(idLevel.id.TryRead());
            writer.Write(idLevel.level);
        }

        public TmplIDLevel Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            if (reader.TryReadNil()) {
                return new TmplIDLevel { id = TmplID.INVALID, level = 0 };
            }
            int count = reader.ReadArrayHeader();
            if (count != 2) {
                throw new MessagePackSerializationException("Invalid Vec2 format");
            }
            return new TmplIDLevel {
                id = TmplID.FromNullableString(reader.ReadString()),
                level = reader.ReadUInt32(),
            };
        }
    }

    public class TmplIDPlusFormatter : IMessagePackFormatter<TmplIDPlus> {
        public void Serialize(ref MessagePackWriter writer, TmplIDPlus idPlus, MessagePackSerializerOptions options) {
            writer.WriteArrayHeader(2);
            writer.Write(idPlus.id.TryRead());
            writer.Write(idPlus.plus);
        }

        public TmplIDPlus Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            if (reader.TryReadNil()) {
                return new TmplIDPlus { id = TmplID.INVALID, plus = 0 };
            }
            int count = reader.ReadArrayHeader();
            if (count != 2) {
                throw new MessagePackSerializationException("Invalid Vec2 format");
            }
            return new TmplIDPlus {
                id = TmplID.FromNullableString(reader.ReadString()),
                plus = reader.ReadUInt32(),
            };
        }
    }

    //
    // Rust Symbol wrapper
    //

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct SymbolNode {
        private SymbolNode* _next;
        private uint _hash;
        internal ushort length;
        internal fixed byte chars[1];
    };

    public struct Symbol {
        private nint _ptr;

        public bool IsNull { get => _ptr == 0; }

        public static bool operator ==(Symbol lhs, Symbol rhs) => lhs._ptr == rhs._ptr;
        public static bool operator !=(Symbol lhs, Symbol rhs) => lhs._ptr != rhs._ptr;

        public override bool Equals(object? obj) => obj is Symbol s && _ptr == s._ptr;

        public static explicit operator nint(Symbol s) => s._ptr;

        public override int GetHashCode() => _ptr.GetHashCode();

        public override string ToString() => _ptr.ToString();

        public string TryRead() {
            if (_ptr == 0) {
                return "";
            }
            unsafe {
                var inner = (SymbolNode*)_ptr;
                return Marshal.PtrToStringUTF8((IntPtr)inner->chars, inner->length) ?? "";
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe nint symbol_create([MarshalAs(UnmanagedType.LPStr)] string str);

        internal Symbol(string str) => this._ptr = symbol_create(str);
    }

    partial struct CustomEvent {
        public string AsEventString() {
            return $"{this.source.TryRead()}/{this.name.TryRead()}";
        }
    }

    internal struct Return<T> where T : unmanaged {
        internal T Value;
        internal IntPtr _errMsg;

        internal bool IsOk => _errMsg == IntPtr.Zero;

        internal bool IsError => _errMsg != IntPtr.Zero;

        internal string ErrMsg => Marshal.PtrToStringUTF8(_errMsg) ?? "Unknown error";

        internal T Unwrap() {
            if (_errMsg != IntPtr.Zero) {
                throw new EngineException(ErrMsg);
            }
            return Value;
        }
    }

    internal struct Return0 {
        internal IntPtr _errMsg;

        internal bool IsOk => _errMsg == IntPtr.Zero;

        internal bool IsError => _errMsg != IntPtr.Zero;

        internal string ErrMsg => Marshal.PtrToStringUTF8(_errMsg) ?? "Unknown error";

        internal void Unwrap() {
            if (_errMsg != IntPtr.Zero) {
                throw new EngineException(ErrMsg);
            }
        }
    }

    public class EngineException : Exception {
        public EngineException(string message) : base(message) { }

        public EngineException(string message, Exception innerException) : base(message, innerException) { }
    }

    //
    // Rust smart pointers wrapper
    //

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct RsBoxDynAny {
        internal IntPtr ptr;
        internal IntPtr meta;

        internal bool IsNull => ptr == IntPtr.Zero;

        internal void Clear() {
            ptr = IntPtr.Zero;
            meta = IntPtr.Zero;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct RsBoxDyn<T> where T : unmanaged {
        internal T* ptr;
        private IntPtr _meta;

        internal bool IsNull => ptr == null;

        internal RsBoxDynAny ToAny() {
            return new RsBoxDynAny {
                ptr = (IntPtr)ptr,
                meta = _meta,
            };
        }

        internal void Clear() {
            ptr = null;
            _meta = IntPtr.Zero;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    internal struct RsArcInner<T> where T : unmanaged {
        internal volatile IntPtr strong;
        internal volatile IntPtr weak;
        internal T data;
    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct RsArcDynAny {
        internal IntPtr ptr;
        internal IntPtr meta;

        internal bool IsNull => ptr == IntPtr.Zero;

        internal void Clear() {
            ptr = IntPtr.Zero;
            meta = IntPtr.Zero;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct RsArcDyn<T> where T : unmanaged {
        internal RsArcInner<T>* ptr;
        private IntPtr _meta;

        internal bool IsNull => ptr == null;

        internal RsArcDynAny ToAny() {
            return new RsArcDynAny {
                ptr = (IntPtr)ptr,
                meta = _meta,
            };
        }

        internal void Clear() {
            ptr = null;
            _meta = IntPtr.Zero;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct RsBox<T> where T : unmanaged {
        internal T* ptr;

        internal bool IsNull => ptr == null;

        internal void Clear() {
            ptr = null;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct RsArc<T> where T : unmanaged {
        internal RsArcInner<T>* ptr;

        internal bool IsNull => ptr == null;

        internal void Clear() {
            ptr = null;
        }
    }

    //
    // Rust [T; N] wrapper
    //

    [StructLayout(LayoutKind.Sequential)]
    public unsafe ref struct RefArrayVal<T> where T : unmanaged {
#if UNITY_EDITOR || UNITY_STANDALONE
        private T* _ref;
        private int _len;

        internal RefArrayVal(T* ptr, int len) {
            _ref = ptr;
            _len = len;
        }

        internal RefArrayVal(ref T re, int len) {
            fixed (T* ptr = &re) {
                _ref = ptr;
            }
            _len = len;
        }

        public int Length { get => _len; }
        public bool IsEmpty { get => _len == 0; }

        public ref T this[int index] {
            get {
                if ((uint)index >= (uint)_len) {
                    string msg = string.Format("index:{0} len:{1}", index, _len);
                    throw new IndexOutOfRangeException(msg);
                }
                return ref *(_ref + index);
            }
        }
#else
        private ref T _ref;
        private int _len;

        internal RefArrayVal(T* ptr, int len) {
            _ref = ref *ptr;
            _len = len;
        }

        internal RefArrayVal(ref T re, int len) {
            _ref = ref re;
            _len = len;
        }

        public int Length { get => _len; }
        public bool IsEmpty { get => _len == 0; }

        public ref T this[int index] {
            get {
                if ((uint)index >= (uint)_len) {
                    string msg = string.Format("index:{0} len:{1}", index, _len);
                    throw new IndexOutOfRangeException(msg);
                }
                return ref Unsafe.Add(ref _ref, (nint)(uint)index);
            }
        }
#endif

        public Enumerator GetEnumerator() => new Enumerator(this);

        public ref struct Enumerator {
            private RefArrayVal<T> _array;
            private int _index;

            public Enumerator(RefArrayVal<T> array) {
                _array = array;
                _index = -1;
            }

            public T Current { get => _array[_index]; }

            public bool MoveNext() => ++_index < _array._len;
        }
    }

    //
    // Rust &[T] wrapper
    //

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct RsSlice<T> where T : unmanaged {
        internal T* ptr;
        internal UIntPtr len;

        internal RsSlice(T* ptr, UIntPtr len) {
            this.ptr = ptr;
            this.len = len;
        }

        internal RsSlice(RsVec<T> vec) {
            ptr = vec.ptr;
            len = vec.len;
        }

        internal bool IsNull => ptr == null;

        internal void Clear() {
            ptr = null;
            len = UIntPtr.Zero;
        }
    }

    public ref struct RefSliceVal<T> where T : unmanaged {
        private RsSlice<T> _slice;

        internal RefSliceVal(RsSlice<T> slice) => _slice = slice;

        public int Length { get => (int)_slice.len; }
        public bool IsEmpty { get => _slice.len == UIntPtr.Zero; }

        public readonly T this[int index] {
            get {
                if ((uint)index >= (uint)_slice.len) {
                    string msg = string.Format("index:{0} len:{1}", index, _slice.len);
                    throw new IndexOutOfRangeException(msg);
                }
                unsafe {
                    return Unsafe.Add(ref *_slice.ptr, (nint)(uint)index);
                }
            }
        }

        public Enumerator GetEnumerator() => new Enumerator(this);

        public ref struct Enumerator {
            private RefSliceVal<T> _slice;
            private int _index;

            public Enumerator(RefSliceVal<T> slice) {
                _slice = slice;
                _index = -1;
            }

            public T Current { get => _slice[_index]; }

            public bool MoveNext() => ++_index < _slice.Length;
        }
    }

    //
    // Rust ArrayVec helper
    //

    [StructLayout(LayoutKind.Explicit)]
    internal unsafe struct ArrayVecLen<T> {
        [FieldOffset(0)]
        internal readonly ushort len;
        [FieldOffset(0)]
        private readonly T _padding;
    }

    //
    // Rust String wrapper
    //

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct RsString {
        internal UIntPtr len;
        internal IntPtr ptr;
        internal UIntPtr cap;

        internal bool IsNull => ptr == IntPtr.Zero;

        internal void Clear() {
            len = UIntPtr.Zero;
            ptr = IntPtr.Zero;
            cap = UIntPtr.Zero;
        }
    }

    public ref struct RefRsString {
        private RsString _str;

        internal RefRsString(RsString str) => _str = str;

        public int Length { get => (int)_str.len; }
        public bool IsEmpty { get => _str.len == UIntPtr.Zero; }

        public override string ToString() {
            if (_str.ptr == IntPtr.Zero) {
                return "";
            }
            return Marshal.PtrToStringUTF8(_str.ptr, (int)_str.len) ?? "";
        }
    }

    //
    // Rust Vec<T> wrapper
    //

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct RsVec<T> where T : unmanaged {
        internal UIntPtr cap;
        internal T* ptr;
        internal UIntPtr len;

        internal bool IsNull => ptr == null;

        internal void Clear() {
            cap = UIntPtr.Zero;
            ptr = null;
            len = UIntPtr.Zero;
        }
    }

    public ref struct RefVecVal<T> where T : unmanaged {
        private RsVec<T> _vec;

        internal RefVecVal(RsVec<T> vec) => _vec = vec;

        public int Length { get => (int)_vec.len; }
        public bool IsEmpty { get => _vec.len == UIntPtr.Zero; }

        public readonly T this[int index] {
            get {
                if ((uint)index >= (uint)_vec.len) {
                    string msg = string.Format("index:{0} len:{1}", index, _vec.len);
                    throw new IndexOutOfRangeException(msg);
                }
                unsafe {
                    return Unsafe.Add(ref *_vec.ptr, (nint)(uint)index);
                }
            }
        }

        internal RsSlice<T> AsSlice() {
            unsafe { return new RsSlice<T>(_vec.ptr, _vec.len); }
        }

        public Enumerator GetEnumerator() => new Enumerator(this);

        public ref struct Enumerator {
            private RefVecVal<T> _vec;
            private int _index;

            public Enumerator(RefVecVal<T> vec) {
                _vec = vec;
                _index = -1;
            }

            public T Current { get => _vec[_index]; }

            public bool MoveNext() => ++_index < _vec.Length;
        }
    }

    public ref struct RefVecRsString {
        private RsVec<RsString> _vec;

        internal RefVecRsString(RsVec<RsString> vec) => _vec = vec;

        public int Length { get => (int)_vec.len; }
        public bool IsEmpty { get => _vec.len == UIntPtr.Zero; }

        public RefRsString this[int index] {
            get {
                if ((uint)index >= (uint)_vec.len) {
                    string msg = string.Format("index:{0} len:{1}", index, _vec.len);
                    throw new IndexOutOfRangeException(msg);
                }
                unsafe {
                    return new RefRsString(Unsafe.Add(ref *_vec.ptr, (nint)(uint)index));
                }
            }
        }

        internal RsSlice<RsString> AsSlice() => new RsSlice<RsString>(_vec);

        public Enumerator GetEnumerator() => new Enumerator(this);

        public ref struct Enumerator {
            private RefVecRsString _vec;
            private int _index;

            public Enumerator(RefVecRsString vec) {
                _vec = vec;
                _index = -1;
            }

            public RefRsString Current { get => _vec[_index]; }

            public bool MoveNext() => ++_index < _vec.Length;
        }
    }

    public ref struct RefVecBoxStateAny {
        private RsVec<RsBoxDynStateAny> _vec;

        internal RefVecBoxStateAny(RsVec<RsBoxDynStateAny> vec) => _vec = vec;

        public int Length { get => (int)_vec.len; }
        public bool IsEmpty { get => _vec.len == UIntPtr.Zero; }

        public RefDynStateAny this[int index] {
            get {
                if ((uint)index >= (uint)_vec.len) {
                    string msg = string.Format("index:{0} len:{1}", index, _vec.len);
                    throw new IndexOutOfRangeException(msg);
                }
                unsafe {
                    return Unsafe.Add(ref *_vec.ptr, (nint)(uint)index).MakeRef();
                }
            }
        }

        internal RsSlice<RsBoxDynStateAny> AsSlice() => new RsSlice<RsBoxDynStateAny>(_vec);

        public Enumerator GetEnumerator() => new Enumerator(this);

        public ref struct Enumerator {
            private RefVecBoxStateAny _vec;
            private int _index;

            public Enumerator(RefVecBoxStateAny vec) {
                _vec = vec;
                _index = -1;
            }

            public RefDynStateAny Current { get => _vec[_index]; }

            public bool MoveNext() => ++_index < _vec.Length;
        }
    }

    public ref struct RefVecArcStateAny {
        private RsVec<RsArcDynStateAny> _vec;

        internal RefVecArcStateAny(RsVec<RsArcDynStateAny> vec) => _vec = vec;

        public int Length { get => (int)_vec.len; }
        public bool IsEmpty { get => _vec.len == UIntPtr.Zero; }

        public WeakDynStateAny this[int index] {
            get {
                if ((uint)index >= (uint)_vec.len) {
                    string msg = string.Format("index:{0} len:{1}", index, _vec.len);
                    throw new IndexOutOfRangeException(msg);
                }
                unsafe {
                    return Unsafe.Add(ref *_vec.ptr, (nint)(uint)index).MakeWeak();
                }
            }
        }

        internal RsSlice<RsArcDynStateAny> AsSlice() => new RsSlice<RsArcDynStateAny>(_vec);

        public Enumerator GetEnumerator() => new Enumerator(this);

        public ref struct Enumerator {
            private RefVecArcStateAny _vec;
            private int _index;

            public Enumerator(RefVecArcStateAny vec) {
                _vec = vec;
                _index = -1;
            }

            public WeakDynStateAny Current { get => _vec[_index]; }

            public bool MoveNext() => ++_index < _vec.Length;
        }
    }

    public ref struct RefVecBoxStateActionAny {
        private RsVec<RsBoxDynStateActionAny> _vec;

        internal RefVecBoxStateActionAny(RsVec<RsBoxDynStateActionAny> vec) => _vec = vec;

        public int Length { get => (int)_vec.len; }
        public bool IsEmpty { get => _vec.len == UIntPtr.Zero; }

        public RefDynStateActionAny this[int index] {
            get {
                if ((uint)index >= (uint)_vec.len) {
                    string msg = string.Format("index:{0} len:{1}", index, _vec.len);
                    throw new IndexOutOfRangeException(msg);
                }
                unsafe {
                    return Unsafe.Add(ref *_vec.ptr, (nint)(uint)index).MakeRef();
                }
            }
        }

        internal RsSlice<RsBoxDynStateActionAny> AsSlice() => new RsSlice<RsBoxDynStateActionAny>(_vec);

        public Enumerator GetEnumerator() => new Enumerator(this);

        public ref struct Enumerator {
            private RefVecBoxStateActionAny _vec;
            private int _index;

            public Enumerator(RefVecBoxStateActionAny vec) {
                _vec = vec;
                _index = -1;
            }

            public RefDynStateActionAny Current { get => _vec[_index]; }

            public bool MoveNext() => ++_index < _vec.Length;
        }
    }
}

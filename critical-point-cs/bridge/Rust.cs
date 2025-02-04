using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace CriticalPoint {

    public struct Symbol {
        private nint _n;

        public bool IsNull { get => _n == 0; }

        public static bool operator ==(Symbol lhs, Symbol rhs) => lhs._n == rhs._n;
        public static bool operator !=(Symbol lhs, Symbol rhs) => lhs._n != rhs._n;

        public override bool Equals(object? obj) => obj is Symbol s && _n == s._n;

        public static explicit operator nint(Symbol s) => s._n;

        public override int GetHashCode() => _n.GetHashCode();

        public override string ToString() => _n.ToString();

        public string TryRead() {
            if (_n == 0) {
                return "";
            }
            unsafe {
                var inner = (SymbolInner*)_n;
                return Marshal.PtrToStringUTF8((IntPtr)inner->chars, inner->length) ?? "";
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Symbol new_symbol([MarshalAs(UnmanagedType.LPStr)] string str);

        internal Symbol(string str) => this = new_symbol(str);
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
    // Rust Symbol wrapper
    //

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe struct SymbolInner {
        private SymbolInner* _next;
        private ulong _hash;
        private uint _ref_count;
        internal ushort length;
        internal fixed byte chars[1];
    };

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

            public bool MoveNext() => ++_index < _index;
        }
    }
}

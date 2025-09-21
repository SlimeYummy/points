using MessagePack;
using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

#if GODOT
using Godot;
#elif UNITY_EDITOR || UNITY_STANDALONE
using UnityEngine;
#endif

namespace CriticalPoint {
    public class LogicEngine : IDisposable {
        /// frames per second
        public const float FPS = 30f;
        /// frames per second
        public const double FPS_D = 30;

        /// seconds per frame
        public const float SPF = 1.0f / FPS;
        /// seconds per frame
        public const double SPF_D = 1.0 / FPS_D;

        public const float CFG_FPS = 60f;
        public const double CFG_FPS_D = 60;

        public const float CFG_SPF = 1.0f / CFG_FPS;
        public const double CFG_SPF_D = 1.0f / CFG_FPS_D;

#if GODOT
#elif UNITY_EDITOR || UNITY_STANDALONE
        /// default camera direction
        public static readonly Vector2 DEFAULT_VIEW_DIR_2D = new Vector2(0, -1);
        /// default camera direction
        public static readonly Vector3 DEFAULT_VIEW_DIR_3D = Vector3.back;

        /// default character toward direction
        public static readonly Vector2 DEFAULT_TOWARD_DIR_2D = new Vector2(0, 1);
        /// default character toward direction
        public static readonly Vector3 DEFAULT_TOWARD_DIR_3D = Vector3.forward;
#endif
        
        public const uint LOG_OFF = 0;
        public const uint LOG_ERROR = 1;
        public const uint LOG_WARN = 2;
        public const uint LOG_INFO = 3;
        public const uint LOG_DEBUG = 4;
        public const uint LOG_TRACE = 5;

        private IntPtr _engine;

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 engine_initialize(
            [MarshalAs(UnmanagedType.LPStr)] string tmpl_path,
            [MarshalAs(UnmanagedType.LPStr)] string asset_path,
            [MarshalAs(UnmanagedType.LPStr)] string log_file,
            uint log_level
        );

        public static void Initialize(string tmpl_path, string asset_path, string log_file, uint log_level)
        {
            engine_initialize(tmpl_path, asset_path, log_file, log_level).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<IntPtr> engine_create();

        public LogicEngine() {
            _engine = engine_create().Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe void engine_destroy(IntPtr engine);

        public void Dispose() {
            if (_engine != IntPtr.Zero) {
                engine_destroy(_engine);
                _engine = IntPtr.Zero;
            }
        }

        ~LogicEngine() => Dispose();

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 engine_verify_player(
            IntPtr engine,
            byte* player_data,
            uint player_len
        );

        // Return "OK" success
        // Return error message if failed
        public string VerifyPlayer(ParamPlayer player) {
            byte[] bytes = MessagePackSerializer.Serialize(player, Static.MsgPackOpts);
            unsafe {
                fixed (byte* ptr = bytes) {
                    Return0 ret = engine_verify_player(_engine, ptr, (uint)bytes.Length);
                    if (ret.IsError) {
                        return ret.ErrMsg;
                    }
                    return "OK";
                }
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<LogicEngineStatus> engine_get_game_status(IntPtr engine);

        LogicEngineStatus GetGameStatus() {
            return engine_get_game_status(_engine).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsArcStateSet> engine_start_game(
            IntPtr engine,
            byte* zone_data,
            uint zone_len,
            byte* players_data,
            uint players_len
        );

        public ArcStateSet StartGame(ParamZone zone, List<ParamPlayer> players) {
            byte[] zone_bytes = MessagePackSerializer.Serialize(zone, Static.MsgPackOpts);
            byte[] players_bytes = MessagePackSerializer.Serialize(players, Static.MsgPackOpts);
            unsafe {
                fixed (byte* zone_ptr = zone_bytes, players_ptr = players_bytes) {
                    var raw = engine_start_game(_engine, zone_ptr, (uint)zone_bytes.Length, players_ptr, (uint)players_bytes.Length).Unwrap();
                    return raw.MakeArc();
                }
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsArcStateSet> engine_update_game(
            IntPtr engine,
            byte* events_data,
            uint events_len
        );

        public ArcStateSet UpdateGame(List<InputPlayerEvents> events) {
            byte[] events_bytes = MessagePackSerializer.Serialize(events, Static.MsgPackOpts);
            unsafe {
                fixed (byte* events_ptr = events_bytes) {
                    var raw = engine_update_game(_engine, events_ptr, (uint)events_bytes.Length).Unwrap();
                    return raw.MakeArc();
                }
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 engine_stop_game(IntPtr engine);

        public void StopGame() {
            engine_stop_game(_engine).Unwrap();
        }
    }

    public class VecArcStateSet : IDisposable {
        private RsVec<RsArcStateSet> _vec;

        internal VecArcStateSet(RsVec<RsArcStateSet> vec) => _vec = vec;

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe void vec_arc_state_set_drop(RsVec<RsArcStateSet> vec);

        public void Dispose() {
            if (!_vec.IsNull) {
                vec_arc_state_set_drop(_vec);
                _vec.Clear();
            }
        }

        ~VecArcStateSet() => Dispose();

        public int Length { get => (int)_vec.len; }
        public bool IsEmpty { get => _vec.len == UIntPtr.Zero; }

        public WeakStateSet this[int index] {
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

        public Enumerator GetEnumerator() => new Enumerator(this);

        public ref struct Enumerator {
            private VecArcStateSet _vec;
            private int _index;

            public Enumerator(VecArcStateSet vec) {
                _vec = vec;
                _index = -1;
            }

            public WeakStateSet Current { get => _vec[_index]; }

            public bool MoveNext() => ++_index < _vec.Length;
        }
    }
}

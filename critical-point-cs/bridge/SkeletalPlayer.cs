using MessagePack;
using System;
using System.Runtime.InteropServices;

namespace CriticalPoint {
    [MessagePackObject(keyAsPropertyName: true)]
    public struct SkeletalAnimation {
        public string animation;
        public string root_motion;
        public string weapon_motion;
        public float start_ratio;
        public float finish_ratio;
        public float fade_in_secs;
        public bool fade_out_update;

        public SkeletalAnimation(string animation, string root_motion, string weapon_motion, float start_ratio, float finish_ratio) {
            this.animation = animation;
            this.root_motion = root_motion;
            this.weapon_motion = weapon_motion;
            this.start_ratio = start_ratio;
            this.finish_ratio = finish_ratio;
            this.fade_in_secs = 0.0f;
            this.fade_out_update = false;
        }

        public SkeletalAnimation(string animation, float start_ratio, float finish_ratio) {
            this.animation = animation;
            this.root_motion = "";
            this.weapon_motion = "";
            this.start_ratio = start_ratio;
            this.finish_ratio = finish_ratio;
            this.fade_in_secs = 0.0f;
            this.fade_out_update = false;
        }

        public SkeletalAnimation(string animation) {
            this.animation = animation;
            this.root_motion = "";
            this.weapon_motion = "";
            this.start_ratio = 0.0f;
            this.finish_ratio = 1.0f;
            this.fade_in_secs = 0.0f;
            this.fade_out_update = false;
        }

        public SkeletalAnimation(string animation, string root_motion) {
            this.animation = animation;
            this.root_motion = root_motion;
            this.weapon_motion = "";
            this.start_ratio = 0.0f;
            this.finish_ratio = 1.0f;
            this.fade_in_secs = 0.0f;
            this.fade_out_update = false;
        }

        public SkeletalAnimation(string animation, string root_motion, string weapon_motion) {
            this.animation = animation;
            this.root_motion = root_motion;
            this.weapon_motion = weapon_motion;
            this.start_ratio = 0.0f;
            this.finish_ratio = 1.0f;
            this.fade_in_secs = 0.0f;
            this.fade_out_update = false;
        }

        public SkeletalAnimation SetFadeInSecs(float secs) {
            this.fade_in_secs = secs;
            return this;
        }

        public SkeletalAnimation SetFadeInFrames(uint frame) {
            this.fade_in_secs = frame / 60.0f;
            return this;
        }

        public SkeletalAnimation SetFadeOutUpdate(bool update) {
            this.fade_out_update = update;
            return this;
        }
    }

    public class SkeletalPlayer : IDisposable {
        private IntPtr _player = IntPtr.Zero;

        public bool IsNull { get => _player == IntPtr.Zero; }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<IntPtr> skeletal_player_create(
            [MarshalAs(UnmanagedType.LPStr)] string skeleton_path
        );

        public SkeletalPlayer(string skeleton_path) {
            _player = skeletal_player_create(skeleton_path).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe void skeletal_player_destroy(IntPtr playback);

        public void Dispose() {
            if (_player != IntPtr.Zero) {
                skeletal_player_destroy(_player);
                _player = IntPtr.Zero;
            }
        }

        ~SkeletalPlayer() => Dispose();

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsBox<RsSkeletonMeta>> skeletal_player_skeleton_meta(IntPtr playback);

        public BoxSkeletonMeta SkeletonMeta() {
            unsafe {
                return new BoxSkeletonMeta(skeletal_player_skeleton_meta(_player).Unwrap().ptr);
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_player_set_animations(
            IntPtr playback,
            byte* player_data,
            uint player_len,
            [MarshalAs(UnmanagedType.U1)] bool is_loop
        );

        public void SetAnimations(SkeletalAnimation[] animations, bool is_loop = false) {
            byte[] bytes = MessagePackSerializer.Serialize(animations, Static.MsgPackOpts);
            unsafe {
                fixed (byte* ptr = bytes) {
                    skeletal_player_set_animations(_player, ptr, (uint)bytes.Length, is_loop).Unwrap();
                }
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<float> skeletal_player_duration(IntPtr playback);

        public float Duration() {
            return skeletal_player_duration(_player).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<float> skeletal_player_progress(IntPtr playback);

        public float Progress() {
            return skeletal_player_progress(_player).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_player_set_progress(IntPtr playback, float progress);

        public void SetProgress(float progress) {
            skeletal_player_set_progress(_player, progress).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_player_add_progress(IntPtr playback, float delta);

        public void AddProgress(float delta) {
            skeletal_player_add_progress(_player, delta).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_player_update(IntPtr playback);

        public void Update() {
            skeletal_player_update(_player).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<Mat4>> skeletal_player_model_rest_poses(IntPtr playback);

        public RefSliceVal<Mat4> ModelRestPoses() {
            return new RefSliceVal<Mat4>(skeletal_player_model_rest_poses(_player).Unwrap());
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<Mat4>> skeletal_player_model_out(IntPtr playback);

        public RefSliceVal<Mat4> ModelOut() {
            return new RefSliceVal<Mat4>(skeletal_player_model_out(_player).Unwrap());
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<Mat4> skeletal_player_root_motion_out(IntPtr playback);

        public Mat4 RootMotionOut() {
            return skeletal_player_root_motion_out(_player).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<WeaponMotionIsometry>> skeletal_player_weapon_motions_out(IntPtr playback);

        public RefSliceVal<WeaponMotionIsometry> WeaponMotionsOut() {
            return new RefSliceVal<WeaponMotionIsometry>(skeletal_player_weapon_motions_out(_player).Unwrap());
        }
    }
}

using System;
using System.Runtime.InteropServices;

namespace CriticalPoint {

    //
    // SkeletalAnimator
    //

    public class SkeletalAnimator : IDisposable {
        private IntPtr _animator = IntPtr.Zero;

        public bool IsNull { get => _animator == IntPtr.Zero; }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<IntPtr> skeletal_animator_create(
            IntPtr resource,
            [MarshalAs(UnmanagedType.U1)] bool skip_l2m
        );

        public SkeletalAnimator(SkeletalResource resource, bool skip_l2m = false) {
            _animator = skeletal_animator_create(resource.InnerPtr, skip_l2m).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe void skeletal_animator_destroy(IntPtr engine);

        public void Dispose() {
            if (_animator != IntPtr.Zero) {
                skeletal_animator_destroy(_animator);
                _animator = IntPtr.Zero;
            }
        }

        ~SkeletalAnimator() => Dispose();

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsBox<RsSkeletonMeta>> skeletal_animator_skeleton_meta(IntPtr animator);

        public BoxSkeletonMeta SkeletonMeta() {
            unsafe {
                return new BoxSkeletonMeta(skeletal_animator_skeleton_meta(_animator).Unwrap().ptr);
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_animator_update(
            IntPtr animator,
            uint frame,
            RsSlice<RsBoxDynStateAction> states
        );

        public void Update(uint frame, RefVecBoxStateAction states) {
            skeletal_animator_update(_animator, frame, states.AsSlice()).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_animator_restore(
            IntPtr animator,
            uint frame,
            RsSlice<RsBoxDynStateAction> states
        );

        public void Restore(uint frame, RefVecBoxStateAction states) {
            skeletal_animator_restore(_animator, frame, states.AsSlice()).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_animator_discard(IntPtr animator, uint frame);

        public void Discard(uint frame) {
            skeletal_animator_discard(_animator, frame).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_animator_animate(IntPtr animator);

        public void Animate() {
            skeletal_animator_animate(_animator).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<SoaTransform>> skeletal_animator_joint_rest_poses(IntPtr animator);
        public RefSliceVal<SoaTransform> JointRestPoses() {
            return new RefSliceVal<SoaTransform>(skeletal_animator_joint_rest_poses(_animator).Unwrap());
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<SoaTransform>> skeletal_animator_local_out(IntPtr animator);

        public RefSliceVal<SoaTransform> LocalOut() {
            return new RefSliceVal<SoaTransform>(skeletal_animator_local_out(_animator).Unwrap());
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<Mat4>> skeletal_animator_model_out(IntPtr animator);

        public RefSliceVal<Mat4> ModelOut() {
            return new RefSliceVal<Mat4>(skeletal_animator_model_out(_animator).Unwrap());
        }
    }

    //
    // SkeletalResource
    //

    public class SkeletalResource : IDisposable {
        private IntPtr _resource = IntPtr.Zero;

        public bool IsNull { get => _resource == IntPtr.Zero; }

        internal IntPtr InnerPtr { get => _resource; }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<IntPtr> skeletal_resource_create(
            [MarshalAs(UnmanagedType.LPStr)] string skeleton_path
        );

        public SkeletalResource(string skeleton_path) {
            _resource = skeletal_resource_create(skeleton_path).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe void skeletal_resource_destroy(IntPtr resource);

        public void Dispose() {
            if (_resource != IntPtr.Zero) {
                skeletal_resource_destroy(_resource);
                _resource = IntPtr.Zero;
            }
        }

        ~SkeletalResource() => Dispose();

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_resource_add_animation(
            IntPtr resource,
            ASymbol logic_path,
            [MarshalAs(UnmanagedType.LPStr)] string view_path
        );

        public void AddAnimation(ASymbol logic_path, string view_path) {
            skeletal_resource_add_animation(_resource, logic_path, view_path).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_resource_remove_animation(
            IntPtr resource,
            ASymbol logic_path
        );

        public void RemoveAnimation(ASymbol logic_path) {
            skeletal_resource_remove_animation(_resource, logic_path).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<byte> skeletal_resource_has_animation(
            IntPtr resource,
            ASymbol logic_path
        );

        public bool HasAnimation(ASymbol logic_path) {
            return skeletal_resource_has_animation(_resource, logic_path).Unwrap() != 0;
        }
    }

    //
    // SkeletalPlayer
    //

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
        private static extern unsafe Return0 skeletal_player_set_animation(
            IntPtr playback,
            [MarshalAs(UnmanagedType.LPStr)] string animation_path,
            [MarshalAs(UnmanagedType.U1)] bool is_loop
        );

        public void SetAnimation(string animation_path, bool is_loop = false) {
            skeletal_player_set_animation(_player, animation_path, is_loop).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<float> skeletal_player_duration(IntPtr playback);

        public float Duration() {
            return skeletal_player_duration(_player).Unwrap();
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
        private static extern unsafe Return<RsSlice<Transform3A>> skeletal_player_local_rest_poses(IntPtr playback);

        public RefSliceVal<Transform3A> LocalRestPoses() {
            return new RefSliceVal<Transform3A>(skeletal_player_local_rest_poses(_player).Unwrap());
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<Mat4>> skeletal_player_model_rest_poses(IntPtr playback);

        public RefSliceVal<Mat4> ModelRestPoses() {
            return new RefSliceVal<Mat4>(skeletal_player_model_rest_poses(_player).Unwrap());
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<Transform3A>> skeletal_player_local_out(IntPtr playback);

        public RefSliceVal<Transform3A> LocalOut() {
            return new RefSliceVal<Transform3A>(skeletal_player_local_out(_player).Unwrap());
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<Mat4>> skeletal_player_model_out(IntPtr playback);

        public RefSliceVal<Mat4> ModelOut() {
            return new RefSliceVal<Mat4>(skeletal_player_model_out(_player).Unwrap());
        }
    }

    //
    // Utils
    //

    public ref struct RefVecRsSkeletonJointMeta {
        private RsVec<RsSkeletonJointMeta> _vec;

        internal RefVecRsSkeletonJointMeta(RsVec<RsSkeletonJointMeta> vec) => _vec = vec;

        public int Length { get => (int)_vec.len; }
        public bool IsEmpty { get => _vec.len == UIntPtr.Zero; }

        public RefSkeletonJointMeta this[int index] {
            get {
                if ((uint)index >= (uint)_vec.len) {
                    string msg = string.Format("index:{0} len:{1}", index, _vec.len);
                    throw new IndexOutOfRangeException(msg);
                }
                unsafe {
                    return new RefSkeletonJointMeta(&_vec.ptr[index]);
                }
            }
        }

        public Enumerator GetEnumerator() => new Enumerator(this);

        public ref struct Enumerator {
            private RefVecRsSkeletonJointMeta _vec;
            private int _index;

            public Enumerator(RefVecRsSkeletonJointMeta vec) {
                _vec = vec;
                _index = -1;
            }

            public RefSkeletonJointMeta Current { get => _vec[_index]; }

            public bool MoveNext() => ++_index < _vec.Length;
        }
    }
}

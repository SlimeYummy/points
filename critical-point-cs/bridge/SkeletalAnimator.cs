using System;
using System.Runtime.InteropServices;

namespace CriticalPoint {

    public class SkeletalAnimator : IDisposable {
        //
        // Resource
        //

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_resource_load_skeleton(Symbol skel);

        public static void LoadSkeleton(Symbol skel) {
            skeletal_resource_load_skeleton(skel).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_resource_load_animation(Symbol anim);

        public static void LoadAnimation(Symbol anim) {
            skeletal_resource_load_animation(anim).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_resource_load_weapon_tracks(Symbol wt);

        public static void LoadWeaponTracks(Symbol wt) {
            skeletal_resource_load_weapon_tracks(wt).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_resource_load(
            Symbol* skels,
            uint skel_len,
            Symbol* anims,
            uint anim_len,
            Symbol* wts,
            uint wt_len
        );

        public static void Load(Symbol[] skels, Symbol[] anims, Symbol[] wts) {
            unsafe {
                fixed (Symbol* skels_ptr = skels, anims_ptr = anims, wts_ptr = wts) {
                    skeletal_resource_load(
                        skels_ptr,
                        (uint)skels.Length,
                        anims_ptr,
                        (uint)anims.Length,
                        wts_ptr,
                        (uint)wts.Length
                    ).Unwrap();
                }
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe uint skeletal_resource_skeleton_count();

        public static uint SkeletonCount() => skeletal_resource_skeleton_count();

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe uint skeletal_resource_animation_count();

        public static uint AnimationCount() => skeletal_resource_animation_count();

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe uint skeletal_resource_weapon_tracks_count();

        public static uint WeaponTracksCount() => skeletal_resource_weapon_tracks_count();

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe void skeletal_resource_clear_unused();

        public static void ClearUnused() {
            skeletal_resource_clear_unused();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe void skeletal_resource_clear_all();

        public static void ClearAll() {
            skeletal_resource_clear_all();
        }

        //
        // Animator
        //

        private IntPtr _animator = IntPtr.Zero;

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<IntPtr> skeletal_animator_create(Symbol skel);

        public SkeletalAnimator(Symbol skel) {
            var ret = skeletal_animator_create(skel);
            var animator = ret.Unwrap();
            _animator = animator;
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe void skeletal_animator_destroy(IntPtr animator);

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
            RsSlice<RsBoxDynStateActionAny> states
        );

        public void Update(RefVecBoxStateActionAny states) {
            skeletal_animator_update(_animator, states.AsSlice()).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return0 skeletal_animator_animate(IntPtr animator, float ratio);

        public void Animate(float ratio) {
            skeletal_animator_animate(_animator, ratio).Unwrap();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<Mat4>> skeletal_animator_model_rest_poses(IntPtr animator);

        public RefSliceVal<Mat4> ModelRestPoses() {
            return new RefSliceVal<Mat4>(skeletal_animator_model_rest_poses(_animator).Unwrap());
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<Mat4>> skeletal_animator_model_poses(IntPtr animator);

        public RefSliceVal<Mat4> ModelPoses() {
            return new RefSliceVal<Mat4>(skeletal_animator_model_poses(_animator).Unwrap());
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsSlice<WeaponTransform>> skeletal_animator_weapon_transforms(IntPtr animator);

        public RefSliceVal<WeaponTransform> WeaponTransforms() {
            return new RefSliceVal<WeaponTransform>(skeletal_animator_weapon_transforms(_animator).Unwrap());
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

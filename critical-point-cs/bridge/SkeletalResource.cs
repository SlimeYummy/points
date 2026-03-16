using CriticalPoint;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;

namespace CriticalPoint {
    public static class SkeletalResource {
        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsBox<RsSkeletonMeta>> load_skeleton_meta(
            [MarshalAs(UnmanagedType.LPStr)] string str,
            [MarshalAs(UnmanagedType.U1)] bool withJoints
        );

        public static BoxSkeletonMeta LoadSkeletonMeta(string path, bool withJoints) {
            unsafe {
                return new BoxSkeletonMeta(load_skeleton_meta(path, withJoints).Unwrap().ptr);
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsBox<RsAnimationMeta>> load_animation_meta(
            [MarshalAs(UnmanagedType.LPStr)] string str
        );

        public static BoxAnimationMeta LoadAnimationMeta(string path) {
            unsafe {
                return new BoxAnimationMeta(load_animation_meta(path).Unwrap().ptr);
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsBox<RsRootMotionMeta>> load_root_motion_meta(
            [MarshalAs(UnmanagedType.LPStr)] string str
        );

        public static BoxRootMotionMeta LoadRootMotionMeta(string path) {
            unsafe {
                return new BoxRootMotionMeta(load_root_motion_meta(path).Unwrap().ptr);
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsBox<RsWeaponMotionMeta>> load_weapon_motion_meta(
            [MarshalAs(UnmanagedType.LPStr)] string str,
            [MarshalAs(UnmanagedType.U1)] bool withNames
        );

        public static BoxWeaponMotionMeta LoadWeaponMotionMeta(string path, bool withNames) {
            unsafe {
                return new BoxWeaponMotionMeta(load_weapon_motion_meta(path, withNames).Unwrap().ptr);
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe Return<RsBox<RsHitMotionMeta>> load_hit_motion_meta(
            [MarshalAs(UnmanagedType.LPStr)] string str,
            [MarshalAs(UnmanagedType.U1)] bool withNames
        );

        public static BoxHitMotionMeta LoadHitMotionMeta(string path, bool withNames) {
            unsafe {
                return new BoxHitMotionMeta(load_hit_motion_meta(path, withNames).Unwrap().ptr);
            }
        }
    }
}

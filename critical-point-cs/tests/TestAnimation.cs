//using bridge;
using CriticalPoint;
using System.Runtime.InteropServices;

namespace CriticalPointTests {
    [TestClass]
    public class TestResourceMeta {
        const string ASSET_PATH = "../../../../../test-tmp/test-asset/";

        [TestMethod]
        public void TestSkeletonMeta() {
            var meta = SkeletalResource.LoadSkeletonMeta(ASSET_PATH + "Girl.ls-ozz", true);
            Assert.AreEqual(20u, meta.num_joints);
            Assert.AreEqual(5u, meta.num_soa_joints);
            Assert.AreEqual(20, meta.joint_metas.Length);
            Assert.AreEqual(0, meta.joint_metas[0].index);
            Assert.AreEqual(-1, meta.joint_metas[0].parent);
            Assert.AreEqual("Hips", meta.joint_metas[0].name.ToString());
        }

        [TestMethod]
        public void TestAnimationMeta() {
            var meta = SkeletalResource.LoadAnimationMeta(ASSET_PATH + "Girl_Attack_01A.la-ozz");
            Assert.AreEqual(2.8f, meta.duration);
            Assert.AreEqual(20u, meta.num_tracks);
            Assert.AreEqual("Attack_01A", meta.name.ToString());
        }

        [TestMethod]
        public void TestRootMotionMeta() {
            var meta = SkeletalResource.LoadRootMotionMeta(ASSET_PATH + "Girl_Attack_01A.rm-ozz");
            Assert.AreEqual(true, meta.position_default.enabled);
            Assert.AreEqual(meta.position_default.whole_distance, meta.position_default.whole_distance_xz);
            Assert.AreEqual(0.0f, meta.position_default.whole_distance_y);
            Assert.AreEqual(true, meta.position_move.enabled);
            Assert.AreEqual(meta.position_move.whole_distance, meta.position_move.whole_distance_xz);
            Assert.AreEqual(0.0f, meta.position_move.whole_distance_y);
            Assert.AreEqual(true, meta.position_move_ex.enabled);
            Assert.AreEqual(meta.position_move_ex.whole_distance, meta.position_move_ex.whole_distance_xz);
            Assert.AreEqual(0.0f, meta.position_move_ex.whole_distance_y);
            Assert.AreEqual(false, meta.has_rotation);
        }

        [TestMethod]
        public void TestWeaponMotionMeta() {
            var meta = SkeletalResource.LoadWeaponMotionMeta(ASSET_PATH + "Girl_Attack_01A.wm-ozz", true);
            Assert.AreEqual(1u, meta.count);
            Assert.AreEqual("Axe", meta.names[0].ToString());
        }

        [TestMethod]
        public void TestHitMotionMeta() {
            var meta = SkeletalResource.LoadHitMotionMeta(ASSET_PATH + "TestDemo.hm-json", true);
            Assert.AreEqual(3, meta.track_groups.Length);
            Assert.AreEqual("Health", meta.track_groups[0].group.ToString());
            Assert.AreEqual(1u, meta.track_groups[0].count);
            Assert.AreEqual("Counter", meta.track_groups[1].group.ToString());
            Assert.AreEqual(1u, meta.track_groups[1].count);
            Assert.AreEqual("Axe", meta.track_groups[2].group.ToString());
            Assert.AreEqual(2u, meta.track_groups[2].count);
        }
    }

    [TestClass]
    public class TestSkeletalAnimator {
        [ClassInitialize]
        public static void TestInit(TestContext context) {
            Assert.AreEqual(SkeletalAnimator.SkeletonCount(), 0u);
            Assert.AreEqual(SkeletalAnimator.AnimationCount(), 0u);
            SkeletalAnimator.ClearUnused();

            SkeletalAnimator.LoadSkeleton(new Symbol("Girl.*"));
            SkeletalAnimator.LoadAnimation(new Symbol("Girl_Run_Empty.*"));
            SkeletalAnimator.LoadAnimation(new Symbol("Girl_RunStart_Empty.*"));
            SkeletalAnimator.Load(
                new Symbol[] { new Symbol("Girl.*") },
                new Symbol[] { new Symbol("Girl_RunStop_L_Empty.*"), new Symbol("Girl_RunStop_R_Empty.*") },
                new Symbol[] { }
            );

            Assert.AreEqual(SkeletalAnimator.SkeletonCount(), 1u);
            Assert.AreEqual(SkeletalAnimator.AnimationCount(), 4u);
            SkeletalAnimator.ClearUnused();
            Assert.AreEqual(SkeletalAnimator.SkeletonCount(), 0u);
            Assert.AreEqual(SkeletalAnimator.AnimationCount(), 0u);
        }

        [TestMethod]
        public void TestNewDelete() {
            var sb = new Symbol("Girl.*");
            SkeletalAnimator animator = new SkeletalAnimator(sb);
            Assert.IsNotNull(animator);
            animator.Dispose();
        }

        [TestMethod]
        public void TestSkeletonMeta() {
            using (var animator = new SkeletalAnimator(new Symbol("Girl.*"))) {
                var meta = animator.SkeletonMeta();
                Assert.AreEqual(54u, meta.num_joints);
                Assert.AreEqual(14u, meta.num_soa_joints);
                Assert.AreEqual(54, meta.joint_metas.Length);

                var j0 = meta.joint_metas[0];
                Assert.AreEqual(0, j0.index);
                Assert.AreEqual(-1, j0.parent);
                Assert.AreEqual("Hips", j0.name.ToString());

                var j1 = meta.joint_metas[53];
                Assert.AreEqual(53, j1.index);
                Assert.AreEqual(52, j1.parent);
                Assert.AreEqual("RightToes", j1.name.ToString());
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsVec<RsBoxDynStateActionAny> mock_skeleton_animator_state_actions();

        [TestMethod]
        public void TestUpdateAnimate() {
            using (var animator = new SkeletalAnimator(new Symbol("Girl.*"))) {
                var rest_poses = animator.ModelRestPoses();
                var a = rest_poses[0];
                Assert.AreEqual(54, rest_poses.Length);

                var state_actions = new RefVecBoxStateActionAny(mock_skeleton_animator_state_actions());
                animator.Update(state_actions);
                animator.Animate(0.5f);

                var model_poses = animator.ModelPoses();
                var b = model_poses[0];
                Assert.AreEqual(54, model_poses.Length);
            }

            System.Threading.Thread.Sleep(100);

            Assert.AreEqual(SkeletalAnimator.SkeletonCount(), 1u);
            Assert.AreEqual(SkeletalAnimator.AnimationCount(), 1u);
            SkeletalAnimator.ClearUnused();
            Assert.AreEqual(SkeletalAnimator.SkeletonCount(), 0u);
            Assert.AreEqual(SkeletalAnimator.AnimationCount(), 0u);
        }
    }

    [TestClass]
    public class TestSkeletalPlayer {
        const string ASSET_PATH = "../../../../../test-tmp/test-asset/";
        const string SKELETON = "Girl.ls-ozz";
        const string ANIMATION = "Girl_Run_Empty.la-ozz";

        [TestMethod]
        public void TestNewDelete() {
            SkeletalPlayer player = new SkeletalPlayer(ASSET_PATH + SKELETON);
            Assert.IsNotNull(player);
            player.Dispose();
        }

        [TestMethod]
        public void TestSkeletonMeta() {
            using (var player = new SkeletalPlayer(ASSET_PATH + SKELETON)) {
                var meta = player.SkeletonMeta();
                Assert.AreEqual(20u, meta.num_joints);
                Assert.AreEqual(5u, meta.num_soa_joints);
                Assert.AreEqual(20, meta.joint_metas.Length);

                var j0 = meta.joint_metas[0];
                Assert.AreEqual(0, j0.index);
                Assert.AreEqual(-1, j0.parent);
                Assert.AreEqual("Hips", j0.name.ToString());

                var j1 = meta.joint_metas[19];
                Assert.AreEqual(19, j1.index);
                Assert.AreEqual(18, j1.parent);
                Assert.AreEqual("RightFoot", j1.name.ToString());
            }
        }

        [TestMethod]
        public void TestUpdate() {
            using (var player = new SkeletalPlayer(ASSET_PATH + SKELETON)) {
                player.SetAnimations(new SkeletalAnimation[] { new SkeletalAnimation(ASSET_PATH + ANIMATION) });
                player.SetProgress(0.05f);
                player.Update();

                var rest_poses = player.ModelRestPoses();
                var a = rest_poses[0];
                Assert.AreEqual(20, rest_poses.Length);

                var model_poses = player.ModelPoses();
                var b = model_poses[0];
                Assert.AreEqual(20, model_poses.Length);

                var root_motion = player.RootMotion();
            }
        }
    }
}

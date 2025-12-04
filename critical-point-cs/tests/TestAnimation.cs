using CriticalPoint;
using System.Runtime.InteropServices;

namespace CriticalPointTests {
    [TestClass]
    public class TestSkeletalAnimator {
        const string SKELETON = "girl.ls-ozz";
        const string ANIMATION = "girl_run.la-ozz";

        [ClassInitialize]
        public static void TestInit(TestContext context) {
            Assert.AreEqual(SkeletalAnimator.SkeletonCount(), 0u);
            Assert.AreEqual(SkeletalAnimator.AnimationCount(), 0u);
            SkeletalAnimator.ClearUnused();

            SkeletalAnimator.LoadSkeleton(new Symbol("girl.*"));
            SkeletalAnimator.LoadAnimation(new Symbol("girl_run.*"));
            SkeletalAnimator.LoadAnimation(new Symbol("girl_run_start.*"));
            SkeletalAnimator.Load(
                new Symbol[] { new Symbol("girl.*") },
                new Symbol[] { new Symbol("girl_run_stop_l.*"), new Symbol("girl_run_stop_r.*") }
            );

            Assert.AreEqual(SkeletalAnimator.SkeletonCount(), 1u);
            Assert.AreEqual(SkeletalAnimator.AnimationCount(), 4u);
            SkeletalAnimator.ClearUnused();
            Assert.AreEqual(SkeletalAnimator.SkeletonCount(), 0u);
            Assert.AreEqual(SkeletalAnimator.AnimationCount(), 0u);
        }

        [TestMethod]
        public void TestNewDelete() {
            var sb = new Symbol("girl.*");
            SkeletalAnimator animator = new SkeletalAnimator(sb);
            Assert.IsNotNull(animator);
            animator.Dispose();
        }

        [TestMethod]
        public void TestSkeletonMeta() {
            using (var animator = new SkeletalAnimator(new Symbol("girl.*"))) {
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
            using (var animator = new SkeletalAnimator(new Symbol("girl.*"))) {
                var rest_poses = animator.ModelRestPoses();
                var a = rest_poses[0];
                Assert.AreEqual(54, rest_poses.Length);

                var state_actions = new RefVecBoxStateActionAny(mock_skeleton_animator_state_actions());
                animator.Update(state_actions);
                animator.Animate(0.5f);

                var model_out = animator.ModelOut();
                var b = model_out[0];
                Assert.AreEqual(54, model_out.Length);
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
        const string SKELETON = "girl.ls-ozz";
        const string ANIMATION = "girl_run.la-ozz";

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
                player.AddProgress(0.05f);
                player.Update();

                var rest_poses = player.ModelRestPoses();
                var a = rest_poses[0];
                Assert.AreEqual(20, rest_poses.Length);

                var model_out = player.ModelOut();
                var b = model_out[0];
                Assert.AreEqual(20, model_out.Length);

                var motion_out = player.RootMotionOut();
            }
        }
    }
}

using CriticalPoint;
using System.Runtime.InteropServices;

namespace CriticalPointTests {
    [TestClass]
    public class TestSkeletonResource {
        const string ASSET_PATH = "../../../../../critical-point/test-asset/";
        const string SKELETON = "girl_skeleton_logic.ozz";
        const string ANIMATION = "girl_animation_logic_stand_idle.ozz";

        [TestMethod]
        public void TestNewDelete() {
            var resource = new SkeletalResource(ASSET_PATH + SKELETON);
            Assert.AreEqual(resource.IsNull, false);
            resource.Dispose();
        }

        [TestMethod]
        public void TestAnimation() {
            var resource = new SkeletalResource(ASSET_PATH + SKELETON);
            var symbol = new Symbol(ANIMATION);
            resource.AddAnimation(symbol, ASSET_PATH + ANIMATION);
            Assert.AreEqual(resource.HasAnimation(symbol), true);
            resource.RemoveAnimation(symbol);
            Assert.AreEqual(resource.HasAnimation(symbol), false);
        }
    }

    [TestClass]
    public class TestSkeletonAnimator {
        const string ASSET_PATH = "../../../../../critical-point/test-asset/";
        const string SKELETON = "girl_skeleton_logic.ozz";
        const string ANIMATION = "girl_animation_logic_stand_idle.ozz";

        [TestMethod]
        public void TestNewDelete() {
            var resource = new SkeletalResource(ASSET_PATH + SKELETON);
            SkeletalAnimator animator = new SkeletalAnimator(resource);
            Assert.IsNotNull(animator);
            animator.Dispose();
        }

        [TestMethod]
        public void TestSkeletonMeta() {
            var resource = new SkeletalResource(ASSET_PATH + SKELETON);
            resource.AddAnimation(new Symbol(ANIMATION), ASSET_PATH + ANIMATION);

            using (var animator = new SkeletalAnimator(resource)) {
                var meta = animator.SkeletonMeta();
                Assert.AreEqual(67u, meta.num_joints);
                Assert.AreEqual(17u, meta.num_soa_joints);
                Assert.AreEqual(67, meta.joint_metas.Length);

                var j0 = meta.joint_metas[0];
                Assert.AreEqual(0, j0.index);
                Assert.AreEqual(-1, j0.parent);
                Assert.AreEqual("Hips", j0.name.ToString());
                
                var j1 = meta.joint_metas[66];
                Assert.AreEqual(66, j1.index);
                Assert.AreEqual(65, j1.parent);
                Assert.AreEqual("Bip01 R Toe0Nub", j1.name.ToString());
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsVec<RsBoxDynStateAction> mock_skeleton_animator_state_actions();

        [TestMethod]
        public void TestUpdateAnimate() {
            var resource = new SkeletalResource(ASSET_PATH + SKELETON);
            resource.AddAnimation(new Symbol(ANIMATION), ASSET_PATH + ANIMATION);

            using (var animator = new SkeletalAnimator(resource)) {
                var state_actions = new RefVecBoxStateAction(mock_skeleton_animator_state_actions());
                animator.Update(1, state_actions);
                animator.Animate();
                var local_out = animator.LocalOut();
                var a = local_out[0];
                Assert.AreEqual(17, local_out.Length);
                var model_out = animator.ModelOut();
                var b = model_out[0];
                Assert.AreEqual(67, model_out.Length);
            }
        }
    }
    
    [TestClass]
    public class TestOzzPlayer {
        const string ASSET_PATH = "../../../../../critical-point/test-asset/";
        const string SKELETON = "girl_skeleton_logic.ozz";
        const string ANIMATION = "girl_animation_logic_stand_idle.ozz";
        
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
                Assert.AreEqual(67u, meta.num_joints);
                Assert.AreEqual(17u, meta.num_soa_joints);
                Assert.AreEqual(67, meta.joint_metas.Length);

                var j0 = meta.joint_metas[0];
                Assert.AreEqual(0, j0.index);
                Assert.AreEqual(-1, j0.parent);
                Assert.AreEqual("Hips", j0.name.ToString());
                
                var j1 = meta.joint_metas[66];
                Assert.AreEqual(66, j1.index);
                Assert.AreEqual(65, j1.parent);
                Assert.AreEqual("Bip01 R Toe0Nub", j1.name.ToString());
            }
        }

        [TestMethod]
        public void TestUpdate() {
            using (var player = new SkeletalPlayer(ASSET_PATH + SKELETON)) {
                player.SetAnimation(ASSET_PATH + ANIMATION);
                player.Update(0.0f);
                var local_out = player.LocalOut();
                var a = local_out[0];
                Assert.AreEqual(17, local_out.Length);
                var model_out = player.ModelOut();
                var b = model_out[0];
                Assert.AreEqual(67, model_out.Length);
            }
        }
    }
}

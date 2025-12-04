using CriticalPoint;
using System.Runtime.InteropServices;

namespace CriticalPointTests {
    [TestClass]
    public class TestStateAny {
        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxDynStateAny mock_box_dyn_state_any();

        [TestMethod]
        public void TestBoxDynStateAny() {
            var any = mock_box_dyn_state_any().MakeBox();

            Assert.AreEqual(123ul, any.id);
            Assert.AreEqual(StateType.PlayerInit, any.typ);
            Assert.AreEqual(LogicType.Player, any.logic_typ);

            Assert.ThrowsException<NullReferenceException>(() => any.AsRefStateNpcInit());
            var ref_idle = any.AsRefStatePlayerInit();
            Assert.AreEqual("mock_skeleton.ozz", ref_idle.skeleton_file.TryRead());
            var idx = 0;
            foreach (var meta in ref_idle.animation_metas) {
                Assert.AreEqual(string.Format("mock_animation_{0}.ozz", idx++), meta.files.TryRead());
                Assert.AreEqual(false, meta.root_motion);
                Assert.AreEqual(false, meta.weapon_motion);
            }

            any.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcDynStateAny mock_arc_dyn_state_any();

        [TestMethod]
        public void TestArcDynStateAny() {
            var any = mock_arc_dyn_state_any().MakeArc();

            Assert.AreEqual(123ul, any.id);
            Assert.AreEqual(StateType.PlayerInit, any.typ);
            Assert.AreEqual(LogicType.Player, any.logic_typ);

            Assert.ThrowsException<NullReferenceException>(() => any.AsArcStateNpcInit());
            var idle = any.AsArcStatePlayerInit();
            Assert.AreEqual("mock_skeleton.ozz", idle.skeleton_file.TryRead());
            var idx = 0;
            foreach (var meta in idle.animation_metas) {
                Assert.AreEqual(string.Format("mock_animation_{0}.ozz", idx++), meta.files.TryRead());
            }

            Assert.ThrowsException<NullReferenceException>(() => any.AsWeakStateNpcInit());
            var weak_idle = any.AsWeakStatePlayerInit();
            Assert.AreEqual("mock_skeleton.ozz", weak_idle.skeleton_file.TryRead());
            Assert.AreEqual(string.Format("mock_animation_1.ozz"), weak_idle.animation_metas[1].files.TryRead());
            Assert.AreEqual(string.Format("mock_animation_2.ozz"), weak_idle.animation_metas[2].files.TryRead());

            var any2 = any.Arc();
            Assert.AreEqual(LogicType.Player, any2.logic_typ);

            var weak_any = any2.Weak();
            Assert.AreEqual(StateType.PlayerInit, weak_any.typ);

            Assert.AreEqual(3, any.StrongCount);
            any2.Dispose();
            any.Dispose();
            Assert.AreEqual(1, idle.StrongCount);
            idle.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxStateGameInit mock_box_state_game_init();

        [TestMethod]
        public void TestBoxStateGameInit() {
            var init = mock_box_state_game_init().MakeBox();

            Assert.AreEqual(4455ul, init.id);
            Assert.AreEqual(StateType.GameInit, init.typ);
            Assert.AreEqual(LogicType.Game, init.logic_typ);

            var ref_init = init.Ref();
            Assert.AreEqual(LogicType.Game, ref_init.logic_typ);

            init.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcStateGameInit mock_arc_state_game_init();

        [TestMethod]
        public void TestArcStateGameInit() {
            var init = mock_arc_state_game_init().MakeArc();

            Assert.AreEqual(4455ul, init.id);
            Assert.AreEqual(StateType.GameInit, init.typ);
            Assert.AreEqual(LogicType.Game, init.logic_typ);

            var init2 = init.Arc();
            Assert.AreEqual(LogicType.Game, init2.logic_typ);

            var weak_init = init.Weak();
            Assert.AreEqual(LogicType.Game, weak_init.logic_typ);

            Assert.AreEqual(2, init.StrongCount);
            init.Dispose();
            Assert.AreEqual(1, init2.StrongCount);
            init2.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxStateGameUpdate mock_box_state_game_update();

        [TestMethod]
        public void TestBoxStateGameUpdate() {
            var update = mock_box_state_game_update().MakeBox();

            Assert.AreEqual(4477ul, update.id);
            Assert.AreEqual(StateType.GameUpdate, update.typ);
            Assert.AreEqual(LogicType.Game, update.logic_typ);

            Assert.AreEqual(900u, update.frame);
            Assert.AreEqual(42u, update.id_gen_counter);

            var ref_update = update.Ref();
            Assert.AreEqual(42u, ref_update.id_gen_counter);

            update.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcStateGameUpdate mock_arc_state_game_update();

        [TestMethod]
        public void TestArcStateGameUpdate() {
            var update = mock_arc_state_game_update().MakeArc();

            Assert.AreEqual(4477ul, update.id);
            Assert.AreEqual(StateType.GameUpdate, update.typ);
            Assert.AreEqual(LogicType.Game, update.logic_typ);

            Assert.AreEqual(900u, update.frame);
            Assert.AreEqual(42u, update.id_gen_counter);

            var update2 = update.Arc();
            Assert.AreEqual(42u, update2.id_gen_counter);

            var weak_update = update.Weak();
            Assert.AreEqual(42u, weak_update.id_gen_counter);

            Assert.AreEqual(2, update.StrongCount);
            update.Dispose();
            Assert.AreEqual(1, update2.StrongCount);
            update2.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxStatePlayerInit mock_box_state_player_init();

        [TestMethod]
        public void TestBoxStatePlayerInit() {
            var init = mock_box_state_player_init().MakeBox();

            Assert.AreEqual(123ul, init.id);
            Assert.AreEqual(StateType.PlayerInit, init.typ);
            Assert.AreEqual(LogicType.Player, init.logic_typ);

            Assert.AreEqual("mock_skeleton.ozz", init.skeleton_file.TryRead());
            var idx = 0;
            foreach (var meta in init.animation_metas) {
                Assert.AreEqual(string.Format("mock_animation_{0}.ozz", idx++), meta.files.TryRead());
            }

            var ref_init = init.Ref();
            Assert.AreEqual(string.Format("mock_animation_2.ozz"), ref_init.animation_metas[2].files.TryRead());

            Assert.AreEqual("model.vrm", init.view_model.TryRead());
            Assert.AreEqual(new Vec3A(1.0f, 2.0f, 3.0f), init.init_position);
            Assert.AreEqual(new Vec2(0.0f, 1.0f), init.init_direction);

            init.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcStatePlayerInit mock_arc_state_player_init();

        [TestMethod]
        public void TestArcStatePlayerInit() {
            var init = mock_arc_state_player_init().MakeArc();

            Assert.AreEqual(123ul, init.id);
            Assert.AreEqual(StateType.PlayerInit, init.typ);
            Assert.AreEqual(LogicType.Player, init.logic_typ);

            Assert.AreEqual("mock_skeleton.ozz", init.skeleton_file.TryRead());
            var idx = 0;
            foreach (var meta in init.animation_metas) {
                Assert.AreEqual(string.Format("mock_animation_{0}.ozz", idx++), meta.files.TryRead());
            }

            var init2 = init.Arc();
            Assert.AreEqual(string.Format("mock_animation_0.ozz"), init2.animation_metas[0].files.TryRead());

            var weak_init = init.Weak();
            Assert.AreEqual(string.Format("mock_animation_2.ozz"), weak_init.animation_metas[2].files.TryRead());

            Assert.AreEqual(2, init.StrongCount);
            init.Dispose();
            Assert.AreEqual(1, init2.StrongCount);
            init2.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxStatePlayerUpdate mock_box_state_player_update();

        [TestMethod]
        public void TestBoxStatePlayerUpdate() {
            var update = mock_box_state_player_update().MakeBox();

            Assert.AreEqual(321ul, update.id);
            Assert.AreEqual(StateType.PlayerUpdate, update.typ);
            Assert.AreEqual(LogicType.Player, update.logic_typ);

            Assert.AreEqual(new Vec3A(4.0f, 5.0f, 6.0f), update.physics.velocity);
            Assert.AreEqual(new Vec3A(1.0f, 2.0f, 3.0f), update.physics.position);
            Assert.AreEqual(new Vec2(0.0f, -1.0f), update.physics.direction);
            Assert.AreEqual(2, update.actions.Length);
            Assert.AreEqual("Action.One.Idle", update.actions[0].tmpl_id.TryRead());
            Assert.AreEqual(0.207f, update.actions[0].AsRefStateActionIdle().fade_in_weight);
            Assert.AreEqual("Action.One.Run", update.actions[1].tmpl_id.TryRead());
            Assert.AreEqual(40u, update.actions[1].AsRefStateActionMove().derive_level);

            var ref_update = update.Ref();
            Assert.AreEqual(2, ref_update.actions.Length);

            update.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcStatePlayerUpdate mock_arc_state_player_update();

        [TestMethod]
        public void TestArcStatePlayerUpdate() {
            var update = mock_arc_state_player_update().MakeArc();

            Assert.AreEqual(321ul, update.id);
            Assert.AreEqual(StateType.PlayerUpdate, update.typ);
            Assert.AreEqual(LogicType.Player, update.logic_typ);

            Assert.AreEqual(new Vec3A(4.0f, 5.0f, 6.0f), update.physics.velocity);
            Assert.AreEqual(new Vec3A(1.0f, 2.0f, 3.0f), update.physics.position);
            Assert.AreEqual(new Vec2(0.0f, -1.0f), update.physics.direction);
            Assert.AreEqual(2, update.actions.Length);
            Assert.AreEqual("Action.One.Idle", update.actions[0].tmpl_id.TryRead());
            Assert.AreEqual(0.207f, update.actions[0].AsRefStateActionIdle().fade_in_weight);
            Assert.AreEqual("Action.One.Run", update.actions[1].tmpl_id.TryRead());
            Assert.AreEqual(40u, update.actions[1].AsRefStateActionMove().derive_level);

            var update2 = update.Arc();
            Assert.AreEqual(2, update.actions.Length);

            var weak_update = update.Weak();
            Assert.AreEqual(2, weak_update.actions.Length);

            Assert.AreEqual(2, update.StrongCount);
            update.Dispose();
            Assert.AreEqual(1, update2.StrongCount);
            update2.Dispose();
        }
    }
}

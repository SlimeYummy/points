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
            Assert.AreEqual(StateType.CharacterInit, any.typ);
            Assert.AreEqual(LogicType.Character, any.logic_typ);

            Assert.ThrowsException<NullReferenceException>(() => any.AsRefStateGameInit());
            var ref_idle = any.AsRefStateCharacterInit();
            Assert.AreEqual(true, ref_idle.is_player);
            Assert.AreEqual("mock_skeleton.ozz", ref_idle.skeleton_files.TryRead());
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
            Assert.AreEqual(StateType.CharacterInit, any.typ);
            Assert.AreEqual(LogicType.Character, any.logic_typ);

            Assert.ThrowsException<NullReferenceException>(() => any.AsArcStateGameInit());
            var idle = any.AsArcStateCharacterInit();
            Assert.AreEqual(true, idle.is_player);
            Assert.AreEqual("mock_skeleton.ozz", idle.skeleton_files.TryRead());
            var idx = 0;
            foreach (var meta in idle.animation_metas) {
                Assert.AreEqual(string.Format("mock_animation_{0}.ozz", idx++), meta.files.TryRead());
            }

            Assert.ThrowsException<NullReferenceException>(() => any.AsWeakStateGameInit());
            var weak_idle = any.AsWeakStateCharacterInit();
            Assert.AreEqual(true, weak_idle.is_player);
            Assert.AreEqual("mock_skeleton.ozz", weak_idle.skeleton_files.TryRead());
            Assert.AreEqual(string.Format("mock_animation_1.ozz"), weak_idle.animation_metas[1].files.TryRead());
            Assert.AreEqual(string.Format("mock_animation_2.ozz"), weak_idle.animation_metas[2].files.TryRead());

            var any2 = any.Arc();
            Assert.AreEqual(LogicType.Character, any2.logic_typ);

            var weak_any = any2.Weak();
            Assert.AreEqual(StateType.CharacterInit, weak_any.typ);

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
            Assert.AreEqual(108u, update.gene.player_id);
            Assert.AreEqual(1000u, update.gene.auto_gen_id);
            Assert.AreEqual(0u, update.gene.action_id);

            Assert.AreEqual(1, update.hit_events.Length);
            Assert.AreEqual(100u, update.hit_events[0].src_chara_id);
            Assert.AreEqual(101u, update.hit_events[0].dst_chara_id);
            Assert.AreEqual("group-name", update.hit_events[0].group.TryRead());

            var ref_update = update.Ref();
            Assert.AreEqual(900u, ref_update.frame);
            Assert.AreEqual(108u, ref_update.gene.player_id);
            Assert.AreEqual(1000u, ref_update.gene.auto_gen_id);
            Assert.AreEqual(0u, ref_update.gene.action_id);
            Assert.AreEqual(1, ref_update.hit_events.Length);

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
            Assert.AreEqual(108u, update.gene.player_id);
            Assert.AreEqual(1000u, update.gene.auto_gen_id);
            Assert.AreEqual(0u, update.gene.action_id);

            Assert.AreEqual(1, update.hit_events.Length);
            Assert.AreEqual(100u, update.hit_events[0].src_chara_id);
            Assert.AreEqual(101u, update.hit_events[0].dst_chara_id);
            Assert.AreEqual("group-name", update.hit_events[0].group.TryRead());

            var update2 = update.Arc();
            Assert.AreEqual(900u, update2.frame);
            Assert.AreEqual(108u, update2.gene.player_id);
            Assert.AreEqual(1000u, update2.gene.auto_gen_id);
            Assert.AreEqual(0u, update2.gene.action_id);
            Assert.AreEqual(1, update2.hit_events.Length);

            var weak_update = update.Weak();
            Assert.AreEqual(900u, weak_update.frame);
            Assert.AreEqual(108u, weak_update.gene.player_id);
            Assert.AreEqual(1000u, weak_update.gene.auto_gen_id);
            Assert.AreEqual(0u, weak_update.gene.action_id);
            Assert.AreEqual(1, weak_update.hit_events.Length);

            Assert.AreEqual(2, update.StrongCount);
            update.Dispose();
            Assert.AreEqual(1, update2.StrongCount);
            update2.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxStateCharacterInit mock_box_state_character_init();

        [TestMethod]
        public void TestBoxStatePlayerInit() {
            var init = mock_box_state_character_init().MakeBox();

            Assert.AreEqual(123ul, init.id);
            Assert.AreEqual(StateType.CharacterInit, init.typ);
            Assert.AreEqual(LogicType.Character, init.logic_typ);

            Assert.AreEqual(true, init.is_player);
            Assert.AreEqual("mock_skeleton.ozz", init.skeleton_files.TryRead());
            var idx = 0;
            foreach (var meta in init.animation_metas) {
                Assert.AreEqual(string.Format("mock_animation_{0}.ozz", idx++), meta.files.TryRead());
            }

            var ref_init = init.Ref();
            Assert.AreEqual(string.Format("mock_animation_2.ozz"), ref_init.animation_metas[2].files.TryRead());

            Assert.AreEqual("model.vrm", init.view_model.ToString());
            Assert.AreEqual(new Vec3A(1.0f, 2.0f, 3.0f), init.init_position);
            Assert.AreEqual(new Vec2(0.0f, 1.0f), init.init_direction);

            init.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcStateCharacterInit mock_arc_state_character_init();

        [TestMethod]
        public void TestArcStatePlayerInit() {
            var init = mock_arc_state_character_init().MakeArc();

            Assert.AreEqual(123ul, init.id);
            Assert.AreEqual(StateType.CharacterInit, init.typ);
            Assert.AreEqual(LogicType.Character, init.logic_typ);

            Assert.AreEqual(true, init.is_player);
            Assert.AreEqual("mock_skeleton.ozz", init.skeleton_files.TryRead());
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
        private static extern unsafe RsBoxStateCharacterUpdate mock_box_state_character_update();

        [TestMethod]
        public void TestBoxStatePlayerUpdate() {
            var update = mock_box_state_character_update().MakeBox();

            Assert.AreEqual(321ul, update.id);
            Assert.AreEqual(StateType.CharacterUpdate, update.typ);
            Assert.AreEqual(LogicType.Character, update.logic_typ);

            Assert.AreEqual(new Vec3A(4.0f, 5.0f, 6.0f), update.physics.velocity);
            Assert.AreEqual(new Vec3A(1.0f, 2.0f, 3.0f), update.physics.position);
            Assert.AreEqual(new Vec2(0.0f, -1.0f), update.physics.direction);

            Assert.AreEqual(33u, update.action.event_cursor_id);
            Assert.AreEqual(true, update.action.derive_keeping.action_id.IsInvalid);
            Assert.AreEqual(0u, update.action.derive_keeping.derive_level);
            Assert.AreEqual(0, update.action.derive_keeping.end_time);
            Assert.AreEqual(true, update.action.action_changed);
            Assert.AreEqual(false, update.action.animation_changed);

            Assert.AreEqual(6.0f, update.value.hit_lag_time.begin);
            Assert.AreEqual(9.5f, update.value.hit_lag_time.end);

            Assert.AreEqual(2, update.actions.Length);
            Assert.AreEqual("Action.One.Idle", update.actions[0].tmpl_id.TryRead());
            Assert.AreEqual(0.207f, update.actions[0].AsRefStateActionIdle().fade_in_weight);
            Assert.AreEqual("Action.One.Run", update.actions[1].tmpl_id.TryRead());
            Assert.AreEqual(70u, update.actions[1].AsRefStateActionMove().derive_level);
            for (int idx = 0; idx < update.custom_events.Length; ++idx) {
                Assert.AreEqual($"Action.One.Attack^1/Event{idx}", update.custom_events[idx].AsEventString());
            }

            var ref_update = update.Ref();
            Assert.AreEqual(2, ref_update.actions.Length);
            Assert.AreEqual(3, ref_update.custom_events.Length);

            update.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcStateCharacterUpdate mock_arc_state_character_update();

        [TestMethod]
        public void TestArcStatePlayerUpdate() {
            var update = mock_arc_state_character_update().MakeArc();

            Assert.AreEqual(321ul, update.id);
            Assert.AreEqual(StateType.CharacterUpdate, update.typ);
            Assert.AreEqual(LogicType.Character, update.logic_typ);

            Assert.AreEqual(new Vec3A(4.0f, 5.0f, 6.0f), update.physics.velocity);
            Assert.AreEqual(new Vec3A(1.0f, 2.0f, 3.0f), update.physics.position);
            Assert.AreEqual(new Vec2(0.0f, -1.0f), update.physics.direction);

            Assert.AreEqual(33u, update.action.event_cursor_id);
            Assert.AreEqual(true, update.action.derive_keeping.action_id.IsInvalid);
            Assert.AreEqual(0u, update.action.derive_keeping.derive_level);
            Assert.AreEqual(0, update.action.derive_keeping.end_time);
            Assert.AreEqual(true, update.action.action_changed);
            Assert.AreEqual(false, update.action.animation_changed);

            Assert.AreEqual(6.0f, update.value.hit_lag_time.begin);
            Assert.AreEqual(9.5f, update.value.hit_lag_time.end);

            Assert.AreEqual(2, update.actions.Length);
            Assert.AreEqual("Action.One.Idle", update.actions[0].tmpl_id.TryRead());
            Assert.AreEqual(0.207f, update.actions[0].AsRefStateActionIdle().fade_in_weight);
            Assert.AreEqual("Action.One.Run", update.actions[1].tmpl_id.TryRead());
            Assert.AreEqual(70u, update.actions[1].AsRefStateActionMove().derive_level);
            for (int idx = 0; idx < update.custom_events.Length; ++idx) {
                Assert.AreEqual($"Action.One.Attack^1/Event{idx}", update.custom_events[idx].AsEventString());
            }

            var update2 = update.Arc();
            Assert.AreEqual(2, update.actions.Length);
            Assert.AreEqual(3, update.custom_events.Length);

            var weak_update = update.Weak();
            Assert.AreEqual(2, weak_update.actions.Length);
            Assert.AreEqual(3, weak_update.custom_events.Length);

            Assert.AreEqual(2, update.StrongCount);
            update.Dispose();
            Assert.AreEqual(1, update2.StrongCount);
            update2.Dispose();
        }
    }
}

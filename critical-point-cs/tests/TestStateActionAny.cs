using CriticalPoint;
using System.Runtime.InteropServices;

namespace CriticalPointTests {
    [TestClass]
    public class TestStateActionAny {
        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxDynStateActionAny mock_box_dyn_state_action_any();

        [TestMethod]
        public void TestBoxDynStateAction() {
            var action = mock_box_dyn_state_action_any().MakeBox();
            Assert.AreEqual("mock_action_idle_2", action.animations[1].files.TryRead());

            Assert.ThrowsException<NullReferenceException>(() => action.AsRefStateActionMove());
            var ref_idle = action.AsRefStateActionIdle();
            Assert.AreEqual(555u, ref_idle.first_frame);

            var ref_action = action.Ref();
            Assert.AreEqual(3456u, ref_action.animations[1].animation_id);

            action.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcDynStateActionAny mock_arc_dyn_state_action_any();

        [TestMethod]
        public void TestArcDynStateAction() {
            var raw = mock_arc_dyn_state_action_any();
            ArcDynStateActionAny action = raw.MakeArc();
            Assert.AreEqual("mock_action_idle_2", action.animations[1].files.TryRead());

            Assert.ThrowsException<NullReferenceException>(() => action.AsArcStateActionMove());
            var idle = action.AsArcStateActionIdle();
            Assert.AreEqual(555u, idle.first_frame);

            Assert.ThrowsException<NullReferenceException>(() => action.AsWeakStateActionMove());
            var weak_idle = action.AsWeakStateActionIdle();
            Assert.AreEqual(0.207f, weak_idle.fade_in_weight);

            var weak_action = action.Weak();
            Assert.AreEqual("mock_action_idle_2", weak_action.animations[1].files.TryRead());

            var action2 = action.Arc();
            Assert.AreEqual("mock_action_idle_1", action2.animations[0].files.TryRead());

            Assert.AreEqual(3, action.StrongCount);
            action.Dispose();
            action2.Dispose();
            Assert.AreEqual(1, idle.StrongCount);
            idle.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxStateActionIdle mock_box_state_action_idle();

        [TestMethod]
        public void TestBoxStateActionIdle() {
            var idle = mock_box_state_action_idle().MakeBox();

            Assert.AreEqual(1234ul, idle.id);
            Assert.AreEqual("Action.One.Idle", idle.tmpl_id.TryRead());
            Assert.AreEqual(StateActionType.Idle, idle.typ);
            Assert.AreEqual(TmplType.ActionIdle, idle.tmpl_typ);
            Assert.AreEqual(LogicActionStatus.Activing, idle.status);
            Assert.AreEqual(555u, idle.first_frame);
            Assert.AreEqual(uint.MaxValue, idle.last_frame);
            Assert.AreEqual(0.207f, idle.fade_in_weight);
            Assert.AreEqual(50u, idle.derive_level);
            Assert.AreEqual(100u, idle.poise_level);

            Assert.AreEqual("mock_action_idle_1", idle.animations[0].files.TryRead());
            Assert.AreEqual(9999u, idle.animations[0].animation_id);
            Assert.AreEqual(true, idle.animations[0].weapon_motion);
            Assert.AreEqual(0.125f, idle.animations[0].ratio);
            Assert.AreEqual(0.333f, idle.animations[0].weight);

            Assert.AreEqual("mock_action_idle_2", idle.animations[1].files.TryRead());
            Assert.AreEqual(3456u, idle.animations[1].animation_id);
            Assert.AreEqual(false, idle.animations[1].weapon_motion);
            Assert.AreEqual(0.6f, idle.animations[1].ratio);
            Assert.AreEqual(0.7f, idle.animations[1].weight);

            Assert.AreEqual("", idle.animations[2].files.TryRead());
            Assert.AreEqual(0xffffu, idle.animations[2].animation_id);
            Assert.AreEqual(false, idle.animations[2].weapon_motion);
            Assert.AreEqual(0f, idle.animations[2].ratio);
            Assert.AreEqual(1f, idle.animations[2].weight);

            Assert.AreEqual(ActionIdleMode.Idle, idle.mode);
            Assert.AreEqual(3.3f, idle.idle_time);
            Assert.AreEqual(4.4f, idle.ready_time);
            Assert.AreEqual(1.5f, idle.auto_idle_time);
            Assert.AreEqual(0.5f, idle.switch_time);

            var ref_idle = idle.Ref();
            Assert.AreEqual(4.4f, ref_idle.ready_time);

            idle.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcStateActionIdle mock_arc_state_action_idle();

        [TestMethod]
        public void TestArcStateActionIdle() {
            var idle = mock_arc_state_action_idle().MakeArc();

            Assert.AreEqual(1234ul, idle.id);
            Assert.AreEqual("Action.One.Idle", idle.tmpl_id.TryRead());
            Assert.AreEqual(StateActionType.Idle, idle.typ);
            Assert.AreEqual(TmplType.ActionIdle, idle.tmpl_typ);
            Assert.AreEqual(LogicActionStatus.Activing, idle.status);
            Assert.AreEqual(555u, idle.first_frame);
            Assert.AreEqual(uint.MaxValue, idle.last_frame);
            Assert.AreEqual(0.207f, idle.fade_in_weight);
            Assert.AreEqual(50u, idle.derive_level);
            Assert.AreEqual(100u, idle.poise_level);

            Assert.AreEqual("mock_action_idle_1", idle.animations[0].files.TryRead());
            Assert.AreEqual(9999u, idle.animations[0].animation_id);
            Assert.AreEqual(true, idle.animations[0].weapon_motion);
            Assert.AreEqual(0.125f, idle.animations[0].ratio);
            Assert.AreEqual(0.333f, idle.animations[0].weight);

            Assert.AreEqual("mock_action_idle_2", idle.animations[1].files.TryRead());
            Assert.AreEqual(3456u, idle.animations[1].animation_id);
            Assert.AreEqual(false, idle.animations[1].weapon_motion);
            Assert.AreEqual(0.6f, idle.animations[1].ratio);
            Assert.AreEqual(0.7f, idle.animations[1].weight);

            Assert.AreEqual("", idle.animations[2].files.TryRead());
            Assert.AreEqual(0xffffu, idle.animations[2].animation_id);
            Assert.AreEqual(false, idle.animations[2].weapon_motion);
            Assert.AreEqual(0f, idle.animations[2].ratio);
            Assert.AreEqual(1f, idle.animations[2].weight);

            Assert.AreEqual(ActionIdleMode.Idle, idle.mode);
            Assert.AreEqual(3.3f, idle.idle_time);
            Assert.AreEqual(4.4f, idle.ready_time);
            Assert.AreEqual(1.5f, idle.auto_idle_time);
            Assert.AreEqual(0.5f, idle.switch_time);

            var idle2 = idle.Arc();
            Assert.AreEqual(4.4f, idle2.ready_time);

            var weak_idle = idle2.Weak();
            Assert.AreEqual(0.207f, weak_idle.fade_in_weight);

            Assert.AreEqual(2, idle.StrongCount);
            idle.Dispose();
            Assert.AreEqual(1, idle2.WeakCount);
            idle2.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxStateActionMove mock_box_state_action_move();

        [TestMethod]
        public void TestBoxStateActionMove() {
            var move = mock_box_state_action_move().MakeBox();

            Assert.AreEqual(783ul, move.id);
            Assert.AreEqual("Action.One.Run", move.tmpl_id.TryRead());
            Assert.AreEqual(StateActionType.Move, move.typ);
            Assert.AreEqual(TmplType.ActionMove, move.tmpl_typ);
            Assert.AreEqual(LogicActionStatus.Activing, move.status);
            Assert.AreEqual(123u, move.first_frame);
            Assert.AreEqual(uint.MaxValue, move.last_frame);
            Assert.AreEqual(0.77f, move.fade_in_weight);
            Assert.AreEqual(70u, move.derive_level);
            Assert.AreEqual(68u, move.poise_level);

            Assert.AreEqual("mock_action_move_1", move.animations[0].files.TryRead());
            Assert.AreEqual(888u, move.animations[0].animation_id);
            Assert.AreEqual(true, move.animations[0].weapon_motion);
            Assert.AreEqual(0.02f, move.animations[0].ratio);
            Assert.AreEqual(0.287f, move.animations[0].weight);

            Assert.AreEqual("mock_action_move_2", move.animations[1].files.TryRead());
            Assert.AreEqual(3456, move.animations[1].animation_id);
            Assert.AreEqual(false, move.animations[1].weapon_motion);
            Assert.AreEqual(0.875f, move.animations[1].ratio);
            Assert.AreEqual(0.46f, move.animations[1].weight);

            Assert.AreEqual("", move.animations[2].files.TryRead());
            Assert.AreEqual(0xffffu, move.animations[2].animation_id);
            Assert.AreEqual(false, move.animations[2].weapon_motion);
            Assert.AreEqual(0f, move.animations[2].ratio);
            Assert.AreEqual(1f, move.animations[2].weight);

            Assert.AreEqual(ActionMoveMode.Move, move.mode);
            Assert.AreEqual(false, move.smooth_move_switch);
            Assert.AreEqual(1.5f, move.current_time);
            Assert.AreEqual(1, move.start_anim_idx);
            Assert.AreEqual(2, move.turn_anim_idx);
            Assert.AreEqual(3, move.stop_anim_idx);

            Assert.AreEqual(0, move.root_motion.local_id);
            Assert.AreEqual(RootTrackName.Default, move.root_motion.pos_track);
            Assert.AreEqual(0f, move.root_motion.ratio);
            Assert.AreEqual(new Vec3A(-5.0f, -4.0f, -3.0f), move.root_motion.current_pos);
            Assert.AreEqual(new Vec3A(-2.0f, -1.0f, 0.0f), move.root_motion.previous_pos);
            Assert.AreEqual(new Vec3A(1.0f, 1.0f, 1.0f), move.root_motion.pos_delta);

            Assert.AreEqual(new Vec2(0.0f, -1.0f), move.start_turn_angle_step);
            Assert.AreEqual(0.5f, move.smooth_move_start_speed);
            Assert.AreEqual(1.0f, move.local_fade_in_weight);
            Assert.AreEqual(0.57f, move.anim_offset_time);
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcStateActionMove mock_arc_state_action_move();

        [TestMethod]
        public void TestArcStateActionMove() {
            var move = mock_arc_state_action_move().MakeArc();

            Assert.AreEqual(783ul, move.id);
            Assert.AreEqual("Action.One.Run", move.tmpl_id.TryRead());
            Assert.AreEqual(StateActionType.Move, move.typ);
            Assert.AreEqual(TmplType.ActionMove, move.tmpl_typ);
            Assert.AreEqual(LogicActionStatus.Activing, move.status);
            Assert.AreEqual(123u, move.first_frame);
            Assert.AreEqual(uint.MaxValue, move.last_frame);
            Assert.AreEqual(0.77f, move.fade_in_weight);
            Assert.AreEqual(70u, move.derive_level);
            Assert.AreEqual(68u, move.poise_level);

            Assert.AreEqual("mock_action_move_1", move.animations[0].files.TryRead());
            Assert.AreEqual(888u, move.animations[0].animation_id);
            Assert.AreEqual(true, move.animations[0].weapon_motion);
            Assert.AreEqual(0.02f, move.animations[0].ratio);
            Assert.AreEqual(0.287f, move.animations[0].weight);

            Assert.AreEqual("mock_action_move_2", move.animations[1].files.TryRead());
            Assert.AreEqual(3456, move.animations[1].animation_id);
            Assert.AreEqual(false, move.animations[1].weapon_motion);
            Assert.AreEqual(0.875f, move.animations[1].ratio);
            Assert.AreEqual(0.46f, move.animations[1].weight);

            Assert.AreEqual("", move.animations[2].files.TryRead());
            Assert.AreEqual(0xffffu, move.animations[2].animation_id);
            Assert.AreEqual(false, move.animations[2].weapon_motion);
            Assert.AreEqual(0f, move.animations[2].ratio);
            Assert.AreEqual(1f, move.animations[2].weight);

            Assert.AreEqual(ActionMoveMode.Move, move.mode);
            Assert.AreEqual(false, move.smooth_move_switch);
            Assert.AreEqual(1.5f, move.current_time);
            Assert.AreEqual(1, move.start_anim_idx);
            Assert.AreEqual(2, move.turn_anim_idx);
            Assert.AreEqual(3, move.stop_anim_idx);

            Assert.AreEqual(0, move.root_motion.local_id);
            Assert.AreEqual(0f, move.root_motion.ratio);
            Assert.AreEqual(RootTrackName.Default, move.root_motion.pos_track);
            Assert.AreEqual(0f, move.root_motion.ratio);
            Assert.AreEqual(new Vec3A(-5.0f, -4.0f, -3.0f), move.root_motion.current_pos);
            Assert.AreEqual(new Vec3A(-2.0f, -1.0f, 0.0f), move.root_motion.previous_pos);
            Assert.AreEqual(new Vec3A(1.0f, 1.0f, 1.0f), move.root_motion.pos_delta);

            Assert.AreEqual(new Vec2(0.0f, -1.0f), move.start_turn_angle_step);
            Assert.AreEqual(0.5f, move.smooth_move_start_speed);
            Assert.AreEqual(1.0f, move.local_fade_in_weight);
            Assert.AreEqual(0.57f, move.anim_offset_time);

            var move2 = move.Arc();
            Assert.AreEqual(new Vec3A(-5.0f, -4.0f, -3.0f), move.root_motion.current_pos);

            var weak_move = move2.Weak();
            Assert.AreEqual(new Vec2(0.0f, -1.0f), weak_move.start_turn_angle_step);

            Assert.AreEqual(2, move.StrongCount);
            move.Dispose();
            Assert.AreEqual(1, move2.WeakCount);
            move2.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxStateActionGeneral mock_box_state_action_general();

        [TestMethod]
        public void TestBoxStateActionGeneral() {
            var gen = mock_box_state_action_general().MakeBox();

            Assert.AreEqual(5551ul, gen.id);
            Assert.AreEqual("Action.One.Attack^1", gen.tmpl_id.TryRead());
            Assert.AreEqual(StateActionType.General, gen.typ);
            Assert.AreEqual(TmplType.ActionGeneral, gen.tmpl_typ);
            Assert.AreEqual(LogicActionStatus.Activing, gen.status);
            Assert.AreEqual(891u, gen.first_frame);
            Assert.AreEqual(uint.MaxValue, gen.last_frame);
            Assert.AreEqual(0.112f, gen.fade_in_weight);
            Assert.AreEqual(9u, gen.derive_level);
            Assert.AreEqual(13u, gen.poise_level);

            Assert.AreEqual("mock_action_gen_1", gen.animations[0].files.TryRead());
            Assert.AreEqual(81u, gen.animations[0].animation_id);
            Assert.AreEqual(true, gen.animations[0].weapon_motion);
            Assert.AreEqual(0.66f, gen.animations[0].ratio);
            Assert.AreEqual(0.74f, gen.animations[0].weight);

            Assert.AreEqual("", gen.animations[1].files.TryRead());
            Assert.AreEqual(0xffffu, gen.animations[1].animation_id);
            Assert.AreEqual(false, gen.animations[1].weapon_motion);
            Assert.AreEqual(0f, gen.animations[1].ratio);
            Assert.AreEqual(1f, gen.animations[1].weight);

            Assert.AreEqual(0.98f, gen.current_time);
            Assert.AreEqual(1.0f, gen.from_rotation);
            Assert.AreEqual(2.0f, gen.to_rotation);
            Assert.AreEqual(1.5f, gen.current_rotation);
            Assert.AreEqual(new TimeRange { begin = 10.0f, end = 20.0f }, gen.rotation_time);

            Assert.AreEqual(RootTrackName.Move, gen.root_motion.pos_track);
            Assert.AreEqual(0.9f, gen.root_motion.ratio);
            Assert.AreEqual(new Vec3A(1.0f, 2.0f, 3.0f), gen.root_motion.current_pos);
            Assert.AreEqual(new Vec3A(7.0f, 7.0f, 7.0f), gen.root_motion.previous_pos);
            Assert.AreEqual(new Vec3A(4.0f, 5.0f, 6.0f), gen.root_motion.pos_delta);
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcStateActionGeneral mock_arc_state_action_general();

        [TestMethod]
        public void TestArcStateActionGeneral() {
            var gen = mock_arc_state_action_general().MakeArc();

            Assert.AreEqual(5551ul, gen.id);
            Assert.AreEqual("Action.One.Attack^1", gen.tmpl_id.TryRead());
            Assert.AreEqual(StateActionType.General, gen.typ);
            Assert.AreEqual(TmplType.ActionGeneral, gen.tmpl_typ);
            Assert.AreEqual(LogicActionStatus.Activing, gen.status);
            Assert.AreEqual(891u, gen.first_frame);
            Assert.AreEqual(uint.MaxValue, gen.last_frame);
            Assert.AreEqual(0.112f, gen.fade_in_weight);
            Assert.AreEqual(9u, gen.derive_level);
            Assert.AreEqual(13u, gen.poise_level);

            Assert.AreEqual("mock_action_gen_1", gen.animations[0].files.TryRead());
            Assert.AreEqual(81u, gen.animations[0].animation_id);
            Assert.AreEqual(true, gen.animations[0].weapon_motion);
            Assert.AreEqual(0.66f, gen.animations[0].ratio);
            Assert.AreEqual(0.74f, gen.animations[0].weight);

            Assert.AreEqual("", gen.animations[1].files.TryRead());
            Assert.AreEqual(0xffffu, gen.animations[1].animation_id);
            Assert.AreEqual(false, gen.animations[1].weapon_motion);
            Assert.AreEqual(0f, gen.animations[1].ratio);
            Assert.AreEqual(1f, gen.animations[1].weight);

            Assert.AreEqual(0.98f, gen.current_time);
            Assert.AreEqual(1.0f, gen.from_rotation);
            Assert.AreEqual(2.0f, gen.to_rotation);
            Assert.AreEqual(1.5f, gen.current_rotation);
            Assert.AreEqual(new TimeRange { begin = 10.0f, end = 20.0f }, gen.rotation_time);

            Assert.AreEqual(RootTrackName.Move, gen.root_motion.pos_track);
            Assert.AreEqual(0.9f, gen.root_motion.ratio);
            Assert.AreEqual(new Vec3A(1.0f, 2.0f, 3.0f), gen.root_motion.current_pos);
            Assert.AreEqual(new Vec3A(7.0f, 7.0f, 7.0f), gen.root_motion.previous_pos);
            Assert.AreEqual(new Vec3A(4.0f, 5.0f, 6.0f), gen.root_motion.pos_delta);

            var gen2 = gen.Arc();
            Assert.AreEqual(0.98f, gen2.current_time);

            var weak_gen = gen2.Weak();
            Assert.AreEqual(1.5f, weak_gen.current_rotation);

            Assert.AreEqual(2, gen.StrongCount);
            gen.Dispose();
            Assert.AreEqual(1, gen2.WeakCount);
            gen2.Dispose();
        }
    }
}

using CriticalPoint;
using System.Runtime.InteropServices;

namespace CriticalPointTests {
    [TestClass]
    public class TestStateAction {
        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxDynStateAction mock_box_dyn_state_action();

        [TestMethod]
        public void TestBoxDynStateAction() {
            var action = mock_box_dyn_state_action().MakeBox();
            Assert.AreEqual("mock_action_idle_2.ozz", action.animations[1].file.TryRead());

            Assert.ThrowsException<NullReferenceException>(() => action.AsRefStateActionMove());
            var ref_idle = action.AsRefStateActionIdle();
            Assert.AreEqual(21u, ref_idle.switch_progress);

            var ref_action = action.Ref();
            Assert.AreEqual(3456u, ref_action.animations[1].animation_id);

            action.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcDynStateAction mock_arc_dyn_state_action();

        [TestMethod]
        public void TestArcDynStateAction() {
            var raw = mock_arc_dyn_state_action();
            ArcDynStateAction action = raw.MakeArc();
            Assert.AreEqual("mock_action_idle_2.ozz", action.animations[1].file.TryRead());

            Assert.ThrowsException<NullReferenceException>(() => action.AsArcStateActionMove());
            var idle = action.AsArcStateActionIdle();
            Assert.AreEqual(21u, idle.switch_progress);

            Assert.ThrowsException<NullReferenceException>(() => action.AsWeakStateActionMove());
            var weak_idle = action.AsWeakStateActionIdle();
            Assert.AreEqual(207u, weak_idle.enter_progress);

            var weak_action = action.Weak();
            Assert.AreEqual("mock_action_idle_2.ozz", weak_action.animations[1].file.TryRead());

            var action2 = action.Arc();
            Assert.AreEqual("mock_action_idle_1.ozz", action2.animations[0].file.TryRead());

            Assert.AreEqual(3, action.StrongCount);
            action.Dispose();
            action2.Dispose();
            Assert.AreEqual(1, idle.StrongCount);
            idle.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsBoxStateActionIdle mock_box_state_action_idle();

        [TestMethod]
        public void TestBoxStateActionIdel() {
            var idle = mock_box_state_action_idle().MakeBox();

            Assert.AreEqual(1234ul, idle.id);
            Assert.AreEqual("Mock.ActionIdle", idle.tmpl_id.TryRead());
            Assert.AreEqual(StateActionType.Idle, idle.typ);
            Assert.AreEqual(TmplType.ActionIdle, idle.tmpl_typ);
            Assert.AreEqual(555u, idle.spawn_frame);
            Assert.AreEqual(uint.MaxValue, idle.death_frame);
            Assert.AreEqual(207u, idle.enter_progress);
            Assert.AreEqual(false, idle.is_leaving);
            Assert.AreEqual(7744u, idle.event_idx);
            Assert.AreEqual(50u, idle.derive_level);
            Assert.AreEqual(100u, idle.antibreak_level);
            Assert.AreEqual(0.667f, idle.body_ratio);

            Assert.AreEqual("mock_action_idle_1.ozz", idle.animations[0].file.TryRead());
            Assert.AreEqual(9999u, idle.animations[0].animation_id);
            Assert.AreEqual(0.125f, idle.animations[0].ratio);
            Assert.AreEqual(0.333f, idle.animations[0].weight);

            Assert.AreEqual("mock_action_idle_2.ozz", idle.animations[1].file.TryRead());
            Assert.AreEqual(3456u, idle.animations[1].animation_id);
            Assert.AreEqual(0.6f, idle.animations[1].ratio);
            Assert.AreEqual(0.7f, idle.animations[1].weight);

            Assert.AreEqual("", idle.animations[2].file.TryRead());
            Assert.AreEqual(0u, idle.animations[2].animation_id);
            Assert.AreEqual(0f, idle.animations[2].ratio);
            Assert.AreEqual(0f, idle.animations[2].weight);

            Assert.AreEqual(ActionIdleMode.Idle, idle.mode);
            Assert.AreEqual(30u, idle.idle_progress);
            Assert.AreEqual(40u, idle.ready_progress);
            Assert.AreEqual(0u, idle.idle_timer);
            Assert.AreEqual(21u, idle.switch_progress);

            var ref_idle = idle.Ref();
            Assert.AreEqual(40u, ref_idle.ready_progress);

            idle.Dispose();
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe RsArcStateActionIdle mock_arc_state_action_idle();

        [TestMethod]
        public void TestArcStateActionIdle() {
            var idle = mock_arc_state_action_idle().MakeArc();

            Assert.AreEqual(1234ul, idle.id);
            Assert.AreEqual("Mock.ActionIdle", idle.tmpl_id.TryRead());
            Assert.AreEqual(StateActionType.Idle, idle.typ);
            Assert.AreEqual(TmplType.ActionIdle, idle.tmpl_typ);
            Assert.AreEqual(555u, idle.spawn_frame);
            Assert.AreEqual(uint.MaxValue, idle.death_frame);
            Assert.AreEqual(207u, idle.enter_progress);
            Assert.AreEqual(false, idle.is_leaving);
            Assert.AreEqual(7744u, idle.event_idx);
            Assert.AreEqual(50u, idle.derive_level);
            Assert.AreEqual(100u, idle.antibreak_level);
            Assert.AreEqual(0.667f, idle.body_ratio);

            Assert.AreEqual("mock_action_idle_1.ozz", idle.animations[0].file.TryRead());
            Assert.AreEqual(9999u, idle.animations[0].animation_id);
            Assert.AreEqual(0.125f, idle.animations[0].ratio);
            Assert.AreEqual(0.333f, idle.animations[0].weight);

            Assert.AreEqual("mock_action_idle_2.ozz", idle.animations[1].file.TryRead());
            Assert.AreEqual(3456u, idle.animations[1].animation_id);
            Assert.AreEqual(0.6f, idle.animations[1].ratio);
            Assert.AreEqual(0.7f, idle.animations[1].weight);

            Assert.AreEqual("", idle.animations[2].file.TryRead());
            Assert.AreEqual(0u, idle.animations[2].animation_id);
            Assert.AreEqual(0f, idle.animations[2].ratio);
            Assert.AreEqual(0f, idle.animations[2].weight);

            Assert.AreEqual(ActionIdleMode.Idle, idle.mode);
            Assert.AreEqual(30u, idle.idle_progress);
            Assert.AreEqual(40u, idle.ready_progress);
            Assert.AreEqual(0u, idle.idle_timer);
            Assert.AreEqual(21u, idle.switch_progress);

            var idle2 = idle.Arc();
            Assert.AreEqual(40u, idle2.ready_progress);

            var weak_idle = idle2.Weak();
            Assert.AreEqual(7744u, weak_idle.event_idx);

            Assert.AreEqual(2, idle.StrongCount);
            idle.Dispose();
            Assert.AreEqual(1, idle2.WeakCount);
            idle2.Dispose();
        }
    }
}

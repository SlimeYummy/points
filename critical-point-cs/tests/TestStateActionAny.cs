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
        public void TestBoxStateActionIdel() {
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
            Assert.AreEqual(0.125f, idle.animations[0].ratio);
            Assert.AreEqual(0.333f, idle.animations[0].weight);

            Assert.AreEqual("mock_action_idle_2", idle.animations[1].files.TryRead());
            Assert.AreEqual(3456u, idle.animations[1].animation_id);
            Assert.AreEqual(0.6f, idle.animations[1].ratio);
            Assert.AreEqual(0.7f, idle.animations[1].weight);

            Assert.AreEqual("", idle.animations[2].files.TryRead());
            Assert.AreEqual(0u, idle.animations[2].animation_id);
            Assert.AreEqual(0f, idle.animations[2].ratio);
            Assert.AreEqual(0f, idle.animations[2].weight);

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
            Assert.AreEqual(0.125f, idle.animations[0].ratio);
            Assert.AreEqual(0.333f, idle.animations[0].weight);

            Assert.AreEqual("mock_action_idle_2", idle.animations[1].files.TryRead());
            Assert.AreEqual(3456u, idle.animations[1].animation_id);
            Assert.AreEqual(0.6f, idle.animations[1].ratio);
            Assert.AreEqual(0.7f, idle.animations[1].weight);

            Assert.AreEqual("", idle.animations[2].files.TryRead());
            Assert.AreEqual(0u, idle.animations[2].animation_id);
            Assert.AreEqual(0f, idle.animations[2].ratio);
            Assert.AreEqual(0f, idle.animations[2].weight);

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
    }
}

using CriticalPoint;
using MessagePack;
using System.Diagnostics;
using System.Drawing;
using System.Runtime.InteropServices;

namespace CriticalPointTests {
    [MessagePackObject(keyAsPropertyName: true)]
    public struct Layout {
        public int size;
        public int align;
    }

    [TestClass]
    public class TestLayout {
        [TestMethod]
        public void TestSizeAndAlign() {
            var rsLayouts = GetRustLayouts();
            var csLayouts = GetCsLayouts();

            foreach (var (name, rsLayout) in rsLayouts) {
                //Debug.WriteLine(name);
                var csLayout = csLayouts[name];
                Assert.AreEqual(rsLayout.size, csLayout.size, name);
                Assert.AreEqual(rsLayout.align, csLayout.align, name);
            }
        }

        [DllImport("critical_point_csbridge.dll")]
        private static extern unsafe UIntPtr get_rust_layouts(byte* buf, UIntPtr len);

        private Dictionary<string, Layout> GetRustLayouts() {
            byte[] buffer = new byte[1024 * 20];
            UIntPtr len = 0;
            unsafe {
                fixed (byte* ptr = buffer) {
                    len = get_rust_layouts(ptr, 1024 * 20);
                }
            }
            var layouts = MessagePackSerializer.Deserialize<Dictionary<string, Layout>>(new ArraySegment<byte>(buffer, 0, (int)len));
            return layouts;
        }

        private Dictionary<string, Layout> GetCsLayouts() {
            var layouts = new Dictionary<string, Layout>();
            layouts["AnimationFileMeta"] = MakeLayout<AnimationFileMeta>(AnimationFileMeta.SIZE, AnimationFileMeta.ALIGN);
            layouts["CustomEvent"] = MakeLayout<CustomEvent>(CustomEvent.SIZE, CustomEvent.ALIGN);
            layouts["LogicEngineStatus"] = MakeLayout<LogicEngineStatus>(LogicEngineStatus.SIZE, LogicEngineStatus.ALIGN);
            layouts["SkeletonJointMeta"] = MakeLayout<RsSkeletonJointMeta>(RsSkeletonJointMeta.SIZE, RsSkeletonJointMeta.ALIGN);
            layouts["SkeletonMeta"] = MakeLayout<RsSkeletonMeta>(RsSkeletonMeta.SIZE, RsSkeletonMeta.ALIGN);
            layouts["StateActionAnimation"] = MakeLayout<StateActionAnimation>(StateActionAnimation.SIZE, StateActionAnimation.ALIGN);
            layouts["StateActionBase"] = MakeLayout<RsStateActionBase>(RsStateActionBase.SIZE, RsStateActionBase.ALIGN);
            layouts["StateActionEmpty"] = MakeLayout<RsStateActionEmpty>(RsStateActionEmpty.SIZE, RsStateActionEmpty.ALIGN);
            layouts["StateActionGeneral"] = MakeLayout<RsStateActionGeneral>(RsStateActionGeneral.SIZE, RsStateActionGeneral.ALIGN);
            layouts["StateActionIdle"] = MakeLayout<RsStateActionIdle>(RsStateActionIdle.SIZE, RsStateActionIdle.ALIGN);
            layouts["StateActionMove"] = MakeLayout<RsStateActionMove>(RsStateActionMove.SIZE, RsStateActionMove.ALIGN);
            layouts["StateBase"] = MakeLayout<RsStateBase>(RsStateBase.SIZE, RsStateBase.ALIGN);
            layouts["StateCharaPhysics"] = MakeLayout<StateCharaPhysics>(StateCharaPhysics.SIZE, StateCharaPhysics.ALIGN);
            layouts["StateGameInit"] = MakeLayout<RsStateGameInit>(RsStateGameInit.SIZE, RsStateGameInit.ALIGN);
            layouts["StateGameUpdate"] = MakeLayout<RsStateGameUpdate>(RsStateGameUpdate.SIZE, RsStateGameUpdate.ALIGN);
            layouts["StateMultiRootMotion"] = MakeLayout<StateMultiRootMotion>(StateMultiRootMotion.SIZE, StateMultiRootMotion.ALIGN);
            layouts["StateNpcInit"] = MakeLayout<RsStateNpcInit>(RsStateNpcInit.SIZE, RsStateNpcInit.ALIGN);
            layouts["StateNpcUpdate"] = MakeLayout<RsStateNpcUpdate>(RsStateNpcUpdate.SIZE, RsStateNpcUpdate.ALIGN);
            layouts["StatePlayerInit"] = MakeLayout<RsStatePlayerInit>(RsStatePlayerInit.SIZE, RsStatePlayerInit.ALIGN);
            layouts["StatePlayerUpdate"] = MakeLayout<RsStatePlayerUpdate>(RsStatePlayerUpdate.SIZE, RsStatePlayerUpdate.ALIGN);
            layouts["StateRootMotion"] = MakeLayout<StateRootMotion>(StateRootMotion.SIZE, StateRootMotion.ALIGN);
            layouts["StateSet"] = MakeLayout<RsStateSet>(RsStateSet.SIZE, RsStateSet.ALIGN);
            layouts["StateZoneInit"] = MakeLayout<RsStateZoneInit>(RsStateZoneInit.SIZE, RsStateZoneInit.ALIGN);
            layouts["StateZoneUpdate"] = MakeLayout<RsStateZoneUpdate>(RsStateZoneUpdate.SIZE, RsStateZoneUpdate.ALIGN);
            layouts["WeaponTransform"] = MakeLayout<WeaponTransform>(WeaponTransform.SIZE, WeaponTransform.ALIGN);
            return layouts;
        }

        private Layout MakeLayout<T>(int size, int align) {
            unsafe {
                Assert.AreEqual(size, sizeof(T));
            }
            return new Layout { size = size, align = align };
        }
    }
}

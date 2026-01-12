use critical_point_core::animation::{AnimationFileMeta, SkeletonJointMeta, SkeletonMeta, WeaponTransform};
use critical_point_core::engine::LogicEngineStatus;
use critical_point_core::logic::{
    StateActionAnimation, StateActionBase, StateActionEmpty, StateActionGeneral, StateActionIdle, StateActionMove,
    StateBase, StateCharaPhysics, StateGameInit, StateGameUpdate, StateMultiRootMotion, StateNpcInit, StateNpcUpdate,
    StatePlayerInit, StatePlayerUpdate, StateRootMotion, StateSet, StateZoneInit, StateZoneUpdate,
};
use critical_point_core::utils::CustomEvent;
use std::collections::HashMap;
use std::mem;
use std::ptr;

#[no_mangle]
pub extern "C" fn get_rust_layouts(buf: *mut u8, len: usize) -> usize {
    let mut layouts = HashMap::new();
    layouts.insert("AnimationFileMeta", Layout::new::<AnimationFileMeta>());
    layouts.insert("CustomEvent", Layout::new::<CustomEvent>());
    layouts.insert("LogicEngineStatus", Layout::new::<LogicEngineStatus>());
    layouts.insert("SkeletonJointMeta", Layout::new::<SkeletonJointMeta>());
    layouts.insert("SkeletonMeta", Layout::new::<SkeletonMeta>());
    layouts.insert("StateActionAnimation", Layout::new::<StateActionAnimation>());
    layouts.insert("StateActionBase", Layout::new::<StateActionBase>());
    layouts.insert("StateActionEmpty", Layout::new::<StateActionEmpty>());
    layouts.insert("StateActionGeneral", Layout::new::<StateActionGeneral>());
    layouts.insert("StateActionIdle", Layout::new::<StateActionIdle>());
    layouts.insert("StateActionMove", Layout::new::<StateActionMove>());
    layouts.insert("StateBase", Layout::new::<StateBase>());
    layouts.insert("StateCharaPhysics", Layout::new::<StateCharaPhysics>());
    layouts.insert("StateGameInit", Layout::new::<StateGameInit>());
    layouts.insert("StateGameUpdate", Layout::new::<StateGameUpdate>());
    layouts.insert("StateMultiRootMotion", Layout::new::<StateMultiRootMotion>());
    layouts.insert("StateNpcInit", Layout::new::<StateNpcInit>());
    layouts.insert("StateNpcUpdate", Layout::new::<StateNpcUpdate>());
    layouts.insert("StatePlayerInit", Layout::new::<StatePlayerInit>());
    layouts.insert("StatePlayerUpdate", Layout::new::<StatePlayerUpdate>());
    layouts.insert("StateRootMotion", Layout::new::<StateRootMotion>());
    layouts.insert("StateSet", Layout::new::<StateSet>());
    layouts.insert("StateZoneInit", Layout::new::<StateZoneInit>());
    layouts.insert("StateZoneUpdate", Layout::new::<StateZoneUpdate>());
    layouts.insert("WeaponTransform", Layout::new::<WeaponTransform>());

    let tmp = rmp_serde::to_vec_named(&layouts).unwrap();
    if len < tmp.len() {
        return 0;
    }
    unsafe { ptr::copy_nonoverlapping(tmp.as_ptr(), buf, tmp.len()) };
    tmp.len()
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Layout {
    size: usize,
    align: usize,
}

impl Layout {
    fn new<T>() -> Layout {
        Layout {
            size: mem::size_of::<T>(),
            align: mem::align_of::<T>(),
        }
    }
}

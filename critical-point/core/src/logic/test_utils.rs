use std::fmt::Debug;
use std::rc::Rc;

use jolt_physics_rs::PhysicsSystem;

use crate::consts::{TEST_ASSET_PATH, TEST_TEMPLATE_PATH};
use crate::instance::{assemble_player, InstPlayer};
use crate::logic::action::{
    ArchivedStateAction, ContextAction, LogicAction, LogicActionBase, StateAction, StateActionBase, StateActionType,
};
use crate::logic::game::{ContextUpdate, LogicSystems};
use crate::logic::physics::{
    BroadPhaseLayerInterfaceImpl, ObjectLayerPairFilterImpl, ObjectVsBroadPhaseLayerFilterImpl,
};
use crate::parameter::ParamPlayer;
use crate::template::{TmplDatabase, TmplType};
use crate::utils::{extend, sb, NumID, StrID, XResult};

pub(crate) fn mock_physics_system() -> PhysicsSystem {
    PhysicsSystem::new(
        BroadPhaseLayerInterfaceImpl::new_vbox(BroadPhaseLayerInterfaceImpl),
        ObjectVsBroadPhaseLayerFilterImpl::new_vbox(ObjectVsBroadPhaseLayerFilterImpl),
        ObjectLayerPairFilterImpl::new_vbox(ObjectLayerPairFilterImpl),
    )
}

pub(crate) fn mock_logic_systems() -> LogicSystems {
    let tmpl_db = TmplDatabase::new(TEST_TEMPLATE_PATH).unwrap();
    LogicSystems::new(tmpl_db, TEST_ASSET_PATH, None).unwrap()
}

pub(crate) fn mock_inst_player(systems: &mut LogicSystems) -> Rc<InstPlayer> {
    let mut ctx = ContextUpdate::new(systems, 0, 0);
    let param_player = ParamPlayer {
        character: sb!("Character.No1"),
        style: sb!("Style.No1-1"),
        level: 4,
        ..Default::default()
    };
    let inst_player = assemble_player(&mut ctx.context_assemble(), &param_player).unwrap();
    Rc::new(inst_player)
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive_attr(derive(Debug))]
pub(crate) struct StateActionEmpty {
    pub _base: StateActionBase,
}

extend!(StateActionEmpty, StateActionBase);

impl Default for StateActionEmpty {
    fn default() -> Self {
        StateActionEmpty {
            _base: StateActionBase::new(StateActionType::Idle, TmplType::ActionIdle),
        }
    }
}

unsafe impl StateAction for StateActionEmpty {
    fn typ(&self) -> StateActionType {
        StateActionType::Idle
    }

    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionIdle
    }
}

#[allow(private_interfaces)]
impl ArchivedStateAction for rkyv::Archived<StateActionEmpty> {
    fn typ(&self) -> StateActionType {
        StateActionType::Idle
    }

    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionIdle
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct LogicActionEmpty {
    _base: LogicActionBase,
}

extend!(LogicActionEmpty, LogicActionBase);

impl LogicActionEmpty {
    pub(crate) fn new(id: NumID) -> Box<LogicActionEmpty> {
        Box::new(LogicActionEmpty {
            _base: LogicActionBase::new(id, StrID::default(), 1),
        })
    }
}

unsafe impl LogicAction for LogicActionEmpty {
    fn typ(&self) -> StateActionType {
        StateActionType::Idle
    }

    fn tmpl_typ(&self) -> TmplType {
        TmplType::ActionIdle
    }

    fn restore(&mut self, _state: &(dyn StateAction + 'static)) -> XResult<()> {
        Ok(())
    }

    fn update(
        &mut self,
        _ctx: &mut ContextUpdate<'_>,
        _ctx_au: &mut ContextAction<'_>,
    ) -> XResult<Option<Box<dyn StateAction>>> {
        Ok(Some(Box::new(StateActionEmpty {
            _base: StateActionBase::new(StateActionType::Idle, TmplType::ActionIdle),
        })))
    }
}

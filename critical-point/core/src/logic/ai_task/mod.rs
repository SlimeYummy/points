mod base;
mod idle;
mod patrol;

pub use base::*;
pub use idle::*;
pub use patrol::*;

use std::rc::Rc;

use crate::instance::{InstAiTaskAny, InstCharacter};
use crate::logic::game::ContextUpdate;
use crate::utils::{AiTaskType, Castable, XResult, xres};

pub(crate) fn new_logic_ai_task(
    ctx: &mut ContextUpdate,
    inst_task: Rc<dyn InstAiTaskAny>,
    inst_chara: Rc<InstCharacter>,
) -> XResult<Box<dyn LogicAiTaskAny>> {
    use AiTaskType::*;

    let logic_task: Box<dyn LogicAiTaskAny> = match inst_task.typ() {
        Idle => {
            let inst_task = unsafe { inst_task.cast_unchecked() };
            Box::new(LogicAiTaskIdle::new(ctx, inst_task, inst_chara)?)
        }
        Patrol => {
            let inst_task = unsafe { inst_task.cast_unchecked() };
            Box::new(LogicAiTaskPatrol::new(ctx, inst_task, inst_chara)?)
        }
        _ => return xres!(BadType),
    };
    Ok(logic_task)
}

pub(crate) fn try_reuse_logic_aitask(
    logic_task: &mut Box<dyn LogicAiTaskAny>,
    ctx: &mut ContextUpdate,
    inst_task: Rc<dyn InstAiTaskAny>,
    inst_chara: Rc<InstCharacter>,
) -> XResult<bool> {
    use AiTaskType::*;

    match inst_task.typ() {
        Idle => {
            if let Ok(logic_task) = logic_task.cast::<LogicAiTaskIdle>() {
                let inst_task = unsafe { inst_task.cast_unchecked() };
                *logic_task = LogicAiTaskIdle::new(ctx, inst_task, inst_chara)?;
                return Ok(true);
            }
        }
        Patrol => {
            if let Ok(logic_task) = logic_task.cast::<LogicAiTaskPatrol>() {
                let inst_task = unsafe { inst_task.cast_unchecked() };
                *logic_task = LogicAiTaskPatrol::new(ctx, inst_task, inst_chara)?;
                return Ok(true);
            }
        }
        _ => return Ok(false),
    }
    Ok(false)
}

mod base;
mod general;
mod idle;
mod move_to_character;
mod patrol;
mod sequence;

pub use base::*;
pub use general::*;
pub use idle::*;
pub use move_to_character::*;
pub use patrol::*;
pub use sequence::*;

use std::rc::Rc;

use crate::template::{At, TmplAny, TmplType};
use crate::utils::{XResult, xres};

pub(crate) fn assemble_ai_task(tmpl: At<dyn TmplAny>) -> XResult<Rc<dyn InstAiTaskAny>> {
    let task: Rc<dyn InstAiTaskAny> = match tmpl.typ() {
        TmplType::AiTaskPatrol => {
            let inst = InstAiTaskPatrol::new(unsafe { tmpl.cast_unchecked() });
            Rc::new(inst)
        }
        TmplType::AiTaskIdle => {
            let inst = InstAiTaskIdle::new(unsafe { tmpl.cast_unchecked() });
            Rc::new(inst)
        }
        TmplType::AiTaskMoveToCharacter => {
            let inst = InstAiTaskMoveToCharacter::new(unsafe { tmpl.cast_unchecked() });
            Rc::new(inst)
        }
        TmplType::AiTaskGeneral => {
            let inst = InstAiTaskGeneral::new(unsafe { tmpl.cast_unchecked() });
            Rc::new(inst)
        }
        TmplType::AiTaskSequence => {
            let inst = InstAiTaskSequence::new(unsafe { tmpl.cast_unchecked() });
            Rc::new(inst)
        }
        _ => return xres!(BadType),
    };

    Ok(task)
}

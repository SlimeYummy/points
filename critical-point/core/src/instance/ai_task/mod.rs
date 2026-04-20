mod base;
mod idle;
mod patrol;

pub use base::*;
pub use idle::*;
pub use patrol::*;

use std::rc::Rc;

use crate::template::{At, TmplAny, TmplType};
use crate::utils::{XResult, xres};

pub(crate) fn assemble_ai_task(tmpl: At<dyn TmplAny>) -> XResult<Rc<dyn InstAiTaskAny>> {
    let task: Rc<dyn InstAiTaskAny> = match tmpl.typ() {
        TmplType::AiTaskIdle => {
            let inst = InstAiTaskIdle::new(unsafe { tmpl.cast_unchecked() });
            Rc::new(inst)
        }
        TmplType::AiTaskPatrol => {
            let inst = InstAiTaskPatrol::new(unsafe { tmpl.cast_unchecked() });
            Rc::new(inst)
        }
        _ => return xres!(BadType),
    };

    Ok(task)
}

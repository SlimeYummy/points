use std::any::Any;
use std::fmt::Debug;

use crate::template::TmplRepeatLimit;
use crate::utils::{AiTaskType, TmplID, interface};

pub type InstRepeatLimit = TmplRepeatLimit;

pub unsafe trait InstAiTaskAny: Debug + Any {
    fn typ(&self) -> AiTaskType;
    fn actions(&self, actions: &mut Vec<TmplID>);
}

#[derive(Default, Debug)]
pub struct InstAiTaskBase {
    pub tmpl_id: TmplID,
}

interface!(InstAiTaskAny, InstAiTaskBase);

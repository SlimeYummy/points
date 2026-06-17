use critical_point_macros::wasm_struct;
use std::path::Path;
use std::rc::Rc;
use talc::TalcCell;
use wasmtime::TypedFunc;

use crate::logic::ai_task::WsAiTask;
use crate::logic::character::WsCharaValue;
use crate::script::{ScriptEngine, ScriptEngineConfig, TalcSource, WsBox, WsVec};
use crate::utils::{TmplID, XResult};

pub(crate) struct LogicScriptEngine {
    engine: ScriptEngine,
    global: WsBox<WsGameGlobal>,
    tmp_ai_tasks: WsVec<WsAiTask>,
}

impl LogicScriptEngine {
    pub(crate) fn new<P: AsRef<Path>>(wasm_path: P, config: ScriptEngineConfig) -> XResult<Self> {
        let engine = ScriptEngine::new(wasm_path, config)?;
        let global = WsBox::new_in(WsGameGlobal::default(), engine.alloc());
        let tmp_ai_tasks = WsVec::with_capacity_in(128, engine.alloc());
        Ok(Self {
            engine,
            global,
            tmp_ai_tasks,
        })
    }

    #[inline]
    pub(crate) fn alloc(&self) -> Rc<TalcCell<TalcSource>> {
        self.engine.alloc()
    }

    #[inline]
    pub(crate) fn global(&self) -> &WsBox<WsGameGlobal> {
        &self.global
    }

    #[inline]
    pub(crate) fn global_mut(&mut self) -> &mut WsBox<WsGameGlobal> {
        &mut self.global
    }

    #[inline]
    pub(crate) fn get_ai_brain_execute(&mut self, id: TmplID) -> XResult<WsFuncAiBrainExecute> {
        let func_name = id.make_func_name("execute")?;
        self.engine
            .get_typed_func::<WsArgsAiBrainExecute, WsRetsAiBrainExecute>(&func_name)
    }

    #[inline]
    pub(crate) fn call_ai_brain_execute<'t>(
        &'t mut self,
        func: WsFuncAiBrainExecute,
        chara_value: &WsBox<WsCharaValue>,
    ) -> XResult<&'t [WsAiTask]> {
        self.tmp_ai_tasks.clear();
        let res = self.engine.call(
            func,
            (
                self.engine.to_wasm_addr(&self.global),
                self.engine.to_wasm_addr(chara_value),
                self.engine.to_wasm_addr(&self.tmp_ai_tasks),
                self.tmp_ai_tasks.capacity() as u32,
            ),
        )?;

        let ctx = self.engine.store().data();
        let (error, tmpl_ids_len) = ctx.unpack(res);
        log::debug!(
            "LogicScriptEngine::call_ai_brain_execute() chara_id={} => ({}, {})",
            chara_value.chara_id,
            error,
            tmpl_ids_len
        );
        ctx.read_result(error)?;

        unsafe { self.tmp_ai_tasks.set_len(tmpl_ids_len as usize) };
        Ok(self.tmp_ai_tasks.as_slice())
    }
}

#[repr(C)]
#[wasm_struct(8, 4)]
#[derive(Debug, Default)]
pub(crate) struct WsGameGlobal {
    pub frame: u32,
    pub time: f32,
}

/// ```
/// fn(
///     global_ptr: *const WsGameGlobal,
///     chara_value_ptr: *const WsCharaValue,
///     ai_tasks_ptr: *mut WsAiTask,
///     ai_tasks_len: u32
/// ) -> (error: u32, ai_tasks_len: u32)
/// ```
pub(crate) type WsFuncAiBrainExecute = TypedFunc<WsArgsAiBrainExecute, WsRetsAiBrainExecute>;
pub(crate) type WsArgsAiBrainExecute = (u32, u32, u32, u32);
pub(crate) type WsRetsAiBrainExecute = u64;

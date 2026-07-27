use critical_point_macros::wasm_struct;
use std::path::Path;
use std::rc::Rc;
use talc::TalcCell;
use wasmtime::TypedFunc;

use crate::logic::ai_task::WsAiDo;
use crate::logic::character::{WsCharaControl, WsCharaPhysics, WsCharaValue};
use crate::logic::game::GameTime;
use crate::script::{ScriptEngine, ScriptEngineConfig, TalcSource, WsBox, WsVec};
use crate::utils::{TmplID, XResult};

pub(crate) struct LogicScriptEngine {
    engine: ScriptEngine,
    global: WsBox<WsGameGlobal>,
}

impl LogicScriptEngine {
    pub(crate) fn new<P: AsRef<Path>>(wasm_path: P, config: ScriptEngineConfig) -> XResult<Self> {
        let engine = ScriptEngine::new(wasm_path, config)?;
        let global = WsBox::new_in(WsGameGlobal::default(), engine.alloc());
        Ok(Self { engine, global })
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
    pub(crate) fn update_global(&mut self, time: &GameTime) {
        self.global.frame = time.frame;
        self.global.time = time.time;
    }

    #[inline]
    pub(crate) fn get_ai_brain_execute(&mut self, id: TmplID) -> XResult<WsFuncAiBrainExecute> {
        let func_name = id.make_func_name("execute", None)?;
        self.engine
            .get_typed_func::<WsArgsAiBrainExecute, WsRetsAiBrainExecute>(&func_name)
    }

    /// The parameter `do_list`'s capacity > 0 and will be cleared before use.
    #[inline]
    pub(crate) fn call_ai_brain_execute<'t>(
        &'t mut self,
        func: WsFuncAiBrainExecute,
        chara_ctrl: &WsBox<WsCharaControl>,
        chara_phy: &WsBox<WsCharaPhysics>,
        chara_val: &WsBox<WsCharaValue>,
        tgt_phy: Option<&WsBox<WsCharaPhysics>>,
        tgt_val: Option<&WsBox<WsCharaValue>>,
        do_list: &mut WsVec<WsAiDo>,
    ) -> XResult<()> {
        do_list.clear();
        debug_assert!(do_list.capacity() > 0);

        let res = self.engine.call(
            func,
            (
                self.engine.to_wasm_addr(&self.global),
                self.engine.to_wasm_addr(chara_ctrl),
                self.engine.to_wasm_addr(chara_phy),
                self.engine.to_wasm_addr(chara_val),
                self.engine.to_wasm_addr_opt(tgt_phy),
                self.engine.to_wasm_addr_opt(tgt_val),
                self.engine.to_wasm_addr(do_list),
                do_list.capacity() as u32,
            ),
        )?;

        let ctx = self.engine.store().data();
        let (error, tmpl_ids_len) = ctx.unpack(res);
        log::debug!(
            "LogicScriptEngine::call_ai_brain_execute() chara_id={} => ({}, {})",
            chara_val.chara_id,
            error,
            tmpl_ids_len
        );
        ctx.read_result(error)?;

        unsafe { do_list.set_len(tmpl_ids_len as usize) };
        Ok(())
    }

    #[inline]
    pub(crate) fn get_ai_routine_if(&mut self, id: TmplID, func_no: u16) -> XResult<WsFuncAiRoutineIf> {
        let func_name = id.make_func_name("if", Some(func_no))?;
        self.engine
            .get_typed_func::<WsArgsAiRoutineIf, WsRetsAiRoutineIf>(&func_name)
    }

    #[inline]
    pub(crate) fn call_ai_routine_if(
        &mut self,
        func: WsFuncAiRoutineIf,
        chara_ctrl: &WsBox<WsCharaControl>,
        chara_phy: &WsBox<WsCharaPhysics>,
        chara_val: &WsBox<WsCharaValue>,
        tgt_phy: Option<&WsBox<WsCharaPhysics>>,
        tgt_val: Option<&WsBox<WsCharaValue>>,
    ) -> XResult<bool> {
        let res = self.engine.call(
            func,
            (
                self.engine.to_wasm_addr(&self.global),
                self.engine.to_wasm_addr(chara_ctrl),
                self.engine.to_wasm_addr(chara_phy),
                self.engine.to_wasm_addr(chara_val),
                self.engine.to_wasm_addr_opt(tgt_phy),
                self.engine.to_wasm_addr_opt(tgt_val),
            ),
        )?;

        let ctx = self.engine.store().data();
        let (error, result) = ctx.unpack(res);
        ctx.read_result(error)?;

        Ok(result != 0)
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
///     chara_ctrl_ptr: *const WsCharaControl,
///     chara_phy_ptr: *const WsCharaPhysics,
///     chara_val_ptr: *const WsCharaValue,
///     tgt_phy_ptr: *const WsCharaPhysics, // nullable
///     tgt_val_ptr: *const WsCharaValue, // nullable
///     do_list_ptr: *mut WsAiDo,
///     do_list_len: u32,
/// ) -> (error: u32, do_list_len: u32)
/// ```
pub(crate) type WsFuncAiBrainExecute = TypedFunc<WsArgsAiBrainExecute, WsRetsAiBrainExecute>;
pub(crate) type WsArgsAiBrainExecute = (u32, u32, u32, u32, u32, u32, u32, u32);
pub(crate) type WsRetsAiBrainExecute = u64;

/// ```
/// fn(
///     global_ptr: *const WsGameGlobal,
///     chara_ctrl_ptr: *const WsCharaControl,
///     chara_phy_ptr: *const WsCharaPhysics,
///     chara_val_ptr: *const WsCharaValue,
///     tgt_phy_ptr: *const WsCharaPhysics, // nullable
///     tgt_val_ptr: *const WsCharaValue, // nullable
/// ) -> bool
/// ```
pub(crate) type WsFuncAiRoutineIf = TypedFunc<WsArgsAiRoutineIf, WsRetsAiRoutineIf>;
pub(crate) type WsArgsAiRoutineIf = (u32, u32, u32, u32, u32, u32);
pub(crate) type WsRetsAiRoutineIf = u64;

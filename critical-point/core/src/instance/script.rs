use std::collections::HashMap;
use std::{mem, slice};

use crate::instance::values::{ExtraValues, PanelValues, PrimaryValues, SecondaryValues};
use crate::script::{ScriptEnv, ScriptInputMap, ScriptOutputMap, SEGMENT_IN_MIN, SEGMENT_OUT_MIN};
use crate::template::{TmplIsPlus, TmplScript};
use crate::utils::{KvList, Num, Symbol, SymbolMap, Table, XError, XResult};

#[derive(Debug)]
pub struct InstScript {
    pub script: TmplScript,
    pub closure: Vec<Num>,
}

impl InstScript {
    pub fn new_kvlist(script: &TmplScript, list: &KvList<Symbol, Num>) -> XResult<InstScript> {
        if list.len() != script.arguments.len() {
            return Err(XError::bad_script("argument count"));
        }

        let cap = script.arguments.len() + script.closure_inits.len();
        let mut closure = Vec::with_capacity(cap);

        unsafe {
            closure.set_len(script.arguments.len());
        }
        Self::fill_kvlist_arguments(script, &mut closure[0..script.arguments.len()], &list)?;

        closure.extend_from_slice(&script.closure_inits);

        Ok(InstScript {
            script: script.clone(),
            closure,
        })
    }

    pub fn new_table(script: &TmplScript, norm_level: u32, table: &Table<Symbol, Num>) -> XResult<InstScript> {
        if table.len() != script.arguments.len() {
            return Err(XError::bad_script("argument count"));
        }

        let cap = script.arguments.len() + script.closure_inits.len();
        let mut closure = Vec::with_capacity(cap);

        unsafe {
            closure.set_len(script.arguments.len());
        }
        Self::fill_table_arguments(script, &mut closure[0..script.arguments.len()], norm_level, &table)?;

        closure.extend_from_slice(&script.closure_inits);

        Ok(InstScript {
            script: script.clone(),
            closure,
        })
    }

    pub fn new_table_plus(
        script: &TmplScript,
        piece: u32,
        plus: u32,
        arguments: &Table<(Symbol, TmplIsPlus), Num>,
    ) -> XResult<InstScript> {
        let cap = script.arguments.len() + script.closure_inits.len();
        let mut closure = Vec::with_capacity(cap);

        unsafe {
            closure.set_len(script.arguments.len());
        }
        Self::fill_table_arguments_plus(script, &mut closure[0..script.arguments.len()], piece, plus, &arguments)?;

        closure.extend_from_slice(&script.closure_inits);

        Ok(InstScript {
            script: script.clone(),
            closure,
        })
    }

    fn fill_kvlist_arguments(script: &TmplScript, closure: &mut [Num], list: &&KvList<Symbol, Num>) -> XResult<()> {
        'out: for (offset, (argument, value)) in list.iter().enumerate() {
            for idx in 0..script.arguments.len() {
                let pos = (offset + idx) % script.arguments.len();
                if script.arguments[pos] == *argument {
                    closure[pos] = *value;
                    continue 'out;
                }
            }
            return Err(XError::bad_script(format!("invalid argument \"{}\"", argument)));
        }
        Ok(())
    }

    fn fill_table_arguments(
        script: &TmplScript,
        closure: &mut [Num],
        norm_level: u32,
        table: &&Table<Symbol, Num>,
    ) -> XResult<()> {
        'out: for (offset, (argument, values)) in table.iter().enumerate() {
            for idx in 0..script.arguments.len() {
                let pos = (offset + idx) % script.arguments.len();
                if script.arguments[pos] == *argument {
                    closure[pos] = values[norm_level as usize];
                    continue 'out;
                }
            }
            return Err(XError::bad_script(format!("invalid argument \"{}\"", argument)));
        }
        Ok(())
    }

    fn fill_table_arguments_plus(
        script: &TmplScript,
        closure: &mut [Num],
        piece: u32,
        plus: u32,
        table: &&Table<(Symbol, TmplIsPlus), Num>,
    ) -> XResult<()> {
        'out: for (offset, ((argument, is_plus), values)) in table.iter().enumerate() {
            let pp = if *is_plus { plus } else { piece };
            for idx in 0..script.arguments.len() {
                let pos = (offset + idx) % script.arguments.len();
                if script.arguments[pos] == *argument {
                    closure[pos] = values[pp as usize];
                    continue 'out;
                }
            }
            return Err(XError::bad_script(format!("invalid argument \"{}\"", argument)));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct OnAssembleEnv<'e> {
    pub(crate) closure: &'e mut [Num],
    pub(crate) primary: &'e PrimaryValues,
    pub(crate) secondary: &'e mut SecondaryValues,
    pub(crate) global: &'e mut SymbolMap<Num>,
}

impl<'e> OnAssembleEnv<'e> {
    pub fn script_inputs() -> HashMap<u8, ScriptInputMap> {
        HashMap::from([(SEGMENT_IN_MIN, PrimaryValues::script_input())])
    }

    pub fn script_outputs() -> HashMap<u8, ScriptOutputMap> {
        HashMap::from([(SEGMENT_OUT_MIN, SecondaryValues::script_output())])
    }
}

impl<'e> ScriptEnv<1, 1> for OnAssembleEnv<'e> {
    fn closure_segment(&mut self) -> &mut [u64] {
        unsafe { mem::transmute_copy::<&mut [Num], &mut [u64]>(&self.closure) }
    }

    fn in_segments(&self) -> [&[u64]; 1] {
        return [unsafe {
            slice::from_raw_parts(
                self.primary as *const _ as *const u64,
                mem::size_of::<PrimaryValues>() / mem::size_of::<u64>(),
            )
        }];
    }

    fn out_segments(&mut self) -> [&mut [u64]; 1] {
        return [unsafe {
            slice::from_raw_parts_mut(
                self.secondary as *mut _ as *mut u64,
                mem::size_of::<SecondaryValues>() / mem::size_of::<u64>(),
            )
        }];
    }

    fn global(&mut self) -> &mut SymbolMap<Num> {
        self.global
    }
}

#[derive(Debug)]
pub struct AfterAssembleEnv<'e> {
    pub(crate) closure: &'e mut [Num],
    pub(crate) panel: &'e PanelValues,
    pub(crate) extra: &'e mut ExtraValues,
    pub(crate) global: &'e mut SymbolMap<Num>,
}

impl<'e> AfterAssembleEnv<'e> {
    pub fn script_inputs() -> HashMap<u8, ScriptInputMap> {
        HashMap::from([(SEGMENT_IN_MIN, PanelValues::script_input())])
    }

    pub fn script_outputs() -> HashMap<u8, ScriptOutputMap> {
        HashMap::from([(SEGMENT_OUT_MIN, ExtraValues::script_output())])
    }
}

impl<'e> ScriptEnv<1, 1> for AfterAssembleEnv<'e> {
    fn closure_segment(&mut self) -> &mut [u64] {
        unsafe { mem::transmute_copy::<&mut [Num], &mut [u64]>(&self.closure) }
    }

    fn in_segments(&self) -> [&[u64]; 1] {
        return [unsafe {
            slice::from_raw_parts(
                self.panel as *const _ as *const u64,
                mem::size_of::<PanelValues>() / mem::size_of::<u64>(),
            )
        }];
    }

    fn out_segments(&mut self) -> [&mut [u64]; 1] {
        return [unsafe {
            slice::from_raw_parts_mut(
                self.extra as *mut _ as *mut u64,
                mem::size_of::<ExtraValues>() / mem::size_of::<u64>(),
            )
        }];
    }

    fn global(&mut self) -> &mut SymbolMap<Num> {
        self.global
    }
}

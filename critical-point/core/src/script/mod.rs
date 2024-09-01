mod ast;
mod builtin;
mod command;
mod config;
mod generator;
mod parser;
mod runner;
#[allow(dead_code)]
mod test;
mod utils;

use anyhow::Result;
pub use command::*;
pub use config::*;
use generator::ScriptGenerator;
use parser::ScriptParser;
pub use parser::{ScriptInputMap, ScriptOutputMap};
pub use runner::{ScriptEnv, ScriptRunner};
use std::collections::HashMap;
pub use utils::*;

use crate::utils::Num;

pub struct ScriptCompiler {
    parser: ScriptParser,
    generator: ScriptGenerator,
}

impl ScriptCompiler {
    pub fn new(
        global_consts: &HashMap<String, Num>,
        all_inputs: &HashMap<(ScriptBlockType, u8), HashMap<String, u16>>,
        all_outputs: &HashMap<(ScriptBlockType, u8), HashMap<String, (u16, ScriptOutType)>>,
        all_funcs: &HashMap<ScriptBlockType, HashMap<String, (u16, Vec<CmdType>)>>,
    ) -> Result<ScriptCompiler> {
        return Ok(ScriptCompiler {
            parser: ScriptParser::new(global_consts, all_inputs, all_outputs, all_funcs)?,
            generator: ScriptGenerator::new(),
        });
    }

    pub fn compile(&mut self, code: &str, args: &[&str]) -> Result<ScriptBlocks> {
        let tmp = self.parser.run(code, args)?;
        let blocks = self.generator.run(tmp)?;
        return Ok(blocks);
    }
}

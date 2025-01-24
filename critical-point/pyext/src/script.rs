use std::collections::HashMap;

use cirtical_point_core::instance::{AfterAssembleEnv, OnAssembleEnv};
use cirtical_point_core::script::{ScriptBlockType, ScriptCompiler, ScriptInputMap, ScriptOutputMap};
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};

static mut COMPILER: Option<ScriptCompiler> = None;

pub fn new_compiler() -> PyResult<ScriptCompiler> {
    use ScriptBlockType::*;

    let mut inputs = HashMap::new();
    let mut outputs = HashMap::new();
    let funcs = HashMap::new();

    // OnAssemble
    append_inputs(&mut inputs, OnAssemble, OnAssembleEnv::script_inputs());
    append_outputs(&mut outputs, OnAssemble, OnAssembleEnv::script_outputs());

    // AfterAssemble
    append_inputs(&mut inputs, AfterAssemble, AfterAssembleEnv::script_inputs());
    append_outputs(&mut outputs, AfterAssemble, AfterAssembleEnv::script_outputs());

    ScriptCompiler::new(&HashMap::new(), &inputs, &outputs, &funcs)
        .map_err(|err| PyException::new_err(format!("Init compiler error: {}", err)))
}

fn append_inputs(
    inputs: &mut HashMap<(ScriptBlockType, u8), ScriptInputMap>,
    block_type: ScriptBlockType,
    env_inputs: HashMap<u8, ScriptInputMap>,
) {
    for (key, value) in env_inputs {
        inputs.insert((block_type, key), value);
    }
}

fn append_outputs(
    outputs: &mut HashMap<(ScriptBlockType, u8), ScriptOutputMap>,
    block_type: ScriptBlockType,
    env_outputs: HashMap<u8, ScriptOutputMap>,
) {
    for (key, value) in env_outputs {
        outputs.insert((block_type, key), value);
    }
}

#[pyfunction]
pub fn compile_script<'py>(py: Python<'py>, py_code: &PyString, py_args: &PyList) -> PyResult<&'py PyDict> {
    if unsafe { COMPILER.is_none() } {
        let c = new_compiler().map_err(|err| PyException::new_err(format!("Init compiler error: {}", err)))?;
        unsafe { COMPILER = Some(c) };
    }
    let compiler = unsafe { COMPILER.as_mut().unwrap() };

    let code = py_code.to_str()?;
    let mut args = Vec::with_capacity(py_args.len());
    for arg in py_args.iter() {
        args.push(arg.downcast::<PyString>()?.to_str()?);
    }

    let blocks = compiler
        .compile(code, &args)
        .map_err(|err| PyException::new_err(format!("Compile code error: {}", err)))?;

    let res = PyDict::new(py);
    res.set_item(
        "blocks",
        blocks
            .blocks()
            .iter()
            .map(|b| {
                let d = PyDict::new(py);
                if b.is_hook() {
                    d.set_item("type", format!("{:?}", b.typ()))?;
                    d.set_item("code", b.code().to_base64())?;
                } else {
                    d.set_item("type", format!("{:?}", b.typ()))?;
                    d.set_item("arg", b.arg())?;
                    d.set_item("code", b.code().to_base64())?;
                }
                Ok(d)
            })
            .collect::<PyResult<Vec<_>>>()?,
    )?;
    res.set_item("hook_indexes", blocks.hook_indexes())?;
    res.set_item("timer_start", blocks.timer_start())?;
    res.set_item("constant_segment", blocks.constant_segment())?;
    res.set_item(
        "string_segment",
        blocks
            .string_segment()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>(),
    )?;
    res.set_item(
        "arguments",
        blocks.arguments().iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
    )?;
    res.set_item("closure_inits", blocks.closure_inits())?;

    Ok(res)
}

mod ozz_animation;
mod script;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn critical_point_pyext(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(script::compile_script, m)?)?;
    m.add_function(wrap_pyfunction!(ozz_animation::read_skeleton_meta, m)?)?;
    m.add_function(wrap_pyfunction!(ozz_animation::read_animation_meta, m)?)?;
    Ok(())
}

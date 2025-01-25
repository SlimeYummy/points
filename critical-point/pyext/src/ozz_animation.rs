use ozz_animation_rs::{Animation, Archive, Skeleton};
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyfunction]
#[pyo3(signature = (path, with_joints=false))]
pub fn read_skeleton_meta<'py>(py: Python<'py>, path: &str, with_joints: bool) -> PyResult<&'py PyDict> {
    let mut archive =
        Archive::from_path(path).map_err(|err| PyException::new_err(format!("Read archive error: {}", err)))?;
    let meta = Skeleton::read_meta(&mut archive, with_joints)
        .map_err(|err| PyException::new_err(format!("Read skeleton meta error: {}", err)))?;

    let res = PyDict::new(py);
    res.set_item("version", meta.version)?;
    res.set_item("num_joints", meta.num_joints)?;
    if with_joints {
        let names = PyDict::new(py);
        for (i, joint) in meta.joint_names.iter() {
            names.set_item(i, joint)?;
        }
        res.set_item("joint_names", names)?;
        res.set_item("joint_parents", meta.joint_parents)?;
    }
    Ok(res)
}

#[pyfunction]
pub fn read_animation_meta<'py>(py: Python<'py>, path: &str) -> PyResult<&'py PyDict> {
    let mut archive =
        Archive::from_path(path).map_err(|err| PyException::new_err(format!("Read archive error: {}", err)))?;
    let meta = Animation::read_meta(&mut archive)
        .map_err(|err| PyException::new_err(format!("Read animation meta error: {}", err)))?;

    let res = PyDict::new(py);
    res.set_item("version", meta.version)?;
    res.set_item("duration", meta.duration)?;
    res.set_item("num_tracks", meta.num_tracks)?;
    res.set_item("name", meta.name)?;
    res.set_item("translations_count", meta.translations_count)?;
    res.set_item("rotations_count", meta.rotations_count)?;
    res.set_item("scales_count", meta.scales_count)?;
    Ok(res)
}

use std::path::PathBuf;
use std::{env, fs};

pub(super) fn prepare_tmp_dir(name: &str) -> PathBuf {
    let mut dir = env::current_dir().unwrap();
    dir.pop();
    dir.pop();
    dir.push("test-tmp");
    dir.push(name);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::create_dir(&dir);
    dir
}

pub(super) fn write_json<T: ?Sized + serde::Serialize>(path: &PathBuf, data: &T) {
    let buf = serde_json::to_vec(data).unwrap();
    fs::write(&path, buf).unwrap();
}

pub(super) fn write_rkyv<T>(path: &PathBuf, data: &T)
where
    T: rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<2048>>,
{
    let buf = rkyv::to_bytes::<T, 2048>(&data).unwrap();
    fs::write(&path, buf).unwrap();
}

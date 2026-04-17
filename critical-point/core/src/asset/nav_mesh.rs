use recastnavigation_rs::demo::load_nav_mesh;
use recastnavigation_rs::detour::DtNavMesh;

use crate::asset::loader::AssetLoader;
use crate::utils::{Symbol, XResult, xerrf, xfromf};

impl AssetLoader {
    pub fn load_nav_mesh(&mut self, path_pattern: Symbol) -> XResult<DtNavMesh> {
        let path = self.make_full_path(&format!("{}.nm-bin", &path_pattern[0..path_pattern.len() - 2]));
        let path_str = path.to_str().ok_or(xerrf!(NotFound; "path={:?}", &path))?;
        let nav_mesh = load_nav_mesh(path_str).map_err(xfromf!("path={:?}", &path))?;
        Ok(nav_mesh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TEST_ASSET_PATH;
    use crate::utils::sb;

    #[test]
    fn test_load_nav_mesh() {
        let mut loader = AssetLoader::new(TEST_ASSET_PATH).unwrap();
        let nav_mesh = loader.load_nav_mesh(sb!("Zones/Demo1.*")).unwrap();
    }
}

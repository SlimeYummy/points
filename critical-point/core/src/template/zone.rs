use crate::template::base::impl_tmpl;
use crate::utils::TmplID;

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplZone {
    pub id: TmplID,
    pub name: String,
    pub zone_file: String,
    pub view_zone_file: String,
}

impl_tmpl!(TmplZone, Zone, "Zone");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_load_zone() {
        let db = TmplDatabase::new(10240, 150).unwrap();
        let zone = db.find_as::<TmplZone>(id!("Zone.Demo")).unwrap();
        assert_eq!(zone.id, id!("Zone.Demo"));
        assert_eq!(zone.name, "Demo");
        assert_eq!(zone.zone_file, "stage-demo.json");
        assert_eq!(zone.view_zone_file, "stage-demo.tscn");
    }
}

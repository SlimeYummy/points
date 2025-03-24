use crate::template2::base::{ArchivedTmplAny, TmplAny, TmplType};
use crate::template2::TmplID;

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive_attr(derive(Debug))]
pub struct TmplZone {
    pub id: TmplID,
    pub name: String,
    pub zone_file: String,
    pub view_zone_file: String,
}

#[typetag::deserialize(name = "Zone")]
impl TmplAny for TmplZone {
    #[inline]
    fn id(&self) -> TmplID {
        self.id.clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::Zone
    }
}

impl ArchivedTmplAny for ArchivedTmplZone {
    #[inline]
    fn id(&self) -> TmplID {
        TmplID::from(self.id).clone()
    }

    #[inline]
    fn typ(&self) -> TmplType {
        TmplType::Zone
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template2::database::TmplDatabase;
    use crate::template2::id::id;

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

use crate::template::attribute::TmplAttributeType;
use crate::template::base::{TmplAny, TmplClass};
use crate::template::entry::TmplEntryPair;
use crate::template::script::TmplScript;
use crate::template::slot::TmplSlotValue;
use crate::utils::{IDLevel, IDSymbol, KvList, List, Num, StrID, Symbol};

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplPerk {
    pub id: StrID,
    pub name: String,
    pub icon: String,
    pub style: StrID,
    #[serde(default)]
    pub usable_styles: List<StrID>,
    #[serde(default)]
    pub parents: List<IDLevel>,
    #[serde(default)]
    pub attributes: KvList<TmplAttributeType, Num>,
    #[serde(default)]
    pub slot: Option<TmplSlotValue>,
    #[serde(default)]
    pub entries: KvList<StrID, TmplEntryPair>,
    #[serde(default)]
    pub action_args: KvList<IDSymbol, u32>,
    #[serde(default)]
    pub script: Option<TmplScript>,
    #[serde(default)]
    pub script_args: KvList<Symbol, Num>,
}

#[typetag::deserialize(name = "Perk")]
impl TmplAny for TmplPerk {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn class(&self) -> TmplClass {
        TmplClass::Perk
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::s;

    #[test]
    fn test_load_perk() {
        let db = TmplDatabase::new("../test_res").unwrap();

        let perk = db.find_as::<TmplPerk>(&s!("Perk.No1.AttackUp")).unwrap();
        assert_eq!(perk.id, s!("Perk.No1.AttackUp"));
        assert_eq!(perk.name, "AttackUp");
        assert_eq!(perk.icon, "icon");
        assert_eq!(perk.style, s!("Style.No1-1"));
        assert_eq!(perk.usable_styles.as_slice(), &[s!("Style.No1-2")]);

        assert_eq!(
            perk.attributes
                .key_iter().copied()
                .collect::<Vec<TmplAttributeType>>(),
            &[TmplAttributeType::AttackUp,]
        );
        assert_eq!(perk.attributes.value_iter().copied().collect::<Vec<f64>>(), &[0.1]);

        assert!(perk.slot.is_none());
        assert!(perk.entries.is_empty());
        assert!(perk.script.is_some());

        assert_eq!(
            perk.script_args.key_iter().cloned().collect::<Vec<Symbol>>(),
            &[s!("physical_attack"), s!("elemental_attack"), s!("arcane_attack"),]
        );
        assert_eq!(
            perk.script_args.value_iter().copied().collect::<Vec<f64>>(),
            &[2.0, 2.0, 2.0]
        );
    }
}

use super::base::*;
use crate::template::base::{TmplAny, TmplClass, TmplLevelRange, TmplSwitch};
use crate::utils::{KeyCode, KvList, List, StrID, Symbol, Table};

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum TmplActionGeneralAttribute {
    EnterLevel,
    DeriveLevel,
    JustStart,
    JustDuration,
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplActionGeneral {
    pub id: StrID,
    #[serde(default)]
    pub arguments: KvList<Symbol, TmplLevelRange>,
    pub enabled: TmplSwitch,
    pub enter_key: Option<KeyCode>,
    pub enter_level: u16,
    pub base_derive_level: u16,
    pub derive_level: u16,
    pub derive_start: u32,
    pub derive_duration: u32,
    #[serde(default)]
    pub derives: List<TmplDeriveAction>,
    pub just_enabled: TmplSwitch,
    #[serde(default)]
    pub just_start: u32,
    #[serde(default)]
    pub just_duration: u32,
    #[serde(default)]
    pub just_hit: StrID,
    pub insertion_enabled: TmplSwitch,
    #[serde(default)]
    pub insertion_actions: u64,
    #[serde(default)]
    pub insertion_derive_duration: u32,
    #[serde(default)]
    pub attributes: Table<(Symbol, TmplActionGeneralAttribute), u32>,
    pub anime: TmplAnimation,
}

#[typetag::deserialize(name = "ActionGeneral")]
impl TmplAny for TmplActionGeneral {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn class(&self) -> TmplClass {
        TmplClass::ActionGeneral
    }
}

impl TmplActionGeneral {
    pub fn animations(&self) -> TmplActionGeneralIter<'_> {
        TmplActionGeneralIter { action: self, idx: 0 }
    }
}

pub struct TmplActionGeneralIter<'t> {
    action: &'t TmplActionGeneral,
    idx: usize,
}

impl<'t> Iterator for TmplActionGeneralIter<'t> {
    type Item = &'t TmplAnimation;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;
        match idx {
            0 => Some(&self.action.anime),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{s, KeyCode};

    #[test]
    fn test_load_action_general_empty() {
        let db = TmplDatabase::new("../test_res").unwrap();

        let act = db.find_as::<TmplActionGeneral>(&s!("Action.GeneralEmpty")).unwrap();
        assert_eq!(act.id, s!("Action.GeneralEmpty"));
        assert_eq!(act.arguments.len(), 0);
        assert_eq!(act.enabled, TmplSwitch::Bool(true));
        assert_eq!(act.enter_key, None);
        assert_eq!(act.enter_level, LEVEL_FREE);
        assert_eq!(act.base_derive_level, LEVEL_PROGRESSING);
        assert_eq!(act.derive_level, LEVEL_PROGRESSING);
        assert_eq!(act.derive_start, 0);
        assert_eq!(act.derive_duration, 0);
        assert_eq!(act.derives.len(), 0);
        assert_eq!(act.just_enabled, TmplSwitch::Bool(false));
        assert_eq!(act.insertion_enabled, TmplSwitch::Bool(false));
        assert_eq!(act.attributes.len(), 0);

        assert_eq!(act.anime.file, "empty.ozz");
        assert_eq!(act.anime.duration, 15 * 3);
        assert_eq!(act.anime.times, 1);
        assert!(!act.anime.additive);
        assert_eq!(act.anime.body_progress, None);
        assert_eq!(act.animations().count(), 1);
    }

    #[test]
    fn test_load_action_general() {
        let db = TmplDatabase::new("../test_res").unwrap();

        let skill = db.find_as::<TmplActionGeneral>(&s!("Action.No1.Skill")).unwrap();
        assert_eq!(skill.id, s!("Action.No1.Skill"));
        assert_eq!(
            skill
                .arguments
                .iter()
                .map(|x| (x.0.clone(), x.1))
                .collect::<Vec<(Symbol, TmplLevelRange)>>(),
            vec![
                (s!("just"), TmplLevelRange::new(0, 1)),
                (s!("insertion"), TmplLevelRange::new(0, 1)),
                (s!("level"), TmplLevelRange::new(1, 2)),
            ]
        );
        assert_eq!(skill.enabled, TmplSwitch::Bool(true));
        assert_eq!(skill.enter_key, Some(KeyCode::B1));
        assert_eq!(skill.enter_level, 0);
        assert_eq!(skill.base_derive_level, LEVEL_PROGRESSING);
        assert_eq!(skill.derive_level, LEVEL_SKILL);
        assert_eq!(skill.derive_start, 15 * 6);
        assert_eq!(skill.derive_duration, 15 * 4);
        assert_eq!(
            skill
                .derives
                .iter().cloned()
                .collect::<Vec<TmplDeriveAction>>(),
            vec![TmplDeriveAction {
                key: KeyCode::X1,
                enabled: TmplSwitch::Bool(true),
                action: s!("Action.No1.Atk2"),
            },]
        );
        assert_eq!(skill.just_enabled, TmplSwitch::Symbol(s!("just")));
        assert_eq!(skill.just_start, 0);
        assert_eq!(skill.just_duration, 0);
        assert_eq!(skill.just_hit, Symbol::default());
        assert_eq!(skill.insertion_enabled, TmplSwitch::Symbol(s!("insertion")));
        assert_eq!(skill.insertion_actions, 0x14);
        assert_eq!(skill.insertion_derive_duration, 15 * 2);
        assert_eq!(
            skill
                .attributes
                .iter()
                .map(|x| (x.0 .0.clone(), x.0 .1, x.1.to_vec()))
                .collect::<Vec<(Symbol, TmplActionGeneralAttribute, Vec<u32>)>>(),
            vec![
                (s!("level"), TmplActionGeneralAttribute::EnterLevel, vec![0, 100]),
                (s!("level"), TmplActionGeneralAttribute::JustStart, vec![105, 106]),
                (s!("level"), TmplActionGeneralAttribute::JustDuration, vec![5, 7]),
            ]
        );
        assert_eq!(skill.animations().count(), 1);
    }
}

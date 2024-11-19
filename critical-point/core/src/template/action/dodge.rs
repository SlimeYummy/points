use super::base::*;
use crate::template::base::{TmplAny, TmplLevelRange, TmplSwitch, TmplType};
use crate::utils::{KeyCode, KvList, List, StrID, Symbol, Table};

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum TmplActionDodgeAttribute {
    EnterLevel,
    PerfectStart,
    PerfectDuration,
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplActionDodge {
    pub id: StrID,
    #[serde(default)]
    pub arguments: KvList<Symbol, TmplLevelRange>,
    pub enabled: TmplSwitch,
    pub enter_key: Option<KeyCode>,
    pub enter_level: u16,
    pub antibreak_level: u16,
    pub derive_level: u16,
    pub derive_start: u32,
    pub derive_duration: u32,
    #[serde(default)]
    pub derives: List<TmplDeriveAction>,
    pub perfect_start: u32,
    pub perfect_duration: u32,
    pub dodge_derive_start: u32,
    pub dodge_derive_duration: u32,
    #[serde(default)]
    pub attributes: Table<(Symbol, TmplActionDodgeAttribute), u32>,
    pub anime_forward: TmplAnimation,
    pub anime_back: TmplAnimation,
    pub anime_left: TmplAnimation,
    pub anime_right: TmplAnimation,
}

#[typetag::deserialize(name = "ActionDodge")]
impl TmplAny for TmplActionDodge {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::ActionDodge
    }
}

impl TmplActionDodge {
    pub fn animations(&self) -> TmplActionDodgeIter<'_> {
        TmplActionDodgeIter { action: self, idx: 0 }
    }
}

pub struct TmplActionDodgeIter<'t> {
    action: &'t TmplActionDodge,
    idx: usize,
}

impl<'t> Iterator for TmplActionDodgeIter<'t> {
    type Item = &'t TmplAnimation;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;
        match idx {
            0 => Some(&self.action.anime_forward),
            1 => Some(&self.action.anime_back),
            2 => Some(&self.action.anime_left),
            3 => Some(&self.action.anime_right),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::s;

    #[test]
    fn test_load_action_dodge_empty() {
        let db = TmplDatabase::new("../test-res").unwrap();

        let act = db.find_as::<TmplActionDodge>(&s!("Action.DodgeEmpty")).unwrap();
        assert_eq!(act.id, s!("Action.DodgeEmpty"));
        assert_eq!(act.arguments.len(), 0);
        assert_eq!(act.enabled, TmplSwitch::Bool(true));
        assert_eq!(act.enter_key, Some(KeyCode::Dodge));
        assert_eq!(act.enter_level, LEVEL_SKILL);
        assert_eq!(act.antibreak_level, 0);
        assert_eq!(act.derive_level, LEVEL_FREE);
        assert_eq!(act.derive_start, 0);
        assert_eq!(act.derive_duration, 0);
        assert_eq!(act.derives.len(), 0);
        assert_eq!(act.perfect_start, 6);
        assert_eq!(act.perfect_duration, 9);
        assert_eq!(act.dodge_derive_start, 0);
        assert_eq!(act.dodge_derive_duration, 0);
        assert_eq!(act.attributes.len(), 0);
        assert_eq!(act.anime_forward.file, "empty.ozz");
        assert_eq!(act.anime_forward.duration, 45);
        assert_eq!(act.anime_back.file, "empty.ozz");
        assert_eq!(act.anime_left.file, "empty.ozz");
        assert_eq!(act.anime_right.file, "empty.ozz");
        assert_eq!(act.animations().count(), 4);
    }
}

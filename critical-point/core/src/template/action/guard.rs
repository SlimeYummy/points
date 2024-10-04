use super::base::*;
use crate::template::base::{TmplAny, TmplClass, TmplLevelRange, TmplSwitch};
use crate::utils::{KeyCode, KvList, List, StrID, Symbol, Table};

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum TmplActionGuardAttribute {
    EnterLevel,
    PerfectStart,
    PerfectDuration,
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplActionGuard {
    pub id: StrID,
    #[serde(default)]
    pub arguments: KvList<Symbol, TmplLevelRange>,
    pub enabled: TmplSwitch,
    pub enter_key: Option<KeyCode>,
    pub enter_level: u16,
    pub derive_level: u16,
    pub derive_start: u32,
    #[serde(default)]
    pub derives: List<TmplDeriveAction>,
    pub guard_start: u32,
    pub perfect_start: u32,
    pub perfect_duration: u32,
    pub guard_derive_start: u32,
    #[serde(default)]
    pub attributes: Table<(Symbol, TmplActionGuardAttribute), u32>,
    pub anime_enter: TmplAnimation,
    pub anime_leave: TmplAnimation,
    pub anime_move_forward: TmplAnimation,
    pub anime_move_back: TmplAnimation,
    pub anime_move_left: TmplAnimation,
    pub anime_move_right: TmplAnimation,
    pub anime_turn_left: Option<TmplAnimation>,
    pub anime_turn_right: Option<TmplAnimation>,
}

#[typetag::deserialize(name = "ActionGuard")]
impl TmplAny for TmplActionGuard {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn class(&self) -> TmplClass {
        TmplClass::ActionGuard
    }
}

impl TmplActionGuard {
    pub fn antibreak_level(&self) -> u16 {
        100
    }

    pub fn animations(&self) -> TmplActionGuardIter<'_> {
        TmplActionGuardIter { action: self, idx: 0 }
    }
}

pub struct TmplActionGuardIter<'t> {
    action: &'t TmplActionGuard,
    idx: usize,
}

impl<'t> Iterator for TmplActionGuardIter<'t> {
    type Item = &'t TmplAnimation;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let idx = self.idx;
            self.idx += 1;
            return match idx {
                0 => Some(&self.action.anime_enter),
                1 => Some(&self.action.anime_leave),
                2 => Some(&self.action.anime_move_forward),
                3 => Some(&self.action.anime_move_back),
                4 => Some(&self.action.anime_move_left),
                5 => Some(&self.action.anime_move_right),
                6 => match &self.action.anime_turn_left {
                    Some(anime) => Some(anime),
                    None => continue,
                },
                7 => match &self.action.anime_turn_right {
                    Some(anime) => Some(anime),
                    None => continue,
                },
                _ => None,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::s;

    #[test]
    fn test_load_action_guard_empty() {
        let db = TmplDatabase::new("../test_res").unwrap();

        let act = db.find_as::<TmplActionGuard>(&s!("Action.GuardEmpty")).unwrap();
        assert_eq!(act.id(), s!("Action.GuardEmpty"));
        assert_eq!(act.arguments.len(), 0);
        assert_eq!(act.enabled, TmplSwitch::Bool(true));
        assert_eq!(act.enter_key, Some(KeyCode::Guard));
        assert_eq!(act.enter_level, LEVEL_SKILL);
        assert_eq!(act.derive_start, 0);
        assert_eq!(act.derive_level, LEVEL_PROGRESSING);
        assert_eq!(act.derives.len(), 0);
        assert_eq!(act.guard_start, 4);
        assert_eq!(act.perfect_start, 10);
        assert_eq!(act.perfect_duration, 9);
        assert_eq!(act.guard_derive_start, 0);
        assert_eq!(act.attributes.len(), 0);
        assert_eq!(act.anime_enter.file, "empty.ozz");
        assert_eq!(act.anime_enter.duration, 45);
        assert_eq!(act.anime_leave.file, "empty.ozz");
        assert_eq!(act.anime_move_forward.file, "empty.ozz");
        assert_eq!(act.anime_move_back.file, "empty.ozz");
        assert_eq!(act.anime_move_left.file, "empty.ozz");
        assert_eq!(act.anime_move_right.file, "empty.ozz");
        assert!(act.anime_turn_left.is_none());
        assert!(act.anime_turn_right.is_none());
        assert_eq!(act.animations().count(), 6);
    }
}

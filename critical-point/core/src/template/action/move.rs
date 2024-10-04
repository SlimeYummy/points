use super::base::*;
use crate::template::base::{TmplAny, TmplClass, TmplLevelRange, TmplSwitch};
use crate::utils::{KeyCode, KvList, List, StrID, Symbol};

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplActionMove {
    pub id: StrID,
    #[serde(default)]
    pub arguments: KvList<Symbol, TmplLevelRange>,
    pub enabled: TmplSwitch,
    pub enter_key: Option<KeyCode>,
    pub antibreak_level: u16,
    pub derive_start: u32,
    #[serde(default)]
    pub derives: List<TmplDeriveAction>,
    pub anime_move: TmplAnimation,
    pub anime_turn_left: Option<TmplAnimation>,
    pub anime_turn_right: Option<TmplAnimation>,
    pub anime_yam_left: Option<TmplAnimation>,
    pub anime_yam_right: Option<TmplAnimation>,
    pub anime_stop: Option<TmplAnimation>,
}

#[typetag::deserialize(name = "ActionMove")]
impl TmplAny for TmplActionMove {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn class(&self) -> TmplClass {
        TmplClass::ActionMove
    }
}

impl TmplActionMove {
    pub fn enter_level(&self) -> u16 {
        LEVEL_SKILL
    }

    pub fn derive_level(&self) -> u16 {
        LEVEL_FREE
    }

    pub fn animations(&self) -> TmplActionMoveIter<'_> {
        TmplActionMoveIter { action: self, idx: 0 }
    }
}

pub struct TmplActionMoveIter<'t> {
    action: &'t TmplActionMove,
    idx: usize,
}

impl<'t> Iterator for TmplActionMoveIter<'t> {
    type Item = &'t TmplAnimation;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let idx = self.idx;
            self.idx += 1;
            return match idx {
                0 => Some(&self.action.anime_move),
                1 => match &self.action.anime_turn_left {
                    Some(anime) => Some(anime),
                    None => continue,
                },
                2 => match &self.action.anime_turn_right {
                    Some(anime) => Some(anime),
                    None => continue,
                },
                3 => match &self.action.anime_yam_left {
                    Some(anime) => Some(anime),
                    None => continue,
                },
                4 => match &self.action.anime_yam_right {
                    Some(anime) => Some(anime),
                    None => continue,
                },
                5 => match &self.action.anime_stop {
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
    fn test_load_action_move() {
        let db = TmplDatabase::new("../test_res").unwrap();

        let act = db.find_as::<TmplActionMove>(&s!("Action.No1.Run")).unwrap();
        assert_eq!(act.id, s!("Action.No1.Run"));
        assert_eq!(act.arguments.len(), 0);
        assert_eq!(act.enabled, TmplSwitch::Bool(true));
        assert_eq!(act.enter_key, Some(KeyCode::Run));
        assert_eq!(act.antibreak_level, 0);
        assert_eq!(act.derive_start, 0);
        assert_eq!(act.derives.len(), 0);
        assert_eq!(act.anime_move.file, "run.ozz");
        assert_eq!(act.anime_move.duration, 30);
        assert_eq!(act.anime_move.times, 0);
        assert!(act.anime_turn_left.is_none());
        assert!(act.anime_turn_right.is_none());
        assert!(act.anime_yam_left.is_none());
        assert!(act.anime_yam_right.is_none());
        assert!(act.anime_stop.is_none());
        assert_eq!(act.enter_level(), LEVEL_SKILL);
        assert_eq!(act.derive_level(), LEVEL_FREE);
        assert_eq!(act.animations().count(), 1);
    }
}

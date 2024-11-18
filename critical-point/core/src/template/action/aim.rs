use super::base::*;
use crate::template::base::{TmplAny, TmplSwitch, TmplType};
use crate::utils::{KeyCode, List, StrID, Symbol, Table};

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum TmplActionAimAttribute {
    AimStart,
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplActionAim {
    pub id: StrID,
    pub enabled: TmplSwitch,
    pub enter_key: Option<KeyCode>,
    pub enter_level: u16,
    pub derive_level: u16,
    pub antibreak_level: u16,
    pub aim_start: u32,
    pub attributes: Table<(Symbol, TmplActionAimAttribute), u32>,
    pub anime_enter: TmplAnimation,
    pub anime_leave: TmplAnimation,
    pub anime_move_forward: TmplAnimation,
    pub anime_move_back: TmplAnimation,
    pub anime_move_left: TmplAnimation,
    pub anime_move_right: TmplAnimation,
    pub anime_turn_left: Option<TmplAnimation>,
    pub anime_turn_right: Option<TmplAnimation>,
    pub derives: List<TmplDeriveAction>,
}

#[typetag::deserialize(name = "ActionAim")]
impl TmplAny for TmplActionAim {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn typ(&self) -> TmplType {
        TmplType::ActionAim
    }
}

impl TmplActionAim {
    pub fn animations(&self) -> TmplActionAimIter<'_> {
        TmplActionAimIter { action: self, idx: 0 }
    }
}

pub struct TmplActionAimIter<'t> {
    action: &'t TmplActionAim,
    idx: usize,
}

impl<'t> Iterator for TmplActionAimIter<'t> {
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

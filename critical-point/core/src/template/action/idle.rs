use super::base::*;
use crate::template::base::{TmplAny, TmplClass, TmplLevelRange, TmplSwitch};
use crate::utils::{KeyCode, KvList, List, StrID, Symbol};

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplActionIdle {
    pub id: StrID,
    #[serde(default)]
    pub arguments: KvList<Symbol, TmplLevelRange>,
    pub enabled: TmplSwitch,
    pub antibreak_level: u16,
    pub anime_idle: TmplAnimation,
    pub anime_ready: TmplAnimation,
    #[serde(default)]
    pub anime_random: List<TmplAnimation>,
    pub enter_time: u32,
    pub switch_time: u32,
    pub idle_enter_delay: u32,
}

#[typetag::deserialize(name = "ActionIdle")]
impl TmplAny for TmplActionIdle {
    fn id(&self) -> StrID {
        self.id.clone()
    }

    fn class(&self) -> TmplClass {
        TmplClass::ActionIdle
    }
}

impl TmplActionIdle {
    pub fn enter_key(&self) -> KeyCode {
        KeyCode::Idle
    }

    pub fn enter_level(&self) -> u16 {
        LEVEL_FREE
    }

    pub fn derive_level(&self) -> u16 {
        LEVEL_FREE
    }

    pub fn animations(&self) -> TmplActionIdleIter<'_> {
        TmplActionIdleIter { action: self, idx: 0 }
    }
}

pub struct TmplActionIdleIter<'t> {
    action: &'t TmplActionIdle,
    idx: usize,
}

impl<'t> Iterator for TmplActionIdleIter<'t> {
    type Item = &'t TmplAnimation;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;
        return match idx {
            0 => Some(&self.action.anime_idle),
            1 => Some(&self.action.anime_ready),
            _ => {
                let idx = idx - 2;
                self.action.anime_random.get(idx)
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{s, FPS};

    #[test]
    fn test_load_action_idle() {
        let db = TmplDatabase::new("../test_res").unwrap();

        let act = db.find_as::<TmplActionIdle>(&s!("Action.No1.Idle")).unwrap();
        assert_eq!(act.id, s!("Action.No1.Idle"));
        assert_eq!(act.arguments.len(), 0);
        assert_eq!(act.enabled, TmplSwitch::Bool(true));
        assert_eq!(act.antibreak_level, 0);
        assert_eq!(act.anime_idle.file, "girl_animation_logic_stand_idle.ozz");
        assert_eq!(act.anime_idle.duration, 2 * FPS);
        assert_eq!(act.anime_idle.times, 0);
        assert_eq!(act.anime_ready.file, "girl_animation_logic_stand_ready.ozz");
        assert_eq!(act.anime_ready.duration, 2 * FPS);
        assert_eq!(act.anime_ready.times, 0);
        assert_eq!(act.anime_random.len(), 0);
        assert_eq!(act.enter_time, 5);
        assert_eq!(act.switch_time, 5);
        assert_eq!(act.idle_enter_delay, 5 * FPS);
        assert_eq!(act.enter_key(), KeyCode::Idle);
        assert_eq!(act.enter_level(), LEVEL_FREE);
        assert_eq!(act.derive_level(), LEVEL_FREE);
        assert_eq!(act.animations().count(), 2);

        let act2 = db.find_as::<TmplActionIdle>(&s!("Action.No1.Idle2")).unwrap();
        assert_eq!(act2.id, s!("Action.No1.Idle2"));
        assert_eq!(act2.arguments.len(), 1);
        assert_eq!(act2.arguments[0], (s!("flag"), TmplLevelRange::new(0, 1)));
        assert_eq!(act2.enabled, TmplSwitch::Symbol(s!("flag")));
    }
}

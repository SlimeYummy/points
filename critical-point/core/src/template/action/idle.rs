use crate::template::action::base::TmplAnimation;
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::utils::TmplID;

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionIdle {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    pub character: TmplID,
    pub styles: Vec<TmplID>,
    pub anim_idle: TmplAnimation,
    pub anim_ready: TmplAnimation,
    #[serde(default)]
    pub anim_randoms: Vec<TmplAnimation>,
    pub auto_idle_delay: f32,
    pub enter_level: u16,
    pub derive_level: u16,
    pub poise_level: u16,
}

impl_tmpl!(TmplActionIdle, ActionIdle, "ActionIdle");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{id, LEVEL_IDLE};

    #[test]
    fn test_load_action_idle() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let act = db.find_as::<TmplActionIdle>(id!("Action.One.Idle")).unwrap();
        assert_eq!(act.id, id!("Action.One.Idle"));
        assert_eq!(act.enabled.value().unwrap(), true);
        assert_eq!(act.character, id!("Character.One"));
        assert_eq!(act.styles.as_slice(), &[id!("Style.One/1"), id!("Style.One/2")]);
        assert_eq!(act.anim_idle.files, "girl_stand_idle");
        assert_eq!(act.anim_idle.duration, 2.5);
        assert_eq!(act.anim_idle.fade_in, 0.2);
        assert_eq!(act.anim_idle.root_motion, false);
        assert_eq!(act.anim_idle.root_max_distance, 0.0);
        assert_eq!(act.anim_ready.files, "girl_stand_ready");
        assert_eq!(act.anim_ready.duration, 2.0);
        assert_eq!(act.anim_ready.fade_in, 0.2);
        assert_eq!(act.anim_ready.root_motion, false);
        assert_eq!(act.anim_ready.root_max_distance, 0.0);
        assert!(act.anim_randoms.is_empty());
        assert_eq!(act.auto_idle_delay, 10.0);
        assert_eq!(act.enter_level, LEVEL_IDLE);
        assert_eq!(act.derive_level, LEVEL_IDLE);
        assert_eq!(act.poise_level, 0);
    }
}

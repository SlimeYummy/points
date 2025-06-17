use crate::template::action::base::{TmplActionAttributes, TmplAnimation, TmplDeriveRule, TmplTimeline};
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::utils::{Bitsetable, DeriveContinue, EnumBitset, TmplID, VirtualKeyDir};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionGeneral {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    pub character: TmplID,
    pub styles: Vec<TmplID>,
    pub anim_main: TmplAnimation,
    pub enter_key: Option<VirtualKeyDir>,
    pub enter_level: u16,
    pub cool_down_time: TmplVar<f32>,
    pub cool_down_round: TmplVar<u16>,
    pub cool_down_init_round: TmplVar<u16>,
    pub motion_distance: [f32; 2],
    pub motion_toward: f32,
    pub attributes: TmplTimeline<TmplActionAttributes>,
    pub derive_levels: TmplTimeline<TmplVar<u16>>,
    #[serde(default)]
    pub derives: Vec<TmplDeriveRule>,
    #[serde(default)]
    pub derive_continues: EnumBitset<DeriveContinue, { DeriveContinue::LEN }>,
}

impl_tmpl!(TmplActionGeneral, ActionGeneral, "ActionGeneral");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{id, VirtualDir, VirtualKey, LEVEL_ACTION, LEVEL_ATTACK};

    #[test]
    fn test_load_action_general() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let act = db.find_as::<TmplActionGeneral>(id!("Action.One.Attack/1")).unwrap();
        assert_eq!(act.id, id!("Action.One.Attack/1"));
        assert_eq!(act.enabled.value().unwrap(), true);
        assert_eq!(act.character, id!("Character.One"));
        assert_eq!(act.styles.as_slice(), &[id!("Style.One/1"), id!("Style.One/2")]);
        assert_eq!(act.anim_main.files, "girl_attack1_1");
        assert_eq!(act.anim_main.duration, 4.0);
        assert_eq!(act.anim_main.fade_in, 0.2);
        assert_eq!(act.anim_main.root_motion, true);
        assert!(act.anim_main.root_max_distance > 0.0);
        assert_eq!(act.enter_key, Some(VirtualKeyDir::new(VirtualKey::Attack1, None)));
        assert_eq!(act.enter_level, LEVEL_ATTACK);
        assert_eq!(act.cool_down_time.value().unwrap(), 0.0);
        assert_eq!(act.cool_down_round.value().unwrap(), 1);
        assert_eq!(act.cool_down_init_round.value().unwrap(), 1);
        assert_eq!(act.motion_distance, [0.5, 1.0]);
        assert_eq!(act.motion_toward, 45.0);
        assert_eq!(act.attributes.fragments.len(), 1);
        assert_eq!(act.attributes.values.len(), 1);
        assert_eq!(act.attributes.values[0].damage_rdc.value().unwrap(), 0.2);
        assert_eq!(act.attributes.values[0].shield_dmg_rdc.value().unwrap(), 0.0);
        assert_eq!(act.attributes.values[0].poise_level.value().unwrap(), 1);
        assert_eq!(act.derive_levels.fragments.len(), 2);
        assert_eq!(act.derive_levels.values.len(), 2);
        assert_eq!(act.derive_levels.values[0].value().unwrap(), LEVEL_ACTION);
        assert_eq!(act.derive_levels.values[1].value().unwrap(), LEVEL_ATTACK);
        assert_eq!(act.derives.len(), 2);
        assert_eq!(act.derives[0].key, VirtualKey::Attack1);
        assert!(act.derives[0].dir.is_none());
        assert_eq!(act.derives[0].action.value().unwrap(), id!("Action.One.Attack/2"));
        assert_eq!(act.derives[1].key, VirtualKey::Attack2);
        assert_eq!(act.derives[1].dir.unwrap(), VirtualDir::Forward(0.5));
        assert_eq!(act.derives[1].action.value().unwrap(), id!("Action.One.Attack/2"));
        assert!(act.derive_continues.is_empty());
    }
}

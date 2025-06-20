use crate::template::action::base::TmplAnimation;
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::utils::TmplID;

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionMove {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    pub character: TmplID,
    pub styles: Vec<TmplID>,
    pub anim_move: TmplAnimation,
    #[serde(default)]
    pub anim_turn_left: Option<TmplAnimation>,
    #[serde(default)]
    pub anim_turn_right: Option<TmplAnimation>,
    #[serde(default)]
    pub anim_stop: Option<TmplAnimation>,
    pub yam_time: f32,
    pub turn_time: f32,
    pub enter_level: u16,
    pub derive_level: u16,
    pub poise_level: u16,
}

impl_tmpl!(TmplActionMove, ActionMove, "ActionMove");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::id;

    #[test]
    fn test_load_action_move() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let act = db.find_as::<TmplActionMove>(id!("Action.One.Run")).unwrap();
        assert_eq!(act.id, id!("Action.One.Run"));
        assert_eq!(act.enabled.value().unwrap(), true);
        assert_eq!(act.character, id!("Character.One"));
        assert_eq!(act.styles.as_slice(), &[id!("Style.One/1"), id!("Style.One/2")]);
        assert_eq!(act.anim_move.files, "girl_run");
        assert_eq!(act.anim_move.duration, 3.0);
        assert_eq!(act.anim_move.fade_in, 0.2);
        assert_eq!(act.anim_move.root_motion, false);
        assert_eq!(act.anim_move.root_max_distance, 0.0);
        assert!(act.anim_turn_left.is_none());
        assert!(act.anim_turn_right.is_none());
        assert!(act.anim_stop.is_none());
        assert_eq!(act.yam_time, 0.333);
        assert_eq!(act.turn_time, 1.0);
    }
}

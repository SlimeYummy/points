use crate::template::action::base::TmplAnimation;
use crate::template::base::impl_tmpl;
use crate::template::variable::TmplVar;
use crate::utils::{TmplID, VirtualKey};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionHit {
    pub id: TmplID,
    pub enabled: TmplVar<bool>,
    #[serde(default)]
    pub character: TmplID,
    #[serde(default)]
    pub styles: Vec<TmplID>,
    #[serde(default)]
    pub npc_characters: Vec<TmplID>,
    pub tags: Vec<String>,
    pub enter_key: VirtualKey,
    pub enter_level: u16,
    pub derive_level: u16,
    pub be_hits: Vec<TmplActionHitBeHit>,
    pub anim_down: Option<TmplAnimation>,
    #[serde(default)]
    pub max_down_time: f32,
    pub anim_recovery: Option<TmplAnimation>,
}

impl_tmpl!(TmplActionHit, ActionHit, "ActionHit");

// #[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
// #[rkyv(derive(Debug))]
// pub struct TmplNpcActionHit {
//     pub id: TmplID,
//     pub characters: Vec<TmplID>,
//     pub tags: Vec<String>,
//     pub enter_key: VirtualKey,
//     pub be_hits: Vec<TmplActionHitBeHit>,
//     pub anim_down: Option<TmplAnimation>,
//     #[serde(default)]
//     pub max_down_time: f32,
//     pub anim_recovery: Option<TmplAnimation>,
// }

// impl_tmpl!(TmplNpcActionHit, NpcActionHit, "NpcActionHit");

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
pub struct TmplActionHitBeHit {
    pub anim: TmplAnimation,
    pub enter_angle: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::database::TmplDatabase;
    use crate::utils::{cf2s, id};

    #[test]
    fn test_load_npc_action_hit() {
        let db = TmplDatabase::new(10240, 150).unwrap();

        let act = db.find_as::<TmplActionHit>(id!("Action.Enemy.Hit1")).unwrap();
        assert_eq!(act.id, id!("Action.Enemy.Hit1"));
        assert_eq!(act.npc_characters.as_slice(), &[id!("NpcCharacter.Enemy")]);
        assert_eq!(act.tags.as_slice(), &["Hit"]);
        assert_eq!(act.enter_key, VirtualKey::Hit1);
        assert_eq!(act.enter_level, 610);
        assert_eq!(act.derive_level, 600);

        assert_eq!(act.be_hits.len(), 1);
        assert_eq!(act.be_hits[0].enter_angle, 10f32.to_radians());
        assert_eq!(act.be_hits[0].anim.files, "TrainingDummy/Hit1_F.*");
        assert_eq!(act.be_hits[0].anim.duration, cf2s(20));
        assert_eq!(act.be_hits[0].anim.fade_in, 0.1);
        assert_eq!(act.be_hits[0].anim.root_motion, true);
        assert_eq!(act.be_hits[0].anim.weapon_motion, false);
        assert_eq!(act.be_hits[0].anim.hit_motion, false);

        assert!(act.anim_down.is_none());
        assert_eq!(act.max_down_time, 0.0);
        assert!(act.anim_recovery.is_none());
    }
}

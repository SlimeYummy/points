use educe::Educe;
use std::vec;

use crate::script::{script_in, script_out, sin, sout, ScriptInputMap, ScriptOutType, ScriptOutputMap};
use crate::template::{TmplAttributeType, TmplIsPlus};
use crate::utils::{xresf, Num, Table2, XResult};

#[derive(Debug, Default, Clone, Copy)]
pub struct PrimaryValues {
    pub max_health: Num,
    pub max_posture: Num,
    pub posture_recovery: Num,

    pub physical_attack: Num,
    pub elemental_attack: Num,
    pub arcane_attack: Num,

    pub physical_defense: Num,
    pub elemental_defense: Num,
    pub arcane_defense: Num,
}

impl PrimaryValues {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append_attribute(&mut self, label: TmplAttributeType, value: Num) {
        use TmplAttributeType as A;
        match label {
            A::MaxHealth => self.max_health += value,
            A::MaxPosture => self.max_posture += value,
            A::PostureRecovery => self.posture_recovery += value,
            A::PhysicalAttack => self.physical_attack += value,
            A::ElementalAttack => self.elemental_attack += value,
            A::ArcaneAttack => self.arcane_attack += value,
            A::PhysicalDefense => self.physical_defense += value,
            A::ElementalDefense => self.elemental_defense += value,
            A::ArcaneDefense => self.arcane_defense += value,
            _ => {}
        }
    }

    pub fn append_table(&mut self, level: u32, attributes: &Table2<TmplAttributeType, Num>) -> XResult<()> {
        if level == 0 {
            return Ok(());
        }

        for (attr, values) in attributes.iter() {
            match values.get(level as usize) {
                Some(value) => self.append_attribute(*attr, *value),
                None => return xresf!(BadAttribute; "attr={:?} level={}", attr, level),
            }
        }
        Ok(())
    }

    pub fn script_input() -> ScriptInputMap {
        script_in(
            "primary",
            vec![
                sin!(PrimaryValues, max_health),
                sin!(PrimaryValues, max_posture),
                sin!(PrimaryValues, posture_recovery),
                sin!(PrimaryValues, physical_attack),
                sin!(PrimaryValues, elemental_attack),
                sin!(PrimaryValues, arcane_attack),
                sin!(PrimaryValues, physical_defense),
                sin!(PrimaryValues, elemental_defense),
                sin!(PrimaryValues, arcane_defense),
            ],
        )
    }
}

#[derive(Educe, Debug, Clone, Copy)]
#[educe(Default)]
pub struct SecondaryValues {
    pub max_health_up: Num,
    pub max_health_down: Num,

    pub max_posture_up: Num,
    pub max_posture_down: Num,

    pub attack_up: Num,
    pub attack_down: Num,
    pub physical_attack_up: Num,
    pub physical_attack_down: Num,
    pub elemental_attack_up: Num,
    pub elemental_attack_down: Num,
    pub arcane_attack_up: Num,
    pub arcane_attack_down: Num,

    pub critical_chance: Num,
    pub critical_damage: Num,

    pub defense_up: Num,
    pub defense_down: Num,
    pub physical_defense_up: Num,
    pub physical_defense_down: Num,
    pub cut_defense_up: Num,
    pub cut_defense_down: Num,
    pub blunt_defense_up: Num,
    pub blunt_defense_down: Num,
    pub ammo_defense_up: Num,
    pub ammo_defense_down: Num,
    pub elemental_defense_up: Num,
    pub elemental_defense_down: Num,
    pub fire_defense_up: Num,
    pub fire_defense_down: Num,
    pub ice_defense_up: Num,
    pub ice_defense_down: Num,
    pub thunder_defense_up: Num,
    pub thunder_defense_down: Num,
    pub arcane_defense_up: Num,
    pub arcane_defense_down: Num,

    pub damage_up: Num,
    pub damage_down: Num,
    pub physical_damage_up: Num,
    pub physical_damage_down: Num,
    pub cut_damage_up: Num,
    pub cut_damage_down: Num,
    pub blunt_damage_up: Num,
    pub blunt_damage_down: Num,
    pub ammo_damage_up: Num,
    pub ammo_damage_down: Num,
    pub elemental_damage_up: Num,
    pub elemental_damage_down: Num,
    pub fire_damage_up: Num,
    pub fire_damage_down: Num,
    pub ice_damage_up: Num,
    pub ice_damage_down: Num,
    pub thunder_damage_up: Num,
    pub thunder_damage_down: Num,
    pub arcane_damage_up: Num,
    pub arcane_damage_down: Num,

    pub normal_damage_up: Num,
    pub normal_damage_down: Num,
    pub skill_damage_up: Num,
    pub skill_damage_down: Num,

    #[educe(Default = 1.0)]
    pub final_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_physical_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_cut_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_blunt_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_ammo_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_elemental_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_fire_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_ice_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_thunder_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_arcane_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_normal_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_skill_damage_ratio: Num,
}

impl SecondaryValues {
    pub fn append_attribute(&mut self, label: TmplAttributeType, value: Num) {
        use TmplAttributeType as A;
        match label {
            A::MaxHealthUp => self.max_health_up += value,
            A::MaxHealthDown => self.max_health_down += value,
            A::MaxPostureUp => self.max_posture_up += value,
            A::MaxPostureDown => self.max_posture_down += value,
            A::AttackUp => self.attack_up += value,
            A::AttackDown => self.attack_down += value,
            A::PhysicalAttackUp => self.physical_attack_up += value,
            A::PhysicalAttackDown => self.physical_attack_down += value,
            A::ElementalAttackUp => self.elemental_attack_up += value,
            A::ElementalAttackDown => self.elemental_attack_down += value,
            A::ArcaneAttackUp => self.arcane_attack_up += value,
            A::ArcaneAttackDown => self.arcane_attack_down += value,
            A::CriticalChance => self.critical_chance += value,
            A::CriticalDamage => self.critical_damage += value,
            A::DefenseUp => self.defense_up += value,
            A::DefenseDown => self.defense_down += value,
            A::PhysicalDefenseUp => self.physical_defense_up += value,
            A::PhysicalDefenseDown => self.physical_defense_down += value,
            A::CutDefenseUp => self.cut_damage_up += value,
            A::CutDefenseDown => self.cut_damage_down += value,
            A::BluntDefenseUp => self.blunt_damage_up += value,
            A::BluntDefenseDown => self.blunt_damage_down += value,
            A::AmmoDefenseUp => self.ammo_damage_up += value,
            A::AmmoDefenseDown => self.ammo_damage_down += value,
            A::ElementalDefenseUp => self.elemental_damage_up += value,
            A::ElementalDefenseDown => self.elemental_damage_down += value,
            A::FireDefenseUp => self.fire_damage_up += value,
            A::FireDefenseDown => self.fire_damage_down += value,
            A::IceDefenseUp => self.ice_damage_up += value,
            A::IceDefenseDown => self.ice_damage_down += value,
            A::ThunderDefenseUp => self.thunder_damage_up += value,
            A::ThunderDefenseDown => self.thunder_damage_down += value,
            A::ArcaneDefenseUp => self.arcane_damage_up += value,
            A::ArcaneDefenseDown => self.arcane_damage_down += value,
            A::DamageUp => self.damage_up += value,
            A::DamageDown => self.damage_down += value,
            A::PhysicalDamageUp => self.physical_damage_up += value,
            A::PhysicalDamageDown => self.physical_damage_down += value,
            A::CutDamageUp => self.cut_damage_up += value,
            A::CutDamageDown => self.cut_damage_down += value,
            A::BluntDamageUp => self.blunt_damage_up += value,
            A::BluntDamageDown => self.blunt_damage_down += value,
            A::AmmoDamageUp => self.ammo_damage_up += value,
            A::AmmoDamageDown => self.ammo_damage_down += value,
            A::ElementalDamageUp => self.elemental_damage_up += value,
            A::ElementalDamageDown => self.elemental_damage_down += value,
            A::FireDamageUp => self.fire_damage_up += value,
            A::FireDamageDown => self.fire_damage_down += value,
            A::IceDamageUp => self.ice_damage_up += value,
            A::IceDamageDown => self.ice_damage_down += value,
            A::ThunderDamageUp => self.thunder_damage_up += value,
            A::ThunderDamageDown => self.thunder_damage_down += value,
            A::ArcaneDamageUp => self.arcane_damage_up += value,
            A::ArcaneDamageDown => self.arcane_damage_down += value,
            A::NormalDamageUp => self.normal_damage_up += value,
            A::NormalDamageDown => self.normal_damage_down += value,
            A::SkillDamageUp => self.skill_damage_up += value,
            A::SkillDamageDown => self.skill_damage_down += value,
            A::FinalDamageRatio => self.final_damage_ratio *= 1.0 + value,
            A::FinalPhysicalDamageRatio => self.final_physical_damage_ratio *= 1.0 + value,
            A::FinalCutDamageRatio => self.final_cut_damage_ratio *= 1.0 + value,
            A::FinalBluntDamageRatio => self.final_blunt_damage_ratio *= 1.0 + value,
            A::FinalAmmoDamageRatio => self.final_ammo_damage_ratio *= 1.0 + value,
            A::FinalElementalDamageRatio => self.final_elemental_damage_ratio *= 1.0 + value,
            A::FinalFireDamageRatio => self.final_fire_damage_ratio *= 1.0 + value,
            A::FinalIceDamageRatio => self.final_ice_damage_ratio *= 1.0 + value,
            A::FinalThunderDamageRatio => self.final_thunder_damage_ratio *= 1.0 + value,
            A::FinalArcaneDamageRatio => self.final_arcane_damage_ratio *= 1.0 + value,
            A::FinalNormalDamageRatio => self.final_normal_damage_ratio *= 1.0 + value,
            A::FinalSkillDamageRatio => self.final_skill_damage_ratio *= 1.0 + value,
            _ => {}
        }
    }

    pub fn append_table(&mut self, level: u32, attributes: &Table2<TmplAttributeType, Num>) -> XResult<()> {
        for (attr, values) in attributes.iter() {
            match values.get(level as usize) {
                Some(value) => self.append_attribute(*attr, *value),
                None => return xresf!(BadAttribute; "attr={:?} level={}", attr, level),
            }
        }
        Ok(())
    }

    pub fn append_table_plus(
        &mut self,
        piece: u32,
        plus: u32,
        attributes: &Table2<(TmplAttributeType, TmplIsPlus), Num>,
    ) -> XResult<()> {
        for ((attr, is_plus), values) in attributes.iter() {
            let level = if *is_plus { plus } else { piece };
            match values.get(level as usize) {
                Some(value) => self.append_attribute(*attr, *value),
                None => return xresf!(BadAttribute; "attr={:?} level={}", attr, level),
            }
        }
        Ok(())
    }

    pub fn script_output() -> ScriptOutputMap {
        script_out(
            "secondary",
            vec![
                sout!(+, SecondaryValues, max_health_up),
                sout!(+, SecondaryValues, max_health_down),
                sout!(+, SecondaryValues, max_posture_up),
                sout!(+, SecondaryValues, max_posture_down),
                sout!(+, SecondaryValues, attack_up),
                sout!(+, SecondaryValues, attack_down),
                sout!(+, SecondaryValues, physical_attack_up),
                sout!(+, SecondaryValues, physical_attack_down),
                sout!(+, SecondaryValues, elemental_attack_up),
                sout!(+, SecondaryValues, elemental_attack_down),
                sout!(+, SecondaryValues, arcane_attack_up),
                sout!(+, SecondaryValues, arcane_attack_down),
                sout!(+, SecondaryValues, critical_chance),
                sout!(+, SecondaryValues, critical_damage),
                sout!(+, SecondaryValues, defense_up),
                sout!(+, SecondaryValues, defense_down),
                sout!(+, SecondaryValues, physical_defense_up),
                sout!(+, SecondaryValues, physical_defense_down),
                sout!(+, SecondaryValues, cut_defense_up),
                sout!(+, SecondaryValues, cut_defense_down),
                sout!(+, SecondaryValues, blunt_defense_up),
                sout!(+, SecondaryValues, blunt_defense_down),
                sout!(+, SecondaryValues, ammo_defense_up),
                sout!(+, SecondaryValues, ammo_defense_down),
                sout!(+, SecondaryValues, elemental_defense_up),
                sout!(+, SecondaryValues, elemental_defense_down),
                sout!(+, SecondaryValues, fire_defense_up),
                sout!(+, SecondaryValues, fire_defense_down),
                sout!(+, SecondaryValues, ice_defense_up),
                sout!(+, SecondaryValues, ice_defense_down),
                sout!(+, SecondaryValues, thunder_defense_up),
                sout!(+, SecondaryValues, thunder_defense_down),
                sout!(+, SecondaryValues, arcane_defense_up),
                sout!(+, SecondaryValues, arcane_defense_down),
                sout!(+, SecondaryValues, damage_up),
                sout!(+, SecondaryValues, damage_down),
                sout!(+, SecondaryValues, physical_damage_up),
                sout!(+, SecondaryValues, physical_damage_down),
                sout!(+, SecondaryValues, cut_damage_up),
                sout!(+, SecondaryValues, cut_damage_down),
                sout!(+, SecondaryValues, blunt_damage_up),
                sout!(+, SecondaryValues, blunt_damage_down),
                sout!(+, SecondaryValues, ammo_damage_up),
                sout!(+, SecondaryValues, ammo_damage_down),
                sout!(+, SecondaryValues, elemental_damage_up),
                sout!(+, SecondaryValues, elemental_damage_down),
                sout!(+, SecondaryValues, fire_damage_up),
                sout!(+, SecondaryValues, fire_damage_down),
                sout!(+, SecondaryValues, ice_damage_up),
                sout!(+, SecondaryValues, ice_damage_down),
                sout!(+, SecondaryValues, thunder_damage_up),
                sout!(+, SecondaryValues, thunder_damage_down),
                sout!(+, SecondaryValues, arcane_damage_up),
                sout!(+, SecondaryValues, arcane_damage_down),
                sout!(+, SecondaryValues, normal_damage_up),
                sout!(+, SecondaryValues, normal_damage_down),
                sout!(+, SecondaryValues, skill_damage_up),
                sout!(+, SecondaryValues, skill_damage_down),
                sout!(*, SecondaryValues, final_damage_ratio),
                sout!(*, SecondaryValues, final_physical_damage_ratio),
                sout!(*, SecondaryValues, final_cut_damage_ratio),
                sout!(*, SecondaryValues, final_blunt_damage_ratio),
                sout!(*, SecondaryValues, final_ammo_damage_ratio),
                sout!(*, SecondaryValues, final_elemental_damage_ratio),
                sout!(*, SecondaryValues, final_fire_damage_ratio),
                sout!(*, SecondaryValues, final_ice_damage_ratio),
                sout!(*, SecondaryValues, final_thunder_damage_ratio),
                sout!(*, SecondaryValues, final_arcane_damage_ratio),
                sout!(*, SecondaryValues, final_normal_damage_ratio),
                sout!(*, SecondaryValues, final_skill_damage_ratio),
            ],
        )
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ExtraValues {
    pub max_health: Num,
    pub max_posture: Num,

    pub physical_attack: Num,
    pub elemental_attack: Num,
    pub arcane_attack: Num,

    pub critical_chance: Num,
    pub critical_damage: Num,

    pub cut_defense: Num,
    pub blunt_defense: Num,
    pub ammo_defense: Num,
    pub fire_defense: Num,
    pub ice_defense: Num,
    pub thunder_defense: Num,
    pub arcane_defense: Num,
}

impl ExtraValues {
    pub fn script_output() -> ScriptOutputMap {
        script_out(
            "extra",
            vec![
                sout!(+, ExtraValues, max_health),
                sout!(+, ExtraValues, max_posture),
                sout!(+, ExtraValues, physical_attack),
                sout!(+, ExtraValues, elemental_attack),
                sout!(+, ExtraValues, arcane_attack),
                sout!(+, ExtraValues, critical_chance),
                sout!(+, ExtraValues, critical_damage),
                sout!(+, ExtraValues, cut_defense),
                sout!(+, ExtraValues, blunt_defense),
                sout!(+, ExtraValues, ammo_defense),
                sout!(+, ExtraValues, fire_defense),
                sout!(+, ExtraValues, ice_defense),
                sout!(+, ExtraValues, thunder_defense),
                sout!(+, ExtraValues, arcane_defense),
            ],
        )
    }
}

#[derive(Educe, Debug, Clone, Copy)]
#[educe(Default)]
pub struct PanelValues {
    pub max_health: Num,
    pub health_recovery: Num,

    pub max_posture: Num,
    pub posture_recovery: Num,

    pub physical_attack: Num,
    pub elemental_attack: Num,
    pub arcane_attack: Num,

    pub cut_defense: Num,
    pub blunt_defense: Num,
    pub ammo_defense: Num,
    pub fire_defense: Num,
    pub ice_defense: Num,
    pub thunder_defense: Num,
    pub arcane_defense: Num,

    pub max_health_up: Num,
    pub max_health_down: Num,

    pub max_posture_up: Num,
    pub max_posture_down: Num,

    pub physical_attack_up: Num,
    pub physical_attack_down: Num,
    pub elemental_attack_up: Num,
    pub elemental_attack_down: Num,
    pub arcane_attack_up: Num,
    pub arcane_attack_down: Num,

    pub critical_chance: Num,
    pub critical_damage: Num,

    pub cut_defense_up: Num,
    pub cut_defense_down: Num,
    pub blunt_defense_up: Num,
    pub blunt_defense_down: Num,
    pub ammo_defense_up: Num,
    pub ammo_defense_down: Num,
    pub fire_defense_up: Num,
    pub fire_defense_down: Num,
    pub ice_defense_up: Num,
    pub ice_defense_down: Num,
    pub thunder_defense_up: Num,
    pub thunder_defense_down: Num,
    pub arcane_defense_up: Num,
    pub arcane_defense_down: Num,

    pub cut_damage_up: Num,
    pub cut_damage_down: Num,
    pub blunt_damage_up: Num,
    pub blunt_damage_down: Num,
    pub ammo_damage_up: Num,
    pub ammo_damage_down: Num,
    pub fire_damage_up: Num,
    pub fire_damage_down: Num,
    pub ice_damage_up: Num,
    pub ice_damage_down: Num,
    pub thunder_damage_up: Num,
    pub thunder_damage_down: Num,
    pub arcane_damage_up: Num,
    pub arcane_damage_down: Num,

    pub normal_damage_up: Num,
    pub normal_damage_down: Num,
    pub skill_damage_up: Num,
    pub skill_damage_down: Num,

    #[educe(Default = 1.0)]
    pub final_max_health_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_health_recovery_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_max_posture_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_posture_recovery_ratio: Num,

    #[educe(Default = 1.0)]
    pub final_cut_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_blunt_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_ammo_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_fire_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_ice_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_thunder_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_arcane_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_normal_damage_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_skill_damage_ratio: Num,

    #[educe(Default = 1.0)]
    pub final_cut_injury_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_blunt_injury_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_ammo_injury_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_fire_injury_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_ice_injury_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_thunder_injury_ratio: Num,
    #[educe(Default = 1.0)]
    pub final_arcane_injury_ratio: Num,
}

impl PanelValues {
    pub fn new(primary: &PrimaryValues, secondary: &SecondaryValues) -> Self {
        let mut values = Self::default();

        values.max_health_up = secondary.max_health_up;
        values.max_health_down = secondary.max_health_down;
        values.max_posture_up = secondary.max_posture_up;
        values.max_posture_down = secondary.max_posture_down;

        values.physical_attack_up = secondary.attack_up + secondary.physical_attack_up;
        values.physical_attack_down = secondary.attack_down + secondary.physical_attack_down;
        values.elemental_attack_up = secondary.attack_up + secondary.elemental_attack_up;
        values.elemental_attack_down = secondary.attack_down + secondary.elemental_attack_down;
        values.arcane_attack_up = secondary.attack_up + secondary.arcane_attack_up;
        values.arcane_attack_down = secondary.attack_down + secondary.arcane_attack_down;

        values.critical_chance = secondary.critical_chance;
        values.critical_damage = secondary.critical_damage;

        values.cut_defense_up = secondary.defense_up + secondary.physical_defense_up + secondary.cut_defense_up;
        values.cut_defense_down = secondary.defense_down + secondary.physical_defense_down + secondary.cut_defense_down;
        values.blunt_defense_up = secondary.defense_up + secondary.physical_defense_up + secondary.blunt_defense_up;
        values.blunt_defense_down =
            secondary.defense_down + secondary.physical_defense_down + secondary.blunt_defense_down;
        values.ammo_defense_up = secondary.defense_up + secondary.physical_defense_up + secondary.ammo_defense_up;
        values.ammo_defense_down =
            secondary.defense_down + secondary.physical_defense_down + secondary.ammo_defense_down;
        values.fire_defense_up = secondary.defense_up + secondary.elemental_defense_up + secondary.fire_defense_up;
        values.fire_defense_down =
            secondary.defense_down + secondary.elemental_defense_down + secondary.fire_defense_down;
        values.ice_defense_up = secondary.defense_up + secondary.elemental_defense_up + secondary.ice_defense_up;
        values.ice_defense_down =
            secondary.defense_down + secondary.elemental_defense_down + secondary.ice_defense_down;
        values.thunder_defense_up =
            secondary.defense_up + secondary.elemental_defense_up + secondary.thunder_defense_up;
        values.thunder_defense_down =
            secondary.defense_down + secondary.elemental_defense_down + secondary.thunder_defense_down;
        values.arcane_defense_up = secondary.defense_up + secondary.arcane_defense_up;
        values.arcane_defense_down = secondary.defense_down + secondary.arcane_defense_down;

        values.cut_damage_up = secondary.damage_up + secondary.physical_attack_up + secondary.cut_damage_up;
        values.cut_damage_down = secondary.damage_down + secondary.physical_attack_down + secondary.cut_damage_down;
        values.blunt_damage_up = secondary.damage_up + secondary.physical_attack_up + secondary.blunt_damage_up;
        values.blunt_damage_down = secondary.damage_down + secondary.physical_attack_down + secondary.blunt_damage_down;
        values.ammo_damage_up = secondary.damage_up + secondary.physical_attack_up + secondary.ammo_damage_up;
        values.ammo_damage_down = secondary.damage_down + secondary.physical_attack_down + secondary.ammo_damage_down;
        values.fire_damage_up = secondary.damage_up + secondary.elemental_attack_up + secondary.fire_damage_up;
        values.fire_damage_down = secondary.damage_down + secondary.elemental_attack_down + secondary.fire_damage_down;
        values.ice_damage_up = secondary.damage_up + secondary.elemental_attack_up + secondary.ice_damage_up;
        values.ice_damage_down = secondary.damage_down + secondary.elemental_attack_down + secondary.ice_damage_down;
        values.thunder_damage_up = secondary.damage_up + secondary.elemental_attack_up + secondary.thunder_damage_up;
        values.thunder_damage_down =
            secondary.damage_down + secondary.elemental_attack_down + secondary.thunder_damage_down;
        values.arcane_damage_up = secondary.damage_up + secondary.arcane_damage_up;
        values.arcane_damage_down = secondary.damage_down + secondary.arcane_damage_down;

        values.normal_damage_up = secondary.normal_damage_up;
        values.normal_damage_down = secondary.normal_damage_down;
        values.skill_damage_up = secondary.skill_damage_up;
        values.skill_damage_down = secondary.skill_damage_down;

        values.final_cut_damage_ratio =
            secondary.final_damage_ratio * secondary.final_physical_damage_ratio * secondary.final_cut_damage_ratio;
        values.final_blunt_damage_ratio =
            secondary.final_damage_ratio * secondary.final_physical_damage_ratio * secondary.final_blunt_damage_ratio;
        values.final_ammo_damage_ratio =
            secondary.final_damage_ratio * secondary.final_physical_damage_ratio * secondary.final_ammo_damage_ratio;
        values.final_fire_damage_ratio =
            secondary.final_damage_ratio * secondary.final_elemental_damage_ratio * secondary.final_fire_damage_ratio;
        values.final_ice_damage_ratio =
            secondary.final_damage_ratio * secondary.final_elemental_damage_ratio * secondary.final_ice_damage_ratio;
        values.final_thunder_damage_ratio = secondary.final_damage_ratio
            * secondary.final_elemental_damage_ratio
            * secondary.final_thunder_damage_ratio;
        values.final_arcane_damage_ratio = secondary.final_damage_ratio * secondary.final_arcane_damage_ratio;

        fn ratio(up: Num, down: Num) -> Num {
            (1.0 + up) * Num::max(0.1, 1.0 - down)
        }

        values.max_health = primary.max_health * ratio(values.max_health_up, values.max_health_down);
        values.max_health = Num::max(values.max_health, 1.0);

        values.max_posture = primary.max_posture * ratio(values.max_posture_up, values.max_posture_down);
        values.max_posture = Num::max(values.max_posture, 1.0);

        values.physical_attack =
            primary.physical_attack * ratio(values.physical_attack_up, values.physical_attack_down);
        values.elemental_attack =
            primary.elemental_attack * ratio(values.elemental_attack_up, values.elemental_attack_down);
        values.arcane_attack = primary.arcane_attack * ratio(values.arcane_attack_up, values.arcane_attack_down);

        values.cut_defense = primary.physical_defense * ratio(values.cut_defense_up, values.cut_defense_down);
        values.blunt_defense = primary.physical_defense * ratio(values.blunt_defense_up, values.blunt_defense_down);
        values.ammo_defense = primary.physical_defense * ratio(values.ammo_defense_up, values.ammo_defense_down);
        values.fire_defense = primary.elemental_defense * ratio(values.fire_defense_up, values.fire_defense_down);
        values.ice_defense = primary.elemental_defense * ratio(values.ice_defense_up, values.ice_defense_down);
        values.thunder_defense =
            primary.elemental_defense * ratio(values.thunder_defense_up, values.thunder_defense_down);
        values.arcane_defense = primary.arcane_defense * ratio(values.arcane_defense_up, values.arcane_defense_down);

        values
    }

    pub fn append_extra(&mut self, extra: &ExtraValues) {
        fn norm(val: Num) -> Num {
            Num::max(1.0, val)
        }

        self.max_health = norm(self.max_health + extra.max_health);
        self.max_posture = norm(self.max_posture + extra.max_posture);

        self.physical_attack = norm(self.physical_attack + extra.physical_attack);
        self.elemental_attack = norm(self.elemental_attack + extra.elemental_attack);
        self.arcane_attack = norm(self.arcane_attack + extra.arcane_attack);

        self.cut_defense = norm(self.cut_defense + extra.cut_defense);
        self.blunt_defense = norm(self.blunt_defense + extra.blunt_defense);
        self.ammo_defense = norm(self.ammo_defense + extra.ammo_defense);
        self.fire_defense = norm(self.fire_defense + extra.fire_defense);
        self.ice_defense = norm(self.ice_defense + extra.ice_defense);
        self.thunder_defense = norm(self.thunder_defense + extra.thunder_defense);
        self.arcane_defense = norm(self.arcane_defense + extra.arcane_defense);
    }

    pub fn script_input() -> ScriptInputMap {
        script_in(
            "panel",
            vec![
                sin!(PanelValues, max_health_up),
                sin!(PanelValues, max_health_down),
                sin!(PanelValues, max_posture_up),
                sin!(PanelValues, max_posture_down),
                sin!(PanelValues, physical_attack_up),
                sin!(PanelValues, physical_attack_down),
                sin!(PanelValues, elemental_attack_up),
                sin!(PanelValues, elemental_attack_down),
                sin!(PanelValues, arcane_attack_up),
                sin!(PanelValues, arcane_attack_down),
                sin!(PanelValues, critical_chance),
                sin!(PanelValues, critical_damage),
                sin!(PanelValues, cut_defense_up),
                sin!(PanelValues, cut_defense_down),
                sin!(PanelValues, blunt_defense_up),
                sin!(PanelValues, blunt_defense_down),
                sin!(PanelValues, ammo_defense_up),
                sin!(PanelValues, ammo_defense_down),
                sin!(PanelValues, fire_defense_up),
                sin!(PanelValues, fire_defense_down),
                sin!(PanelValues, ice_defense_up),
                sin!(PanelValues, ice_defense_down),
                sin!(PanelValues, thunder_defense_up),
                sin!(PanelValues, thunder_defense_down),
                sin!(PanelValues, arcane_defense_up),
                sin!(PanelValues, arcane_defense_down),
                sin!(PanelValues, cut_damage_up),
                sin!(PanelValues, cut_damage_down),
                sin!(PanelValues, blunt_damage_up),
                sin!(PanelValues, blunt_damage_down),
                sin!(PanelValues, ammo_damage_up),
                sin!(PanelValues, ammo_damage_down),
                sin!(PanelValues, fire_damage_up),
                sin!(PanelValues, fire_damage_down),
                sin!(PanelValues, ice_damage_up),
                sin!(PanelValues, ice_damage_down),
                sin!(PanelValues, thunder_damage_up),
                sin!(PanelValues, thunder_damage_down),
                sin!(PanelValues, arcane_damage_up),
                sin!(PanelValues, arcane_damage_down),
                sin!(PanelValues, normal_damage_up),
                sin!(PanelValues, normal_damage_down),
                sin!(PanelValues, skill_damage_up),
                sin!(PanelValues, skill_damage_down),
                sin!(PanelValues, final_max_health_ratio),
                sin!(PanelValues, final_health_recovery_ratio),
                sin!(PanelValues, final_max_posture_ratio),
                sin!(PanelValues, final_posture_recovery_ratio),
                sin!(PanelValues, final_cut_damage_ratio),
                sin!(PanelValues, final_blunt_damage_ratio),
                sin!(PanelValues, final_ammo_damage_ratio),
                sin!(PanelValues, final_fire_damage_ratio),
                sin!(PanelValues, final_ice_damage_ratio),
                sin!(PanelValues, final_thunder_damage_ratio),
                sin!(PanelValues, final_arcane_damage_ratio),
                sin!(PanelValues, final_normal_damage_ratio),
                sin!(PanelValues, final_skill_damage_ratio),
                sin!(PanelValues, final_cut_injury_ratio),
                sin!(PanelValues, final_blunt_injury_ratio),
                sin!(PanelValues, final_ammo_injury_ratio),
                sin!(PanelValues, final_fire_injury_ratio),
                sin!(PanelValues, final_ice_injury_ratio),
                sin!(PanelValues, final_thunder_injury_ratio),
                sin!(PanelValues, final_arcane_injury_ratio),
            ],
        )
    }
}

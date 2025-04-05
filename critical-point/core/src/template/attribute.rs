use crate::utils::rkyv_self;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TmplAttribute {
    //
    // primary attributes
    //
    MaxHealth,
    HealthCureRatio,
    MaxPosture,
    PostureRecovery,

    PhysicalAttack,
    ElementalAttack,
    ArcaneAttack,

    PhysicalDefense,
    ElementalDefense,
    ArcaneDefense,

    //
    // secondary attributes
    //
    MaxHealthUp,
    MaxHealthDown,
    MaxPostureUp,
    MaxPostureDown,

    AttackUp,
    AttackDown,
    PhysicalAttackUp,
    PhysicalAttackDown,
    ElementalAttackUp,
    ElementalAttackDown,
    ArcaneAttackUp,
    ArcaneAttackDown,

    DefenseUp,
    DefenseDown,
    PhysicalDefenseUp,
    PhysicalDefenseDown,
    CutDefenseUp,
    CutDefenseDown,
    BluntDefenseUp,
    BluntDefenseDown,
    AmmoDefenseUp,
    AmmoDefenseDown,
    ElementalDefenseUp,
    ElementalDefenseDown,
    FireDefenseUp,
    FireDefenseDown,
    IceDefenseUp,
    IceDefenseDown,
    ThunderDefenseUp,
    ThunderDefenseDown,
    ArcaneDefenseUp,
    ArcaneDefenseDown,

    CriticalChance,
    CriticalDamage,

    DamageUp,
    DamageDown,
    PhysicalDamageUp,
    PhysicalDamageDown,
    CutDamageUp,
    CutDamageDown,
    BluntDamageUp,
    BluntDamageDown,
    AmmoDamageUp,
    AmmoDamageDown,
    ElementalDamageUp,
    ElementalDamageDown,
    FireDamageUp,
    FireDamageDown,
    IceDamageUp,
    IceDamageDown,
    ThunderDamageUp,
    ThunderDamageDown,
    ArcaneDamageUp,
    ArcaneDamageDown,

    NormalDamageUp,
    NormalDamageDown,
    SkillDamageUp,
    SkillDamageDown,
    BurstDamageUp,
    BurstDamageDown,
    MeleeDamageUp,
    MeleeDamageDown,
    RangedDamageUp,
    RangedDamageDown,

    DepostureUp,
    DepostureDown,
    PhysicalDepostureUp,
    PhysicalDepostureDown,
    ElementalDepostureUp,
    ElementalDepostureDown,
    ArcaneDepostureUp,
    ArcaneDepostureDown,

    MeleeDepostureUp,
    MeleeDepostureDown,
    RangedDepostureUp,
    RangedDepostureDown,

    PerfectDodgeTime,
    PerfectGuardTime,

    //
    // final attributes
    //
    FinalDamageRatio,
    FinalPhysicalDamageRatio,
    FinalCutDamageRatio,
    FinalBluntDamageRatio,
    FinalAmmoDamageRatio,
    FinalElementalDamageRatio,
    FinalFireDamageRatio,
    FinalIceDamageRatio,
    FinalThunderDamageRatio,
    FinalArcaneDamageRatio,

    FinalNormalDamageRatio,
    FinalSkillDamageRatio,
    FinalBurstDamageRatio,
    FinalMeleeDamageRatio,
    FinalRangedDamageRatio,

    FinalDepostureRatio,
    FinalPhysicalDepostureRatio,
    FinalElementalDepostureRatio,
    FinalArcaneDepostureRatio,

    FinalNormalDepostureRatio,
    FinalSkillDepostureRatio,
    FinalBurstDepostureRatio,
    FinalMeleeDepostureRatio,
    FinalRangedDepostureRatio,

    //
    // action attributes
    //
    CutDamage,
    BluntDamage,
    AmmoDamage,
    FireDamage,
    IceDamage,
    ThunderDamage,
    ArcaneDamage,

    CutDeposture,
    BluntDeposture,
    AmmoDeposture,
    FireDeposture,
    IceDeposture,
    ThunderDeposture,
    ArcaneDeposture,

    SuperArmor,
    BreakArmor,
}

rkyv_self!(TmplAttribute);

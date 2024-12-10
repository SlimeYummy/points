from typing import Any, Literal, Mapping, Sequence, cast, get_args

from turning_point.base import _serialize_float, _serialize_list_float


PrimaryAttribute = Literal[
    "MaxHealth",
    "HealthCureRatio",
    "MaxPosture",
    "PostureRecovery",
    "PhysicalAttack",
    "ElementalAttack",
    "ArcaneAttack",
    "PhysicalDefense",
    "ElementalDefense",
    "ArcaneDefense",
]

MaxHealth = "MaxHealth"
HealthCureRatio = "HealthCureRatio"
MaxPosture = "MaxPosture"
PostureRecovery = "PostureRecovery"

PhysicalAttack = "PhysicalAttack"
ElementalAttack = "ElementalAttack"
ArcaneAttack = "ArcaneAttack"

PhysicalDefense = "PhysicalDefense"
ElementalDefense = "ElementalDefense"
ArcaneDefense = "ArcaneDefense"


SecondaryAttribute = Literal[
    "MaxHealthUp",
    "MaxPostureUp",
    "PostureRecoveryUp",
    "AttackUp",
    "AttackDown",
    "PhysicalAttackUp",
    "PhysicalAttackDown",
    "ElementalAttackUp",
    "ElementalAttackDown",
    "ArcaneAttackUp",
    "ArcaneAttackDown",
    "DefenseUp",
    "DefenseDown",
    "PhysicalDefenseUp",
    "PhysicalDefenseDown",
    "CutDefenseUp",
    "CutDefenseDown",
    "BluntDefenseUp",
    "BluntDefenseDown",
    "AmmoDefenseUp",
    "AmmoDefenseDown",
    "ElementalDefenseUp",
    "ElementalDefenseDown",
    "FireDefenseUp",
    "FireDefenseDown",
    "IceDefenseUp",
    "IceDefenseDown",
    "ThunderDefenseUp",
    "ThunderDefenseDown",
    "ArcaneDefenseUp",
    "ArcaneDefenseDown",
    "CriticalChance",
    "CriticalDamage",
    "DamageUp",
    "DamageDown",
    "PhysicalDamageUp",
    "PhysicalDamageDown",
    "CutDamageUp",
    "CutDamageDown",
    "BluntDamageUp",
    "BluntDamageDown",
    "AmmoDamageUp",
    "AmmoDamageDown",
    "ElementalDamageUp",
    "ElementalDamageDown",
    "FireDamageUp",
    "FireDamageDown",
    "IceDamageUp",
    "IceDamageDown",
    "ThunderDamageUp",
    "ThunderDamageDown",
    "ArcaneDamageUp",
    "ArcaneDamageDown",
    "NormalDamageUp",
    "NormalDamageDown",
    "SkillDamageUp",
    "SkillDamageDown",
    "BurstDamageUp",
    "BurstDamageDown",
    "MeleeDamageUp",
    "MeleeDamageDown",
    "RangedDamageUp",
    "RangedDamageDown",
    "DepostureUp",
    "DepostureDown",
    "PhysicalDepostureUp",
    "PhysicalDepostureDown",
    "ElementalDepostureUp",
    "ElementalDepostureDown",
    "ArcaneDepostureUp",
    "ArcaneDepostureDown",
    "MeleeDepostureUp",
    "MeleeDepostureDown",
    "RangedDepostureUp",
    "RangedDepostureDown",
    "PerfectDodgeTime",
    "PerfectGuardTime",
]

MaxHealthUp = "MaxHealthUp"
MaxPostureUp = "MaxPostureUp"
PostureRecoveryUp = "PostureRecoveryUp"

AttackUp = "AttackUp"
AttackDown = "AttackDown"
PhysicalAttackUp = "PhysicalAttackUp"
PhysicalAttackDown = "PhysicalAttackDown"
ElementalAttackUp = "ElementalAttackUp"
ElementalAttackDown = "ElementalAttackDown"
ArcaneAttackUp = "ArcaneAttackUp"
ArcaneAttackDown = "ArcaneAttackDown"

DefenseUp = "DefenseUp"
DefenseDown = "DefenseDown"
PhysicalDefenseUp = "PhysicalDefenseUp"
PhysicalDefenseDown = "PhysicalDefenseDown"
CutDefenseUp = "CutDefenseUp"
CutDefenseDown = "CutDefenseDown"
BluntDefenseUp = "BluntDefenseUp"
BluntDefenseDown = "BluntDefenseDown"
AmmoDefenseUp = "AmmoDefenseUp"
AmmoDefenseDown = "AmmoDefenseDown"
ElementalDefenseUp = "ElementalDefenseUp"
ElementalDefenseDown = "ElementalDefenseDown"
FireDefenseUp = "FireDefenseUp"
FireDefenseDown = "FireDefenseDown"
IceDefenseUp = "IceDefenseUp"
IceDefenseDown = "IceDefenseDown"
ThunderDefenseUp = "ThunderDefenseUp"
ThunderDefenseDown = "ThunderDefenseDown"
ArcaneDefenseUp = "ArcaneDefenseUp"
ArcaneDefenseDown = "ArcaneDefenseDown"

CriticalChance = "CriticalChance"
CriticalDamage = "CriticalDamage"

DamageUp = "DamageUp"
DamageDown = "DamageDown"
PhysicalDamageUp = "PhysicalDamageUp"
PhysicalDamageDown = "PhysicalDamageDown"
CutDamageUp = "CutDamageUp"
CutDamageDown = "CutDamageDown"
BluntDamageUp = "BluntDamageUp"
BluntDamageDown = "BluntDamageDown"
AmmoDamageUp = "AmmoDamageUp"
AmmoDamageDown = "AmmoDamageDown"
ElementalDamageUp = "ElementalDamageUp"
ElementalDamageDown = "ElementalDamageDown"
FireDamageUp = "FireDamageUp"
FireDamageDown = "FireDamageDown"
IceDamageUp = "IceDamageUp"
IceDamageDown = "IceDamageDown"
ThunderDamageUp = "ThunderDamageUp"
ThunderDamageDown = "ThunderDamageDown"
ArcaneDamageUp = "ArcaneDamageUp"
ArcaneDamageDown = "ArcaneDamageDown"

NormalDamageUp = "NormalDamageUp"
NormalDamageDown = "NormalDamageDown"
SkillDamageUp = "SkillDamageUp"
SkillDamageDown = "SkillDamageDown"
BurstDamageUp = "BurstDamageUp"
BurstDamageDown = "BurstDamageDown"
MeleeDamageUp = "MeleeDamageUp"
MeleeDamageDown = "MeleeDamageDown"
RangedDamageUp = "RangedDamageUp"
RangedDamageDown = "RangedDamageDown"

DepostureUp = "DepostureUp"
DepostureDown = "DepostureDown"
PhysicalDepostureUp = "PhysicalDepostureUp"
PhysicalDepostureDown = "PhysicalDepostureDown"
ElementalDepostureUp = "ElementalDepostureUp"
ElementalDepostureDown = "ElementalDepostureDown"
ArcaneDepostureUp = "ArcaneDepostureUp"
ArcaneDepostureDown = "ArcaneDepostureDown"

MeleeDepostureUp = "MeleeDepostureUp"
MeleeDepostureDown = "MeleeDepostureDown"
RangedDepostureUp = "RangedDepostureUp"
RangedDepostureDown = "RangedDepostureDown"

PerfectDodgeTime = "PerfectDodgeTime"
PerfectGuardTime = "PerfectGuardTime"


FinalAttribute = Literal[
    "FinalMaxHealthRatio",
    "FinalMaxPostureRatio",
    "FinalPostureRecoveryRatio",
    "FinalDamageRatio",
    "FinalPhysicalDamageRatio",
    "FinalCutDamageRatio",
    "FinalBluntDamageRatio",
    "FinalAmmoDamageRatio",
    "FinalElementalDamageRatio",
    "FinalFireDamageRatio",
    "FinalIceDamageRatio",
    "FinalThunderDamageRatio",
    "FinalArcaneDamageRatio",
    "FinalNormalDamageRatio",
    "FinalSkillDamageRatio",
    "FinalBurstDamageRatio",
    "FinalMeleeDamageRatio",
    "FinalRangedDamageRatio",
    "FinalDepostureRatio",
    "FinalPhysicalDepostureRatio",
    "FinalElementalDepostureRatio",
    "FinalArcaneDepostureRatio",
    "FinalNormalDepostureRatio",
    "FinalSkillDepostureRatio",
    "FinalBurstDepostureRatio",
    "FinalMeleeDepostureRatio",
    "FinalRangedDepostureRatio",
]

FinalMaxHealthRatio = "FinalMaxHealthRatio"
FinalMaxPostureRatio = "FinalMaxPostureRatio"
FinalPostureRecoveryRatio = "FinalPostureRecoveryRatio"

FinalDamageRatio = "FinalDamageRatio"
FinalPhysicalDamageRatio = "FinalPhysicalDamageRatio"
FinalCutDamageRatio = "FinalCutDamageRatio"
FinalBluntDamageRatio = "FinalBluntDamageRatio"
FinalAmmoDamageRatio = "FinalAmmoDamageRatio"
FinalElementalDamageRatio = "FinalElementalDamageRatio"
FinalFireDamageRatio = "FinalFireDamageRatio"
FinalIceDamageRatio = "FinalIceDamageRatio"
FinalThunderDamageRatio = "FinalThunderDamageRatio"
FinalArcaneDamageRatio = "FinalArcaneDamageRatio"

FinalNormalDamageRatio = "FinalNormalDamageRatio"
FinalSkillDamageRatio = "FinalSkillDamageRatio"
FinalBurstDamageRatio = "FinalBurstDamageRatio"
FinalMeleeDamageRatio = "FinalMeleeDamageRatio"
FinalRangedDamageRatio = "FinalRangedDamageRatio"

FinalDepostureRatio = "FinalDepostureRatio"
FinalPhysicalDepostureRatio = "FinalPhysicalDepostureRatio"
FinalElementalDepostureRatio = "FinalElementalDepostureRatio"
FinalArcaneDepostureRatio = "FinalArcaneDepostureRatio"

FinalNormalDepostureRatio = "FinalNormalDepostureRatio"
FinalSkillDepostureRatio = "FinalSkillDepostureRatio"
FinalBurstDepostureRatio = "FinalBurstDepostureRatio"
FinalMeleeDepostureRatio = "FinalMeleeDepostureRatio"
FinalRangedDepostureRatio = "FinalRangedDepostureRatio"

_TypesHelper = {}
for key in get_args(PrimaryAttribute):
    _TypesHelper[key] = PrimaryAttribute
for key in get_args(SecondaryAttribute):
    _TypesHelper[key] = SecondaryAttribute
for key in get_args(FinalAttribute):
    _TypesHelper[key] = FinalAttribute


def _serialize_attributes(
    types: Sequence[Any],
    attributes: Mapping[str, float | str | Sequence[float | str]] | None,
    size: int | None,
    where: str = "?",
    optional: bool = False,
    zero: float | None = None,
) -> dict[str, list[float]] | None:
    if optional and attributes is None:
        return None
    if not isinstance(attributes, Mapping):
        raise Exception(f"{where}: must be a Mapping")

    ser = {}
    for attr, vals in attributes.items():
        if _TypesHelper.get(attr) not in types:
            raise Exception(f"{where}[{attr}]: attribute not supported")
        if size is not None:
            ser[attr] = _serialize_list_float(cast(Any, vals), size, f"{where}[{attr}]", zero=zero)
        else:
            ser[attr] = _serialize_float(cast(Any, vals), f"{where}[{attr}]")
    return ser


def _serialize_attributes_plus(
    types: Sequence[Any],
    attributes: Mapping[str, float | str | Sequence[float | str]] | None,
    size: int | None,
    where: str = "?",
    optional: bool = False,
    zero: float | None = None,
) -> list[dict[str, Any]] | None:
    if optional and attributes is None:
        return None
    if not isinstance(attributes, Mapping):
        raise Exception(f"{where}: must be a Mapping")

    ser = []
    for attr, vals in attributes.items():
        plus = attr.endswith("+")
        real_attr = attr[:-1] if plus else attr
        if _TypesHelper.get(real_attr) not in types:
            raise Exception(f"{where}[{attr}]: attribute not supported")

        ser_value = None
        if size is not None:
            ser_value = _serialize_list_float(cast(Any, vals), size, f"{where}[{attr}]", zero=zero)
        else:
            ser_value = _serialize_float(cast(Any, vals), f"{where}[{attr}]")
        ser.append({"k": (real_attr, plus), "v": ser_value})
    return ser

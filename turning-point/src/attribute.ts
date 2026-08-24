import { float, int, parseFloatArray } from './common';

//
// 属性的一些分类
// 属性计算涉及两个阶段，在战斗外计算静态属性的阶段，在战斗内动态计算属性的阶段，具体请参考Script部分。
// 为了明确计算流程，避免出现描述不明确，加成循环依赖，左脚踩右脚的情况，将属性定义为以下几类。
//
// xxx (无前缀)普通属性
// 非聚合属性，相当于词条上的一条普通加成，常用作Script的输出。
// 举例：
//   - max_health_up 生命上限提升x%
//   - critical_damage 暴击伤害提升x%
//   - fire_damage_up 火属性伤害提升x%
//
// primary_xxx 基础属性
// 聚合属性，合并了角色、装备、技能上的全部基础词条后获得。
// 基础属性不可由Script输出或修改，它们会作为Script的输入，传入后续Script阶段。
// 其它加成属性也大都参考基础属性计算，基础属性是一切属性的起点。
// 举例：
//   - primary_max_health 基础生命上限x
//   - primary_max_physical_attack 基础物理攻击x
//   - primary_max_elemental_denfense 基础元素防御x
//
// 静态属性 static_xxx
// 聚合属性，合并了角色、装备、技能上的全部静态效果后获得，可以理解为角色面板上展示的属性。
// 静态属性的计算参考了基础属性，它们在战斗内动态计算属性的阶段是不不可变的。
// 注意，在有些如WhenAssemble等脚本中，输入的静态属性表达的是未计算额外属性的估算值。
// 举例：
//   - static_max_posture 游戏开始前角色的架势上限x
//   - static_cut_attack_up 游戏开始前角色的斩击攻击提升x%
//   - static_arcane_damage_up 游戏开始前角色的奥术伤害提升x%
//
// 动态属性 dynamic_xxx
// 聚合属性，合并了静态属性，在加上Buff、被动、技能的全部动态效果，可以理解为之际战斗中结算的属性。
// 动态属性的计算参考了基础属性和静态属性。
// 注意，在有些如...等脚本中，输入的静态属性表达的是未计算额外属性的估算值。
// 举例：
//   - dynamic_health 当前生命值
//   - dynamic_posture 当前架势值
//   - dynamic_elemental_attack 当前元素攻击
//
// 额外属性 extra_xxx
// 非聚合属性，通常用于需要参考其他属性的加成（如：每100点生命上限额外加1攻击）。
// 通常，额外加成属性，应用于静态、动态属性计算的最后一个阶段。
// 在静态属性计算阶段，额外加成属性参考估算的静态属性计算。
// 在动态属性计算阶段，额外加成属性参考估算的动态属性计算。
// 举例：
//   - extra_ammo_attack 额外提升x射击攻击
//   - extra_ammo_attack_up 额外提升x%射击攻击
//   - extra_skill_damage_up 额外提升x%技能伤害
//
// 最终属性 final_xxx
// 非聚合属性，该类加成在动态属性计算完成后额外计算，且同名值乘算。
// 最终加成通常用于一些非常核心的机制，以确保这些机制绝对不会被稀释。
// 举例：
//   - final_damage_up 最终提升x%伤害
//   - final_damage_down 最终降低x%伤害
//

const _PRIMARY_ATTRIBUTES = [
    'MaxHealth',
    'HealthCureRatio',
    'MaxPosture',
    'PostureRecovery',
    'PhysicalAttack',
    'ElementalAttack',
    'ArcaneAttack',
    'PhysicalDefense',
    'ElementalDefense',
    'ArcaneDefense',
] as const;

export type PrimaryAttribute = (typeof _PRIMARY_ATTRIBUTES)[number];

export const PRIMARY_ATTRIBUTES: ReadonlySet<PrimaryAttribute> = new Set(_PRIMARY_ATTRIBUTES);

export function isPrimaryAttribute(attr: string): attr is PrimaryAttribute {
    return PRIMARY_ATTRIBUTES.has(attr as PrimaryAttribute);
}

export type PrimaryPlusAttribute = `$${PrimaryAttribute}`;

export const PRIMARY_PLUS_ATTRIBUTES: ReadonlySet<PrimaryPlusAttribute> = new Set(
    Array.from(PRIMARY_ATTRIBUTES.values()).map((x) => `$${x}` as PrimaryPlusAttribute),
);

export function isPrimaryPlusAttribute(attr: string): attr is PrimaryPlusAttribute {
    return PRIMARY_PLUS_ATTRIBUTES.has(attr as PrimaryPlusAttribute);
}

const _SECONDARY_ATTRIBUTES = [
    'MaxHealthUp',
    'MaxPostureUp',
    'PostureRecoveryUp',
    'AttackUp',
    'AttackDown',
    'PhysicalAttackUp',
    'PhysicalAttackDown',
    'ElementalAttackUp',
    'ElementalAttackDown',
    'ArcaneAttackUp',
    'ArcaneAttackDown',
    'DefenseUp',
    'DefenseDown',
    'PhysicalDefenseUp',
    'PhysicalDefenseDown',
    'CutDefenseUp',
    'CutDefenseDown',
    'BluntDefenseUp',
    'BluntDefenseDown',
    'AmmoDefenseUp',
    'AmmoDefenseDown',
    'ElementalDefenseUp',
    'ElementalDefenseDown',
    'FireDefenseUp',
    'FireDefenseDown',
    'IceDefenseUp',
    'IceDefenseDown',
    'ThunderDefenseUp',
    'ThunderDefenseDown',
    'ArcaneDefenseUp',
    'ArcaneDefenseDown',
    'CriticalChance',
    'CriticalDamage',
    'DamageUp',
    'DamageDown',
    'PhysicalDamageUp',
    'PhysicalDamageDown',
    'CutDamageUp',
    'CutDamageDown',
    'BluntDamageUp',
    'BluntDamageDown',
    'AmmoDamageUp',
    'AmmoDamageDown',
    'ElementalDamageUp',
    'ElementalDamageDown',
    'FireDamageUp',
    'FireDamageDown',
    'IceDamageUp',
    'IceDamageDown',
    'ThunderDamageUp',
    'ThunderDamageDown',
    'ArcaneDamageUp',
    'ArcaneDamageDown',
    'NormalDamageUp',
    'NormalDamageDown',
    'SkillDamageUp',
    'SkillDamageDown',
    'BurstDamageUp',
    'BurstDamageDown',
    'MeleeDamageUp',
    'MeleeDamageDown',
    'RangedDamageUp',
    'RangedDamageDown',
    'DepostureUp',
    'DepostureDown',
    'PhysicalDepostureUp',
    'PhysicalDepostureDown',
    'ElementalDepostureUp',
    'ElementalDepostureDown',
    'ArcaneDepostureUp',
    'ArcaneDepostureDown',
    'MeleeDepostureUp',
    'MeleeDepostureDown',
    'RangedDepostureUp',
    'RangedDepostureDown',
    'PerfectDodgeTime',
    'PerfectGuardTime',
] as const;

export type SecondaryAttribute = (typeof _SECONDARY_ATTRIBUTES)[number];

export const SECONDARY_ATTRIBUTES: ReadonlySet<SecondaryAttribute> = new Set(_SECONDARY_ATTRIBUTES);

export function isSecondaryAttribute(attr: string): attr is SecondaryAttribute {
    return SECONDARY_ATTRIBUTES.has(attr as SecondaryAttribute);
}

export type SecondaryPlusAttribute = `$${SecondaryAttribute}`;

export const SECONDARY_PLUS_ATTRIBUTES: ReadonlySet<SecondaryPlusAttribute> = new Set(
    Array.from(SECONDARY_ATTRIBUTES.values()).map((x) => `$${x}` as SecondaryPlusAttribute),
);

export function isSecondaryPlusAttribute(attr: string): attr is SecondaryPlusAttribute {
    return SECONDARY_PLUS_ATTRIBUTES.has(attr as SecondaryPlusAttribute);
}

const _FINAL_ATTRIBUTES = [
    'FinalMaxHealthRatio',
    'FinalMaxPostureRatio',
    'FinalPostureRecoveryRatio',
    'FinalDamageRatio',
    'FinalPhysicalDamageRatio',
    'FinalCutDamageRatio',
    'FinalBluntDamageRatio',
    'FinalAmmoDamageRatio',
    'FinalElementalDamageRatio',
    'FinalFireDamageRatio',
    'FinalIceDamageRatio',
    'FinalThunderDamageRatio',
    'FinalArcaneDamageRatio',
    'FinalNormalDamageRatio',
    'FinalSkillDamageRatio',
    'FinalBurstDamageRatio',
    'FinalMeleeDamageRatio',
    'FinalRangedDamageRatio',
    'FinalDepostureRatio',
    'FinalPhysicalDepostureRatio',
    'FinalElementalDepostureRatio',
    'FinalArcaneDepostureRatio',
    'FinalNormalDepostureRatio',
    'FinalSkillDepostureRatio',
    'FinalBurstDepostureRatio',
    'FinalMeleeDepostureRatio',
    'FinalRangedDepostureRatio',
] as const;

export type FinalAttribute = (typeof _FINAL_ATTRIBUTES)[number];

export const FINAL_ATTRIBUTES: ReadonlySet<FinalAttribute> = new Set(_FINAL_ATTRIBUTES);

export function isFinalAttribute(attr: string): attr is FinalAttribute {
    return FINAL_ATTRIBUTES.has(attr as FinalAttribute);
}

export type FinalPlusAttribute = `$${FinalAttribute}`;

export const FINAL_PLUS_ATTRIBUTES: ReadonlySet<FinalPlusAttribute> = new Set(
    Array.from(FINAL_ATTRIBUTES.values()).map((x) => `$${x}` as FinalPlusAttribute),
);

export function isFinalPlusAttribute(attr: string): attr is FinalPlusAttribute {
    return FINAL_PLUS_ATTRIBUTES.has(attr as FinalPlusAttribute);
}

const REVERSE_INDEX = new Map<string, ReadonlySet<string>>();
PRIMARY_ATTRIBUTES.forEach((attr) => REVERSE_INDEX.set(attr, PRIMARY_ATTRIBUTES));
PRIMARY_PLUS_ATTRIBUTES.forEach((attr) => REVERSE_INDEX.set(attr, PRIMARY_PLUS_ATTRIBUTES));
SECONDARY_ATTRIBUTES.forEach((attr) => REVERSE_INDEX.set(attr, SECONDARY_ATTRIBUTES));
SECONDARY_PLUS_ATTRIBUTES.forEach((attr) => REVERSE_INDEX.set(attr, SECONDARY_PLUS_ATTRIBUTES));
FINAL_ATTRIBUTES.forEach((attr) => REVERSE_INDEX.set(attr, FINAL_ATTRIBUTES));
FINAL_PLUS_ATTRIBUTES.forEach((attr) => REVERSE_INDEX.set(attr, FINAL_PLUS_ATTRIBUTES));

export function parseAttributeTable<
    A extends PrimaryAttribute | SecondaryAttribute | FinalAttribute,
>(
    attributes: Readonly<Record<string, ReadonlyArray<float | string>>>,
    includes: Array<ReadonlySet<string>>,
    where: string,
    opts: {
        len?: int;
    } = {},
): Readonly<Partial<Record<A, ReadonlyArray<float>>>> {
    if (typeof attributes !== 'object' || attributes === null) {
        throw new Error(`${where}: must be a object`);
    }

    const res: any = {}; // eslint-disable-next-line @typescript-eslint/no-explicit-any
    for (const [attr, vals] of Object.entries(attributes)) {
        const rev = REVERSE_INDEX.get(attr);
        if (!includes.find((x) => x === rev)) {
            throw new Error(`${where}[${attr}]: attribute not includes`);
        }
        res[attr] = parseFloatArray(vals!, `${where}[${attr}]`, { ...opts, type: 'f32' });
    }
    return res;
}

export function parseAttributePlusTable<
    A extends PrimaryAttribute | SecondaryAttribute | FinalAttribute,
>(
    attributes: Readonly<Record<string, ReadonlyArray<float | string>>>,
    includes: Array<ReadonlySet<string>>,
    where: string,
    opts: {
        len?: int;
    } = {},
): [
    Readonly<Partial<Record<A, ReadonlyArray<float>>>> | undefined,
    Readonly<Partial<Record<A, ReadonlyArray<float>>>> | undefined,
] {
    if (typeof attributes !== 'object' || attributes === null) {
        throw new Error(`${where}: must be a object`);
    }

    const attrs: any = {}; // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let any_attrs = false;
    const pcattrs: any = {}; // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let any_pcattrs = false;
    for (const [attr, vals] of Object.entries(attributes)) {
        const rev = REVERSE_INDEX.get(attr);
        if (!includes.find((x) => x === rev)) {
            throw new Error(`${where}[${attr}]: attribute not includes`);
        }
        if (attr.startsWith('$')) {
            pcattrs[attr.slice(1)] = parseFloatArray(vals!, `${where}[${attr}]`, {
                ...opts,
                type: 'f32',
            });
            any_pcattrs = true;
        } else {
            attrs[attr] = parseFloatArray(vals!, `${where}[${attr}]`, { ...opts, type: 'f32' });
            any_attrs = true;
        }
    }
    return [any_attrs ? attrs : undefined, any_pcattrs ? pcattrs : undefined];
}

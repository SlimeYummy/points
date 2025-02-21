import { int, parseFloatArray } from './common';

import { float } from './common';

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
        add_first?: number;
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
        res[attr] = parseFloatArray(vals!, `${where}[${attr}]`, opts);
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
        add_first?: number;
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
            const pcattr = attr.slice(0, -1);
            pcattrs[pcattr] = parseFloatArray(vals!, `${where}[${attr}]`, opts);
            any_pcattrs = true;
        } else {
            attrs[attr] = parseFloatArray(vals!, `${where}[${attr}]`, opts);
            any_attrs = true;
        }
    }
    return [any_attrs ? attrs : undefined, any_pcattrs ? pcattrs : undefined];
}

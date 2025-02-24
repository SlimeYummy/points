import {
    ID,
    IDPrefix,
    int,
    parseID,
    parseInt,
    parseIntArray,
    parseRareLevel,
    RareLevel,
    Variant1,
} from './common';
import { Resource } from './resource';
import { Entry } from './entry';

export const JEWEL_SLOTS = ['Attack', 'Defense', 'Special'] as const;

export type JewelSlot = (typeof JEWEL_SLOTS)[number];

export const Attack = 'Attack' as const;
export const Defense = 'Defense' as const;
export const Special = 'Special' as const;

export function isJewelSlot(raw: string): raw is JewelSlot {
    return JEWEL_SLOTS.includes(raw as JewelSlot);
}

export function parseJewelSlot(raw: string, where: string): JewelSlot {
    if (!JEWEL_SLOTS.includes(raw as JewelSlot)) {
        throw new Error(where + ': must be a JewelSlot');
    }
    return raw as JewelSlot;
}

export enum JewelSlotIndex {
    Special = 0,
    Attack = 1,
    Defense = 2,
}

const RE_SOLTS = /^([A|D|S]\d+)?([A|D|S]\d+)?([A|D|S]\d+)?$/;

export function parseJevelSlots(
    slot: string | readonly [int, int, int],
    where: string,
): readonly [int, int, int] {
    if (typeof slot === 'string') {
        const capture = slot.match(RE_SOLTS);
        if (capture) {
            const res: [int, int, int] = [0, 0, 0];
            for (const group of capture.slice(1)) {
                if (group?.[0] == 'S') {
                    res[JewelSlotIndex.Special] = Number.parseInt(group.slice(1)) || 0;
                } else if (group?.[0] == 'A') {
                    res[JewelSlotIndex.Attack] = Number.parseInt(group.slice(1)) || 0;
                } else if (group?.[0] == 'D') {
                    res[JewelSlotIndex.Defense] = Number.parseInt(group.slice(1)) || 0;
                }
            }
            return res;
        }
    } else if (Array.isArray(slot)) {
        return parseIntArray(slot, where, { len: 3, min: 0 }) as [int, int, int];
    }
    throw new Error(where + ': must be an A_D_S_/[int,int,int]');
}

export function parseJevelSlotsArray(
    slots: ReadonlyArray<string | readonly [int, int, int]>,
    where: string,
    opts: {
        len?: number;
        min_len?: number;
        max_len?: number;
        add_first?: string | readonly [int, int, int];
    } = {},
): ReadonlyArray<readonly [int, int, int]> {
    if (!Array.isArray(slots)) {
        throw new Error(where + ': must be an array');
    }
    if (opts.len !== undefined && slots.length !== opts.len) {
        throw new Error(`${where}: len must = ${opts.len}`);
    }
    if (opts.min_len !== undefined && slots.length < opts.min_len) {
        throw new Error(`${where}: length must > ${opts.min_len}`);
    }
    if (opts.max_len !== undefined && slots.length > opts.max_len) {
        throw new Error(`${where}: length must < ${opts.max_len}`);
    }

    const res = [];
    if (opts.add_first) {
        res.push(parseJevelSlots(opts.add_first, where));
    }
    for (const slot of slots) {
        res.push(parseJevelSlots(slot, where));
    }
    return res;
}

export const JEWEL_VARIANTS = ['Variant1', 'Variant2', 'Variant3', 'VariantX'] as const;

export type JewelVariant = (typeof JEWEL_VARIANTS)[number];

export function isJewelVariant(raw: JewelVariant | string): raw is JewelVariant {
    return JEWEL_VARIANTS.includes(raw as JewelVariant);
}

export function parseJewelVariant(raw: string, where: string): JewelVariant {
    if (!JEWEL_VARIANTS.includes(raw as JewelVariant)) {
        throw new Error(where + ': must be a JewelVariant');
    }
    return raw as JewelVariant;
}

export type JewelArgs = {
    /** 词条类型 决定宝石嵌入那种插槽 */
    slot: JewelSlot | string;

    /** 稀有度等级 */
    rare: RareLevel;

    /** 对应词条 */
    entry: ID;

    /** 词条的叠加数 */
    piece: int;

    /** 对应词条(副词条) */
    sub_entry?: ID;

    /** 词条的叠加数(副叠加数) */
    sub_piece?: int;

    /** 变体类型 用于区分同稀有度下的同名宝石 */
    variant: JewelVariant;
};

/**
 * 宝石，一类嵌入插槽(slot)的强化附件。
 *
 * 宝石分为attack/defense/special三种类型，与slot配套。
 * 依据词条价值分为R1/R2/R3三个稀有度，attack/defense常见于R1/R2，special常见于R3。
 * 原则上宝石类型因与词条类型匹配，但不排除某些效果较强的会升格成R3&special。
 * 宝石可以通过合成强化「+」值，具体参考词条中的「+」值。
 */
export class Jewel extends Resource {
    public static override prefix: IDPrefix = 'Jewel';

    public static override find(id: string, where: string): Jewel {
        const res = Resource.find(id, where);
        if (!(res instanceof Jewel)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 词条类型 决定宝石嵌入那种插槽 */
    public readonly slot: JewelSlot | string;

    /** 稀有度等级 */
    public readonly rare: RareLevel;

    /** 对应词条 */
    public readonly entry: ID;

    /** 词条的叠加数 */
    public readonly piece: int;

    /** 对应词条(副词条) */
    public readonly sub_entry?: ID;

    /** 词条的叠加数(副叠加数) */
    public readonly sub_piece?: int;

    /** 变体类型 用于区分同稀有度下的同名宝石 */
    public readonly variant: JewelVariant = Variant1;

    public constructor(id: ID, args: JewelArgs) {
        super(id);
        this.slot = parseJewelSlot(args.slot, this.w('slot'));
        this.rare = parseRareLevel(args.rare, this.w('rare'));
        this.entry = parseID(args.entry, 'Entry', this.w('entry'));
        this.piece = parseInt(args.piece, this.w('piece'), { min: 1 });
        if (!args.sub_entry !== !args.sub_piece) {
            throw this.e('', 'sub_entry & sub_piece must be using together');
        }
        this.sub_entry = !args.sub_entry
            ? undefined
            : parseID(args.sub_entry, 'Entry', this.w('sub_entry'));
        this.sub_piece = !args.sub_piece
            ? undefined
            : parseInt(args.sub_piece, this.w('sub_piece'), { min: 1 });
        this.variant = parseJewelVariant(args.variant, this.w('variant'));
    }

    public override verify() {
        const entry = Entry.find(this.entry, this.w('entry'));
        if (entry.max_piece < this.piece) {
            throw this.e('piece', 'must <= entry.max_piece');
        }
        if (this.sub_entry || this.sub_piece) {
            const sub_entry = Entry.find(this.sub_entry!, this.w('sub_entry'));
            if (sub_entry.max_piece < this.sub_piece!) {
                throw this.e('sub_piece', 'must <= sub_entry.max_piece');
            }
        }
    }
}

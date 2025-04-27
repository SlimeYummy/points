import {
    checkRecord,
    float,
    ID,
    IDPrefix,
    int,
    MAX_ENTRY_PLUS,
    parseFloat,
    parseID,
    parseInt,
    parseRareLevel,
    RareLevel,
} from './common';
import { Resource } from './resource';
import { Entry } from './entry';

export type AccessoryPatternArgs = {
    /** 稀有度等级 */
    rare: RareLevel;

    /** 随机词条生成模式 */
    patterns: string;

    /** 最高等级 */
    max_level: int;

    /** A组词条池 高价值随机词条池 数字表示概率占比 */
    a_entries: Readonly<Record<ID, float>>;

    /** B组词条池 低价值随机词条池(非攻击类技能) 数字表示概率占比 */
    b_entries: Readonly<Record<ID, float>>;
};

/**
 * 装饰品随机词条池，用于控制饰品随机词条生成。
 */
export class AccessoryPool extends Resource {
    public static override readonly prefix: IDPrefix = 'AccessoryPool';

    public static override find(id: string, where: string): AccessoryPool {
        const res = Resource.find(id, where);
        if (!(res instanceof AccessoryPool)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 稀有度等级 */
    public readonly rare: RareLevel;

    /** 随机词条生成模式 */
    public readonly patterns: ReadonlyArray<'A' | 'B' | 'AB'>;

    /** 最高等级 */
    public readonly max_level: int;

    /** A组词条池 高价值随机词条池 数字表示概率占比 */
    public readonly a_entries: Readonly<Record<ID, float>>;

    /** B组词条池 低价值随机词条池(非攻击类技能) 数字表示概率占比 */
    public readonly b_entries: Readonly<Record<ID, float>>;

    public constructor(id: ID, args: AccessoryPatternArgs) {
        super(id);
        this.rare = parseRareLevel(args.rare, this.w('rare'));
        this.max_level = parseInt(args.max_level, this.w('max_level'), { min: 1 });
        this.patterns = this.parsePattern(args.patterns, args.max_level);
        this.a_entries = this.parsePool(args.a_entries, 'a_entries');
        this.b_entries = this.parsePool(args.b_entries, 'b_entries');
    }

    private parsePattern(patterns: string, max_level: int): ReadonlyArray<'A' | 'B' | 'AB'> {
        const res: Array<'A' | 'B' | 'AB'> = [];
        const pats = patterns.split(' ');
        const expected_level = pats.length * MAX_ENTRY_PLUS;
        if (max_level !== expected_level) {
            throw this.e('max_level', `must = ${expected_level}`);
        }

        for (const [idx, item] of pats.entries()) {
            if (idx === 0) {
                if (item !== 'S') {
                    throw this.e('patterns', "must be a patterns like 'S A A B AB'");
                }
            } else if (item === 'A' || item === 'B' || item === 'AB') {
                res.push(item);
            } else {
                throw this.e('patterns', "must be a patterns like 'S A A B AB'");
            }
        }
        return res;
    }

    private parsePool(pool: Readonly<Record<ID, int>>, field: string): Readonly<Record<ID, int>> {
        checkRecord(pool, this.w(field));

        const res: Record<string, number> = {};
        for (const [id, val] of Object.entries(pool)) {
            const resId = parseID(id, 'Entry', this.w(`${field}[${id}]`));
            res[resId] = parseFloat(val, this.w(`${field}[${id}]`), { min: 0 });
        }
        return res;
    }

    public override verify() {
        for (const id of Object.keys(this.a_entries)) {
            Entry.find(id, this.w('a_entries'));
        }
        for (const id of Object.keys(this.b_entries)) {
            Entry.find(id, this.w('b_entries'));
        }
    }
}

export const ACCESSORY_VARIANTS = ['Variant1', 'Variant2', 'Variant3'] as const;

export type AccessoryVariant = (typeof ACCESSORY_VARIANTS)[number];

export function isAccessoryVariant(raw: AccessoryVariant | string): raw is AccessoryVariant {
    return ACCESSORY_VARIANTS.includes(raw as AccessoryVariant);
}

export function parseAccessoryVariant(raw: string, where: string): AccessoryVariant {
    if (!ACCESSORY_VARIANTS.includes(raw as AccessoryVariant)) {
        throw new Error(where + ': must be a AccessoryVariant');
    }
    return raw as AccessoryVariant;
}

export type AccessoryArgs = {
    /** 随机词条池 */
    pool: ID;

    /** 稀有度等级 */
    rare: RareLevel;

    /** 对应词条 */
    entry: ID;

    /** 词条的叠加数 */
    piece: int;

    /** 变体类型 用于区分同名宝石 */
    variant: AccessoryVariant;
};

/**
 * 装饰品，一类具有随机词条的物品，随机词条取决于AccessoryPool。
 */
export class Accessory extends Resource {
    public static override readonly prefix: IDPrefix = 'Accessory';

    public static override find(id: string, where: string): Accessory {
        const res = Resource.find(id, where);
        if (!(res instanceof Accessory)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 随机词条池 */
    public readonly pool: ID;

    /** 稀有度等级 */
    public readonly rare: RareLevel;

    /** 对应词条 */
    public readonly entry: ID;

    /** 词条的叠加数 */
    public readonly piece: int;

    /** 变体类型 用于区分同名宝石 */
    public readonly variant: AccessoryVariant;

    public constructor(id: ID, args: AccessoryArgs) {
        super(id);
        this.pool = parseID(args.pool, 'AccessoryPool', this.w('pool'));
        this.rare = parseRareLevel(args.rare, this.w('rare'));
        this.entry = parseID(args.entry, 'Entry', this.w('entry'));
        this.piece = parseInt(args.piece, this.w('piece'), { min: 1 });
        this.variant = parseAccessoryVariant(args.variant, this.w('variant'));
    }

    public override verify() {
        AccessoryPool.find(this.pool, this.w('pool'));
        const entry = Entry.find(this.entry, this.w('entry'));
        if (this.piece > entry.max_piece) {
            throw this.e('piece', `must <= entry.max_piece`);
        }
    }
}

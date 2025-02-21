export type IDPrefix =
    | '#'
    | 'Character'
    | 'Style'
    | 'Equipment'
    | 'Entry'
    | 'Perk'
    | 'AccessoryPool'
    | 'Accessory'
    | 'Jewel'
    | 'Action'
    | 'Material'
    | 'Zone';

export const RE_TMPL_ID_EXTRA =
    /^\.(\#|[\w\-_]{1,64})(?:\.([\w\-_]{1,64}))?(?:\.([\w\-_]{1,64}))?(?:\/([0-9]?[0-9A-Z]|[A-Z][0-9]))?$/;

export type ID = string;

export function parseID(raw: string, prefix: IDPrefix, where: string): string {
    if (typeof raw !== 'string') {
        throw new Error(`${where}: must be a ID`);
    }
    if (!raw.startsWith(prefix)) {
        throw new Error(`${where}: must start with "${prefix}"`);
    }
    if (!RE_TMPL_ID_EXTRA.test(raw.slice(prefix.length))) {
        throw new Error(`${where}: must match ID pattern`);
    }
    return raw;
}

export function parseIDArray(
    raw: ReadonlyArray<string>,
    prefix: IDPrefix,
    where: string,
    opts: {
        len?: number;
        min_len?: number;
        max_len?: number;
        add_first?: string;
        allow_conflict?: boolean;
    } = {},
): ReadonlyArray<string> {
    if (!Array.isArray(raw)) {
        throw new Error(`${where}: must be an array`);
    }
    if (opts.len !== undefined && raw.length !== opts.len) {
        throw new Error(`${where}: len must = ${opts.len}`);
    }
    if (opts.min_len !== undefined && raw.length < opts.min_len) {
        throw new Error(`${where}: length must be > ${opts.min_len}`);
    }
    if (opts.max_len !== undefined && raw.length > opts.max_len) {
        throw new Error(`${where}: length must be < ${opts.max_len}`);
    }

    const res: Array<string> = [];
    if (typeof opts.add_first === 'string') {
        res.push(opts.add_first);
    }

    for (const [idx, id] of Array.from(raw.entries())) {
        if (!opts.allow_conflict && res.find((x) => x == id)) {
            throw new Error(`${where}[${idx}]: ID conflict`);
        }
        res.push(parseID(id, prefix, `${where}[${idx}]`));
    }
    return res;
}

export const Variant1 = 'Variant1' as const;
export const Variant2 = 'Variant2' as const;
export const Variant3 = 'Variant3' as const;
export const VariantX = 'VariantX' as const;

export const RARE_LEVELS = ['Rare1', 'Rare2', 'Rare3'] as const;

export type RareLevel = (typeof RARE_LEVELS)[number];

export const Rare1 = 'Rare1' as const;
export const Rare2 = 'Rare2' as const;
export const Rare3 = 'Rare3' as const;

export function isRareLevel(raw: string): raw is RareLevel {
    return RARE_LEVELS.includes(raw as RareLevel);
}

export function parseRareLevel(raw: string, where: string): RareLevel {
    if (!RARE_LEVELS.includes(raw as RareLevel)) {
        throw new Error(where + ': must be a RareLevel');
    }
    return raw as RareLevel;
}

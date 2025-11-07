import path from 'node:path';

export type int = number;
export type float = number;
export type FilePath = string;

export function parseBool(raw: boolean | int, where: string): boolean {
    if (typeof raw !== 'boolean' && typeof raw !== 'number') {
        throw new Error(`${where}: must be a boolean`);
    }
    return Boolean(raw);
}

export function parseInt(
    raw: int | boolean,
    where: string,
    opts: {
        min?: int;
        max?: int;
        allow_bool?: boolean;
    } = {},
): int {
    if (opts.allow_bool) {
        if (typeof raw !== 'number' && typeof raw !== 'boolean') {
            throw new Error(`${where}: must be a int/boolean`);
        }
    } else {
        if (typeof raw !== 'number') {
            throw new Error(`${where}: must be a int`);
        }
    }

    if (opts.min !== undefined && (raw as number) < opts.min) {
        throw new Error(`${where}: must >= ${opts.min}`);
    }
    if (opts.max !== undefined && (raw as number) > opts.max) {
        throw new Error(`${where}: must <= ${opts.max}`);
    }
    return Math.round(raw as number);
}

const RE_PERCENT = /^\d+(?:\.\d+)?%$/;

export function parseFloat(
    raw: float | string,
    where: string,
    opts: {
        min?: float;
        max?: float;
    } = {},
): float {
    let res = 0.0;
    if (typeof raw === 'number') {
        res = raw;
    } else if (typeof raw === 'string') {
        if (RE_PERCENT.test(raw)) {
            res = Number.parseFloat(raw.slice(0, -1)) / 100;
        } else {
            res = Number.parseFloat(raw);
        }
    } else {
        throw new Error(`${where}: must be a float/percent`);
    }

    if (opts.min !== undefined && (raw as number) < opts.min) {
        throw new Error(`${where}: must >= ${opts.min}`);
    }
    if (opts.max !== undefined && (raw as number) > opts.max) {
        throw new Error(`${where}: must <= ${opts.max}`);
    }
    return res;
}

export function checkArray<T>(
    raw: ReadonlyArray<T>,
    where: string,
    opts: {
        len?: int;
        min_len?: int;
        max_len?: int;
        ascend?: boolean;
        descend?: boolean;
    } = {},
): ReadonlyArray<T> {
    if (!Array.isArray(raw)) {
        throw new Error(`${where}: must be an array`);
    }
    if (opts.len !== undefined && raw.length !== opts.len) {
        throw new Error(`${where}: length must = ${opts.len}`);
    }
    if (opts.min_len !== undefined && raw.length < opts.min_len) {
        throw new Error(`${where}: length must >= ${opts.min_len}`);
    }
    if (opts.max_len !== undefined && raw.length > opts.max_len) {
        throw new Error(`${where}: length must <= ${opts.max_len}`);
    }
    return raw;
}

export function checkOrder(
    raw: ReadonlyArray<number>,
    where: string,
    opts: {
        ascend?: boolean;
        descend?: boolean;
    } = {},
) {
    if (opts.ascend) {
        let prev = -Infinity;
        for (const [idx, item] of raw.entries()) {
            if (item < prev) {
                throw new Error(`${where}[${idx}]: must be ascend`);
            }
            prev = item;
        }
    }
    if (opts.descend) {
        let prev = Infinity;
        for (const [idx, item] of raw.entries()) {
            if (item > prev) {
                throw new Error(`${where}[${idx}]: must be descend`);
            }
            prev = item;
        }
    }
}

export function parseArray<R, T>(
    raw: ReadonlyArray<R>,
    where: string,
    callback: (value: R, where: string) => T,
    opts: {
        len?: int;
        min_len?: int;
        max_len?: int;
    } = {},
): ReadonlyArray<T> {
    checkArray(raw, where, opts);

    const res: Array<T> = [];
    for (const [idx, item] of raw.entries()) {
        res.push(callback(item, `${where}[${idx}]`));
    }
    return res;
}

export function parseBoolArray(
    raw: ReadonlyArray<int | boolean>,
    where: string,
    opts: {
        len?: int;
        min_len?: int;
        max_len?: int;
    } = {},
): ReadonlyArray<boolean> {
    checkArray(raw, where, opts);

    const res = [];
    for (const [idx, item] of raw.entries()) {
        res.push(parseBool(item, `${where}[${idx}]`));
    }
    return res;
}

export function parseIntArray(
    raw: ReadonlyArray<int | boolean>,
    where: string,
    opts: {
        min?: int;
        max?: int;
        len?: int;
        min_len?: int;
        max_len?: int;
        allow_bool?: boolean;
        ascend?: boolean;
        descend?: boolean;
        add_first?: int;
    } = {},
): ReadonlyArray<int> {
    checkArray(raw, where, opts);

    const res = [];
    if (typeof opts.add_first === 'number') {
        res.push(opts.add_first);
    }
    for (const [idx, item] of raw.entries()) {
        res.push(
            parseInt(item, `${where}[${idx}]`, {
                min: opts.min,
                max: opts.max,
                allow_bool: opts.allow_bool,
            }),
        );
    }
    checkOrder(typeof opts.add_first === 'number' ? res.slice(1) : res, where, {
        ascend: opts.ascend,
        descend: opts.descend,
    });
    return res;
}

export function parseIntRange(
    raw: ReadonlyArray<int>,
    where: string,
    opts: {
        min?: int;
        max?: int;
    } = {},
): readonly [int, int] {
    const res = parseIntArray(raw, where, { ...opts, len: 2 });
    if (res && res[0]! > res[1]!) {
        throw new Error(`${where}: range[0] must < range[1]`);
    }
    return res as [int, int];
}

export function parseFloatArray(
    raw: ReadonlyArray<float | string>,
    where: string,
    opts: {
        min?: float;
        max?: float;
        len?: int;
        min_len?: int;
        max_len?: int;
        ascend?: boolean;
        descend?: boolean;
        add_first?: float;
    } = {},
): ReadonlyArray<float> {
    checkArray(raw, where, opts);

    const res = [];
    if (typeof opts.add_first === 'number') {
        res.push(opts.add_first);
    }
    for (const [idx, item] of raw.entries()) {
        res.push(
            parseFloat(item, `${where}[${idx}]`, {
                min: opts.min,
                max: opts.max,
            }),
        );
    }
    checkOrder(typeof opts.add_first === 'number' ? res.slice(1) : res, where, {
        ascend: opts.ascend,
        descend: opts.descend,
    });
    return res;
}

export function parseFloatRange(
    raw: ReadonlyArray<float | string>,
    where: string,
    opts: {
        min?: float;
        max?: float;
    } = {},
): readonly [float, float] {
    const res = parseFloatArray(raw, where, { ...opts, len: 2 });
    if (res && res[0]! > res[1]!) {
        throw new Error(`${where}: range[0] must < range[1]`);
    }
    return res as [float, float];
}

export function parseString(
    raw: string,
    where: string,
    opts: {
        min_len?: int;
        max_len?: int;
        includes?: string[];
        regex?: RegExp;
    } = {},
): string {
    if (typeof raw !== 'string') {
        throw new Error(`${where}: must be a string`);
    }
    if (opts.min_len !== undefined && raw.length < opts.min_len) {
        throw new Error(`${where}: length must >= ${opts.min_len}`);
    }
    if (opts.max_len !== undefined && raw.length > opts.max_len) {
        throw new Error(`${where}: length must <= ${opts.max_len}`);
    }
    if (opts.includes && !opts.includes.includes(raw)) {
        throw new Error(`${where}: must include ${opts.includes}`);
    }
    if (opts.regex && !opts.regex.test(raw)) {
        throw new Error(`${where}: must match ${opts.regex}`);
    }
    return raw;
}

export function parseStringArray(
    raw: ReadonlyArray<string>,
    where: string,
    opts: {
        min_len?: int;
        max_len?: int;
        includes?: string[];
        regex?: RegExp;
        deduplicate?: boolean;
    } = {},
): ReadonlyArray<string> {
    checkArray(raw, where, opts);

    const res: Array<string> = [];
    for (const [idx, item] of raw.entries()) {
        if (opts.deduplicate && res.includes(item)) {
            throw new Error(`${where}[${idx}]: must be unique`);
        }
        res.push(
            parseString(item, `${where}[${idx}]`, {
                min_len: opts.min_len,
                max_len: opts.max_len,
                includes: opts.includes,
                regex: opts.regex,
            }),
        );
    }
    return res;
}

export function parseFile(
    raw: string,
    where: string,
    opts: {
        extension?: string | string[];
        can_absolute?: boolean;
    } = {},
): string {
    if (typeof raw !== 'string') {
        throw new Error(`${where}: must be a string`);
    }

    if (Array.isArray(opts.extension)) {
        const ext = path.extname(raw);
        if (!opts.extension.includes(ext)) {
            throw new Error(`${where}: must have extension ${opts.extension}`);
        }
    } else if (opts.extension) {
        if (path.extname(raw) !== opts.extension) {
            throw new Error(`${where}: must have extension ${opts.extension}`);
        }
    }

    if (!opts.can_absolute && path.isAbsolute(raw)) {
        throw new Error(`${where}: must be a relative path`);
    }
    return path.normalize(raw).replace(/\\/g, '/');
}

export function checkRecord<V>(
    raw: Readonly<Record<string, V>>,
    where: string,
): Readonly<Record<string, V>> {
    if (typeof raw !== 'object' || raw == null) {
        throw new Error(`${where}: must be a object`);
    }
    return raw;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function checkType<T, C extends new (...args: any[]) => T>(
    obj: T,
    cls: C,
    where: string,
): T {
    if (!(obj instanceof cls)) {
        throw new Error(`${where}: must be a ${cls.name}`);
    }
    return obj;
}

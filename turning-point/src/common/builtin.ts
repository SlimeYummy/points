import { FPS } from './config';
import path from 'path';

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

const RE_TIME = /^(\d+(?:\.\d+)*)(s|m|h|ms|min)$/;

function str2time(s: string): int {
    const match = RE_TIME.exec(s);
    if (!match || !match[1]) throw new Error('Invalid time');

    const tm = Number.parseInt(match[1]);
    switch (match[2]) {
        case 's':
            return Math.round(FPS * tm);
        case 'm':
        case 'min':
            return Math.round(FPS * tm * 60);
        case 'h':
            return Math.round(FPS * tm * 60 * 24);
        case 'ms':
            return Math.round((FPS * tm) / 1000);
        default:
            throw new Error('Invalid time');
    }
}

export function parseTime(
    raw: int | string,
    where: string,
    opts: {
        min?: int;
        max?: int;
    } = {},
): int {
    let res = 0;
    if (typeof raw === 'number') {
        res = raw;
    } else if (typeof raw === 'string') {
        res = str2time(raw);
    } else {
        throw new Error(`${where}: must be a int/time`);
    }

    if (opts.min !== undefined && res < opts.min) {
        throw new Error(`${where}: must >= ${opts.min}`);
    }
    if (opts.max !== undefined && res > opts.max) {
        throw new Error(`${where}: must <= ${opts.max}`);
    }
    return Math.round(res);
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
    } = {},
): ReadonlyArray<T> {
    if (!Array.isArray(raw)) {
        throw new Error(`${where}: must be an array`);
    }
    if (opts.len !== undefined && raw.length !== opts.len) {
        throw new Error(`${where}: len must = ${opts.len}`);
    }
    if (opts.min_len !== undefined && raw.length < opts.min_len) {
        throw new Error(`${where}: length must > ${opts.min_len}`);
    }
    if (opts.max_len !== undefined && raw.length > opts.max_len) {
        throw new Error(`${where}: length must < ${opts.max_len}`);
    }
    return raw;
}

export function parseArray<T>(
    raw: ReadonlyArray<T>,
    where: string,
    opts: {
        len?: int;
        min_len?: int;
        max_len?: int;
        add_first?: T;
    } = {},
): ReadonlyArray<T> {
    checkArray(raw, where, opts);

    if (!opts.add_first) {
        return Array.from(raw);
    }
    const array = [opts.add_first];
    for (let idx = 0; idx < array.length; ++idx) {
        array.push(raw[idx]!);
    }
    return array;
}

export function parseBoolArray(
    raw: ReadonlyArray<int | boolean>,
    where: string,
    opts: {
        len?: int;
        min_len?: int;
        max_len?: int;
        add_first?: boolean;
    } = {},
): ReadonlyArray<boolean> {
    checkArray(raw, where, opts);

    const res = [];
    if (opts.add_first !== undefined) {
        res.push(opts.add_first);
    }
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
        add_first?: int;
    } = {},
): ReadonlyArray<int> {
    checkArray(raw, where, opts);

    const res = [];
    if (opts.add_first !== undefined) {
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

export function parseTimeArray(
    raw: ReadonlyArray<int | string>,
    where: string,
    opts: {
        min?: int;
        max?: int;
        len?: int;
        min_len?: int;
        max_len?: int;
        add_first?: int;
    } = {},
): ReadonlyArray<int> {
    checkArray(raw, where, opts);

    const res = [];
    if (opts.add_first !== undefined) {
        res.push(opts.add_first);
    }
    for (const [idx, item] of raw.entries()) {
        res.push(
            parseTime(item, `${where}[${idx}]`, {
                min: opts.min,
                max: opts.max,
            }),
        );
    }
    return res;
}

export function parseTimeRange(
    raw: ReadonlyArray<int | string>,
    where: string,
    opts: {
        min?: int;
        max?: int;
    } = {},
): readonly [int, int] {
    const res = parseTimeArray(raw, where, { ...opts, len: 2 });
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
        add_first?: float;
    } = {},
): ReadonlyArray<float> {
    checkArray(raw, where, opts);

    const res = [];
    if (opts.add_first !== undefined) {
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
        regex?: RegExp;
    } = {},
): string {
    if (typeof raw !== 'string') {
        throw new Error(`${where}: must be a string`);
    }
    if (opts.min_len !== undefined && raw.length < opts.min_len) {
        throw new Error(`${where}: length must > ${opts.min_len}`);
    }
    if (opts.max_len !== undefined && raw.length > opts.max_len) {
        throw new Error(`${where}: length must < ${opts.max_len}`);
    }
    if (opts.regex && !opts.regex.test(raw)) {
        throw new Error(`${where}: must match "${opts.regex}"`);
    }
    return raw;
}

export function parseFile(
    raw: string,
    where: string,
    opts: {
        extension?: string;
        can_absolute?: boolean;
    } = {},
): string {
    if (typeof raw !== 'string') {
        throw new Error(`${where}: must be a string`);
    }
    if (opts.extension && path.extname(raw) !== opts.extension) {
        throw new Error(`${where}: must have extension ${opts.extension}`);
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

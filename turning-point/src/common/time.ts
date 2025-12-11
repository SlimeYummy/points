import { checkArray, checkOrder, float, int } from './builtin';
import { ENABLE_TIME_WARNING, FPS, LOGIC_SPF } from './config';

const RE_TIME = /^(\d+(?:\.\d+)*)(s|m|min|h|hr|ms|F)$/;

export function parseTime(
    raw: float | string,
    where: string,
    opts: {
        min?: float;
        max?: float;
        ignore_warning?: boolean;
    } = {},
): float {
    let res = 0;
    if (typeof raw === 'number') {
        res = raw;
    } else if (typeof raw === 'string') {
        res = str2time(raw, where);
    } else {
        throw new Error(`${where}: must be a float/time`);
    }

    if (opts.min !== undefined && res < opts.min) {
        throw new Error(`${where}: must >= ${opts.min}`);
    }
    if (opts.max !== undefined && res > opts.max) {
        throw new Error(`${where}: must <= ${opts.max}`);
    }

    if (ENABLE_TIME_WARNING && !opts.ignore_warning) {
        const x = res / LOGIC_SPF;
        if (Math.abs(Math.round(x) - x) > 0.01) {
            console.warn(`Warning: ${where}: unaccurate time (${raw})`);
        }
    }
    return res;
}

function unit2time(unit: string, where: string): float {
    switch (unit) {
        case 'F':
            return 1 / FPS;
        case 's':
        case '':
            return 1;
        case 'm':
        case 'min':
            return 60;
        case 'h':
        case 'hr':
            return 3600;
        case 'ms':
            return 0.001;
        default:
            throw new Error(`${where}: invalid time`);
    }
}

function str2time(str: string, where: string): float {
    const match = RE_TIME.exec(str);
    if (!match) {
        throw new Error(`${where}: invalid time`);
    }
    const num = Number.parseFloat(match[1]!);
    const unit = unit2time(match[2]!, where);
    return num * unit;
}

export function parseTimeArray(
    raw: ReadonlyArray<float | string>,
    where: string,
    opts: {
        min?: float;
        max?: float;
        len?: float;
        min_len?: float;
        max_len?: float;
        ascend?: boolean;
        descend?: boolean;
        ignore_warning?: boolean;
    } = {},
): ReadonlyArray<float> {
    checkArray(raw, where, opts);

    const res = [];
    for (const [idx, item] of raw.entries()) {
        res.push(
            parseTime(item, `${where}[${idx}]`, {
                min: opts.min,
                max: opts.max,
                ignore_warning: opts.ignore_warning,
            }),
        );
    }
    checkOrder(res, where, { ascend: opts.ascend, descend: opts.descend });
    return res;
}

const RE_TIME_RANGE = /^(\d+(?:\.\d+)*)(|s|m|min|h|hr|ms|F)\-(\d+(?:\.\d+)*)(|s|m|min|h|hr|ms|F)$/;

/** 一个左开右闭区间表示的时间段 */
export function parseTimeRange(
    raw: string | ReadonlyArray<float | string>,
    where: string,
    opts: {
        min?: float;
        max?: float;
        ignore_warning?: boolean;
    } = {},
): readonly [float, float] {
    const rawArray = typeof raw === 'string' ? str2range(raw, where) : raw;
    const res = parseTimeArray(rawArray, where, { ...opts, len: 2 });
    if (res && res[0]! > res[1]!) {
        throw new Error(`${where}: range[0] must < range[1]`);
    }
    return res as [float, float];
}

function str2range(str: string, where: string): [float, float] {
    const match = RE_TIME_RANGE.exec(str);
    if (!match) {
        throw new Error(`${where}: invalid time`);
    }
    const num1 = Number.parseFloat(match[1]!);
    const unit1 = unit2time(match[2]!, where);
    const num2 = Number.parseFloat(match[3]!);
    const unit2 = unit2time(match[4]!, where);
    return [num1 * unit1, num2 * unit2];
}

export function inTimeRange(range: readonly [float, float], time: float): boolean {
    return range[0] <= time && time < range[1];
}

export class TimeFragment {
    public begin: float;
    public end: float;
    public index: int;

    public constructor(begin: float, end: float, index: int) {
        this.begin = begin;
        this.end = end;
        this.index = index;
    }

    public inRange(time: float): boolean {
        return this.begin <= time && time < this.end;
    }

    public static parseArray(
        raw: ReadonlyArray<string | ReadonlyArray<float | string>>,
        where: string,
        opts: {
            duration: float;
            over_duration?: float;
            ignore_warning?: boolean;
        },
    ): ReadonlyArray<TimeFragment> {
        checkArray(raw, where, { min_len: 1 });

        const ranges = [];
        let range_min = Infinity;
        let range_max = -Infinity;
        const range_opts = {
            min: 0,
            max: opts.over_duration || opts.duration,
            ignore_warning: opts.ignore_warning,
        };
        for (const [idx, item] of raw.entries()) {
            const range = parseTimeRange(item, `${where}[${idx}]`, range_opts);
            ranges.push(range);
            range_min = Math.min(range_min, range[0]);
            range_max = Math.max(range_max, range[1]);
        }
        if (range_min !== 0) {
            throw new Error(`${where}: Invalid time fragment (begin)`);
        }
        if (!opts.over_duration) {
            if (range_max !== opts.duration) {
                throw new Error(`${where}: Invalid time fragment (end)`);
            }
        } else {
            if (range_max < opts.duration || range_max > opts.over_duration) {
                throw new Error(`${where}: Invalid time fragment (end)`);
            }
        }

        let frags = [new TimeFragment(range_min, range_max, -1)];
        for (const [idx, range] of ranges.entries()) {
            if (range[0] === range[1]) {
                continue;
            }

            let iter = 0;
            const new_frags = [];

            while (iter < frags.length) {
                const frag = frags[iter]!;
                if (frag.inRange(range[0])) {
                    if (frag.begin < range[0]) {
                        new_frags.push(new TimeFragment(frag.begin, range[0], frag.index));
                    }
                    break;
                } else {
                    new_frags.push(frag);
                    iter += 1;
                }
            }

            new_frags.push(new TimeFragment(range[0], range[1], idx));

            while (iter < frags.length) {
                const frag = frags[iter]!;
                if (inTimeRange(range, frag.end)) {
                    iter += 1;
                } else {
                    if (frag.end > range[1]) {
                        new_frags.push(new TimeFragment(range[1], frag.end, frag.index));
                    }
                    iter += 1;
                    break;
                }
            }

            while (iter < frags.length) {
                new_frags.push(frags[iter]!);
                iter += 1;
            }
            frags = new_frags;
        }

        for (const frag of frags) {
            if (frag.index < 0) {
                throw new Error(`${where}: Invalid time fragment (overlap)`);
            }
        }
        return frags;
    }
}

export type TimelineRangeArgs<VR> =
    | Readonly<Record<string, VR>>
    | ReadonlyArray<{ time: string } & VR>;

export class TimelineRange<V> {
    public fragments: ReadonlyArray<TimeFragment>;
    public values: ReadonlyArray<V>;

    public constructor(fragments: ReadonlyArray<TimeFragment>, values: ReadonlyArray<V>);
    public constructor(
        raw: TimelineRangeArgs<any>,
        where: string,
        fragmentOpts: {
            duration: float;
            over_duration?: float;
        },
        valueOpts: Record<string, any>,
        parseValue: (raw: any, where: string, opts: Record<string, any>) => V,
    );
    public constructor() {
        if (arguments.length > 2) {
            const raw: TimelineRangeArgs<any> = arguments[0];
            const where: string = arguments[1];
            const fragmentOpts = arguments[2];
            const valueOpts: Record<string, any> = arguments[3];
            const parseValue: (raw: any, where: string, opts: Record<string, any>) => V =
                arguments[4];

            if (Array.isArray(raw)) {
                this.fragments = TimeFragment.parseArray(
                    raw.map((x) => x.time),
                    where,
                    fragmentOpts,
                );
                this.values = raw.map((x, i) => parseValue(x, `${where}[${i}]`, valueOpts));
            } else if (typeof raw === 'object' && raw) {
                this.fragments = TimeFragment.parseArray(Object.keys(raw), where, fragmentOpts);
                this.values = Object.entries(raw).map(([k, v]) =>
                    parseValue(v, `${where}[${k}]`, valueOpts),
                );
            } else {
                throw new Error(`${where}: must be an array/object`);
            }
        } else {
            this.fragments = arguments[0];
            this.values = arguments[1];
        }
    }
}

export type TimelinePointArgs<VR> =
    | Readonly<Record<string, VR>>
    | ReadonlyArray<{ time: string } & VR>;

export class TimelinePoint<V> {
    public pairs: ReadonlyArray<[float, V]>;

    public constructor(pairs: ReadonlyArray<[float, V]>);
    public constructor(
        raw: TimelinePointArgs<any>,
        where: string,
        pointOpts: {
            duration: float;
        },
        valueOpts: Record<string, any>,
        parseValue: (raw: any, where: string, opts: Record<string, any>) => V,
    );
    public constructor() {
        if (arguments.length > 2) {
            const raw: TimelinePointArgs<any> = arguments[0];
            const where: string = arguments[1];
            const pointOpts = arguments[2];
            const valueOpts: Record<string, any> = arguments[3];
            const parseValue: (raw: any, where: string, opts: Record<string, any>) => V =
                arguments[4];

            let pairs: Array<[float, V]>;
            if (Array.isArray(raw)) {
                pairs = raw.map((item, idx) => [
                    parseTime(item.time, `${where}[${idx}].time`, { max: pointOpts.duration }),
                    parseValue(item, `${where}[${idx}]`, valueOpts),
                ]);
            } else if (typeof raw === 'object' && raw) {
                pairs = Object.entries(raw).map(([time, val]) => [
                    parseTime(time, `${where}[${time}]`, { max: pointOpts.duration }),
                    parseValue(val, `${where}[${time}]`, valueOpts),
                ]);
            } else {
                throw new Error(`${where}: must be an array/object`);
            }
            pairs.sort((a, b) => a[0] - b[0]);
            this.pairs = pairs;
        } else {
            this.pairs = arguments[0];
        }
    }
}

// export function parseTimeline<R, T extends TimeFragment>(
//     raw: Readonly<Record<string, Omit<R, 'time'>>> | ReadonlyArray<R>,
//     where: string,
//     opts: {
//         min?: float;
//         max?: float;
//         ignore_warning?: boolean;
//     },
//     parse: (raw: R, where: string, opts: Record<string, any>) => T,
// ): {} {
//     const res = [];
//     if (Array.isArray(raw)) {
//         for (const [idx, rawArg] of raw.entries()) {
//             res.push(parse(rawArg, `${where}[${idx}]`, opts));
//         }
//     } else if (typeof raw === 'object' && raw) {
//         for (const [rawtime, rawValue] of Object.entries(raw)) {
//             res.push(parse({ time: rawtime, ...rawValue } as any, `${where}[${rawtime}]`, opts));
//         }
//     } else {
//         throw new Error(`${where}: must be an array/object`);
//     }
//     return res;
// }

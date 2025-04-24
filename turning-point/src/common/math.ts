import { float, parseFloatArray } from './builtin';

export const EPSILON = 1e-6;

export function absDiffEq(a: float, b: float) {
    return Math.abs(a - b) < EPSILON;
}

export function absDiffNe(a: float, b: float) {
    return Math.abs(a - b) > EPSILON;
}

export function parseVec2(
    raw: ReadonlyArray<float>,
    where: string,
    opts: {
        normalized?: boolean;
        min?: float;
        max?: float;
    } = {},
): readonly [number, number] {
    const res = parseFloatArray(raw, where, { len: 2, min: opts.min, max: opts.max }) as [
        number,
        number,
    ];
    if (opts.normalized) {
        const sqrt = res[0] * res[0] + res[1] * res[1];
        if (absDiffNe(sqrt, 1.0)) {
            throw new Error(`${where}: must be normalized`);
        }
    }
    return res;
}

export function parseVec3(
    raw: ReadonlyArray<float>,
    where: string,
    opts: {
        normalized?: boolean;
        positive?: boolean;
    } = {},
): readonly [number, number, number] {
    const res = parseFloatArray(raw, where, { len: 3 }) as [number, number, number];
    if (opts.normalized) {
        const sqrt = res[0] * res[0] + res[1] * res[1] + res[2] * res[2];
        if (absDiffNe(sqrt, 1.0)) {
            throw new Error(`${where}: must be normalized`);
        }
    }
    return res;
}

export function parseQuat(
    raw: ReadonlyArray<float>,
    where: string,
    opts: {
        normalized?: boolean;
    } = {},
): readonly [number, number, number, number] {
    const res = parseFloatArray(raw, where, { len: 4 }) as [number, number, number, number];
    if (opts.normalized) {
        const sqrt = res[0] * res[0] + res[1] * res[1] + res[2] * res[2] + res[3] * res[3];
        if (absDiffNe(sqrt, 1.0)) {
            throw new Error(`${where}: must be normalized`);
        }
    }
    return res;
}

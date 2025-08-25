import { checkArray, float, int, parseFloatArray } from './builtin';

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

const RE_ANGLE = /^([LR])(\d+)$/;

export function parseAngleXZ(
    raw: float | string,
    where: string,
    opts: {
        min?: float;
        max?: float;
    } = {},
) {
    let degree = 0.0;
    if (typeof raw === 'number') {
        degree = raw;
    } else {
        const res = RE_ANGLE.exec(raw);
        if (!res) {
            throw new Error(`${where}: must be a angle`);
        }
        degree = (res[1] === 'L' ? 1 : -1) * Number.parseFloat(res[2]!);
    }

    if (degree < -180 || degree > 180) {
        throw new Error(`${where}: must be in [-180, 180]`);
    }
    if (opts.min !== undefined && degree < opts.min) {
        throw new Error(`${where}: must >= ${opts.min}`);
    }
    if (opts.max !== undefined && degree > opts.max) {
        throw new Error(`${where}: must <= ${opts.max}`);
    }
    return (degree * Math.PI) / 180;
}

export function parseAngleXZRange(
    raw: ReadonlyArray<float | string>,
    where: string,
    opts: {
        min?: float;
        max?: float;
    } = {},
): readonly [float, float] {
    checkArray(raw, where, { len: 2 });
    return [
        parseAngleXZ(raw[0]!, `${where}[0]`, opts),
        parseAngleXZ(raw[1]!, `${where}[1]`, opts),
    ] as [float, float];
}

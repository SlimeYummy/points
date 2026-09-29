import fs from 'node:fs';
import { memoize } from '@formatjs/fast-memoize';
import { OUTPUT_ASSET } from '../common';
import native from './native';

export * from './native';

const loadSkeletonMetaMemoize = memoize(native.loadSkeletonMeta);

export function loadSkeletonMeta(
    path: string,
    withJoints: boolean = false,
    err?: string,
): native.SkeletonMeta {
    try {
        const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.ls-ozz')}`;
        return loadSkeletonMetaMemoize(realPath, withJoints);
    } catch (e) {
        if (err) {
            throw new (Error as any)(err, { cause: e });
        } else {
            throw e;
        }
    }
}

const loadAnimationMetaMemoize = memoize(native.loadAnimationMeta);

export function loadAnimationMeta(path: string, err?: string): native.AnimationMeta {
    try {
        const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.la-ozz')}`;
        return loadAnimationMetaMemoize(realPath);
    } catch (e) {
        if (err) {
            throw new (Error as any)(err, { cause: e });
        } else {
            throw e;
        }
    }
}

const loadRootMotionMetaMemoize = memoize(native.loadRootMotionMeta);

export function loadRootMotionMeta(path: string, err?: string): native.RootMotionMeta {
    try {
        const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.rm-ozz')}`;
        return loadRootMotionMetaMemoize(realPath);
    } catch (e) {
        if (err) {
            throw new (Error as any)(err, { cause: e });
        } else {
            throw e;
        }
    }
}

export function calcRootMotionDistances(
    path: string,
    ranges: { from: number; to: number }[],
): number[] {
    const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.rm-ozz')}`;
    return native.calcRootMotionDistances(realPath, ranges);
}

const loadWeaponControlMetaMemoize = memoize(native.loadWeaponControlMeta);

export function loadWeaponControlMeta(path: string, err?: string): native.WeaponControlMeta {
    try {
        const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.wc-ozz')}`;
        return loadWeaponControlMetaMemoize(realPath);
    } catch (e) {
        if (err) {
            throw new (Error as any)(err, { cause: e });
        } else {
            throw e;
        }
    }
}

const loadHitMotionMetaMemoize = memoize(native.loadHitMotionMeta);

export function loadHitMotionMeta(path: string, err?: string): native.HitMotionMeta {
    try {
        let realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.hm-rkyv')}`;
        if (!fs.existsSync(realPath)) {
            realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.hm-json')}`;
        }
        return loadHitMotionMetaMemoize(realPath);
    } catch (e) {
        if (err) {
            throw new (Error as any)(err, { cause: e });
        } else {
            throw e;
        }
    }
}

const loadShapeKeyMetaMemoize = memoize(native.loadShapeKeyMeta);

export function loadShapeKeyMeta(path: string, err?: string): native.ShapeKeyMeta {
    try {
        const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.sk-ozz')}`;
        return loadShapeKeyMetaMemoize(realPath);
    } catch (e) {
        if (err) {
            throw new (Error as any)(err, { cause: e });
        } else {
            throw e;
        }
    }
}

const loadJointWeightsTableMetaMemoize = memoize(native.loadJointWeightsTableMeta);

export function loadJointWeightsTableMeta(
    path: string,
    err?: string,
): native.JointWeightsTableMeta {
    try {
        const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.lw-json')}`;
        return loadJointWeightsTableMetaMemoize(realPath);
    } catch (e) {
        if (err) {
            throw new (Error as any)(err, { cause: e });
        } else {
            throw e;
        }
    }
}

export function checkJointWeightsTableName(path: string, name: string, err?: string) {
    const meta = loadJointWeightsTableMeta(path, err);
    const index = meta.names[name];
    if (index == null) {
        throw new (Error as any)(err);
    }
}

export function existCharacterPhysics(path: string) {
    const rkyvPath = `${OUTPUT_ASSET}/${path.replace('.*', '.cp-rkyv')}`;
    if (fs.existsSync(rkyvPath)) {
        return true;
    }
    const jsonPath = `${OUTPUT_ASSET}/${path.replace('.*', '.cp-json')}`;
    return fs.existsSync(jsonPath);
}

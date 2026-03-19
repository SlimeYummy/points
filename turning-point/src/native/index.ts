import { memoize } from '@formatjs/fast-memoize';
import fs from 'node:fs';
import { OUTPUT_ASSET } from '../common';
import native from './native';

export * from './native';

const loadSkeletonMetaMemoize = memoize(native.loadSkeletonMeta);

export function loadSkeletonMeta(path: string, withJoints: boolean = false, err?: string) {
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

export function loadAnimationMeta(path: string, err?: string) {
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

export function loadRootMotionMeta(path: string, err?: string) {
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

const loadWeaponMotionMetaMemoize = memoize(native.loadWeaponMotionMeta);

export function loadWeaponMotionMeta(path: string, err?: string) {
    try {
        const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.wm-ozz')}`;
        return loadWeaponMotionMetaMemoize(realPath);
    } catch (e) {
        if (err) {
            throw new (Error as any)(err, { cause: e });
        } else {
            throw e;
        }
    }
}

const loadHitMotionMetaMemoize = memoize(native.loadHitMotionMeta);

export function loadHitMotionMeta(path: string, err?: string) {
    try {
        let realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.hm-rkyv')}`;
        if (fs.existsSync(realPath)) {
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

export function existCharacterPhysics(path: string) {
    const rkyvPath = `${OUTPUT_ASSET}/${path.replace('.*', '.cp-rkyv')}`;
    if (fs.existsSync(rkyvPath)) {
        return true;
    }
    const jsonPath = `${OUTPUT_ASSET}/${path.replace('.*', '.cp-json')}`;
    return fs.existsSync(jsonPath);
}

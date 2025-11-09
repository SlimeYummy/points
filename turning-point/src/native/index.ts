import { memoize } from '@formatjs/fast-memoize';
import { OUTPUT_ASSET } from '../common';
import native from './native';

export * from './native';

const loadSkeletonMetaMemoize = memoize(native.loadSkeletonMeta);

export function loadSkeletonMeta(path: string, withJoints: boolean = false) {
    const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.ls-ozz')}`;
    return loadSkeletonMetaMemoize(realPath, withJoints);
}

const loadAnimationMetaMemoize = memoize(native.loadAnimationMeta);

export function loadAnimationMeta(path: string) {
    const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.la-ozz')}`;
    return loadAnimationMetaMemoize(realPath);
}

const loadRootMotionMetaMemoize = memoize(native.loadRootMotionMeta);

export function loadRootMotionMeta(path: string) {
    const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.rm-ozz')}`;
    return loadRootMotionMetaMemoize(realPath);
}

const loadWeaponTrajectoryMetaMemoize = memoize(native.loadWeaponTrajectoryMeta);

export function loadWeaponTrajectoryMeta(path: string) {
    const realPath = `${OUTPUT_ASSET}/${path.replace('.*', '.wm-ozz')}`;
    return loadWeaponTrajectoryMetaMemoize(realPath);
}

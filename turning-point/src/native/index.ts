import memoize from 'fast-memoize';
import { ASSET_PATH } from '../common';
import native from './native';

export * from './native';

const loadSkeletonMetaMemoize = memoize(native.loadSkeletonMeta);

export function loadSkeletonMeta(path: string, withJoints: boolean = false) {
    const realPath = `${ASSET_PATH}/${path}.logic-skel.ozz`;
    return loadSkeletonMetaMemoize(realPath, withJoints);
}

const loadAnimationMetaMemoize = memoize(native.loadAnimationMeta);

export function loadAnimationMeta(path: string) {
    const realPath = `${ASSET_PATH}/${path}.logic-anim.ozz`;
    return loadAnimationMetaMemoize(realPath);
}

const loadRootMotionMetaMemoize = memoize(native.loadRootMotionMeta);

export function loadRootMotionMeta(path: string) {
    const realPath = `${ASSET_PATH}/${path}.logic-moti.ozz`;
    return loadRootMotionMetaMemoize(realPath);
}

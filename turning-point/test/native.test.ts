import { describe, it } from '@jest/globals';
import * as native from '../src/native';

describe('native', () => {
    it('sum', () => {
        console.log(native.loadSkeletonMeta('girl', true));
        console.log(native.loadAnimationMeta('girl_run'));
        console.log(native.loadRootMotionMeta('girl_run'));
    });
});

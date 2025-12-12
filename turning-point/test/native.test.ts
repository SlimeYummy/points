import { describe, it } from '@jest/globals';
import * as native from '../src/native';

describe('native', () => {
    it('rust', () => {
        console.log(native.loadSkeletonMeta('Girl.*', true));
        console.log(native.loadAnimationMeta('Girl_Run_Empty.*'));
        console.log(native.loadRootMotionMeta('Girl_Run_Empty.*'));
    });
});

import { Asset, FORCE_GEN, Resource } from '../src';

if (!FORCE_GEN) {
    // Asset.enableIncrement();
}

Asset.copyFiles('../test-asset/', (_dir, file) => {
    return ['Girl.body.json', 'TestZone.json', 'Demo1Zone.json'].includes(file) ? file : null;
});

const MAPPING_VRM_HUMAN = {
    logicFile: 'mapping_vrm_human_logic.json',
    viewFile: 'mapping_vrm_human_view.json',
};

Asset.gltf2ozz('', '', [['GirlLocomotion.glb', 'config_vrm_human.json', MAPPING_VRM_HUMAN, null, 'Girl']]);
Asset.gltf2ozz('', '', [
    ['GirlAttack.glb', 'config_vrm_human.json', MAPPING_VRM_HUMAN, 'GirlAttack', 'Girl'],
]);

console.log('\nGenerate assets done\n');

import './instance';
import './template';
import './verify';

const extra = ['Aaa', 'Bbb', 'Ccc', 'Ddd', 'Eee', 'Fff', 'Ggg', 'Xxx', 'Yyy', 'Zzz', 'Empty'];

declare const __dirname: string;
Resource.write(`${__dirname}/../../test-tmp/test-template`, extra);

console.log('\nGenerate templates done\n');

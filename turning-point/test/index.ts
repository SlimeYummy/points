import { Asset, FORCE_GEN, Resource } from '../src';

if (!FORCE_GEN) {
    // Asset.enableIncrement();
}

const files = ['GirlBody.json', 'TrainingDummyBody.json', 'TestZone.json', 'Demo1Zone.json'];
Asset.copyFiles('../test-asset/', (_dir, file) => {
    if (files.includes(file)) {
        return file;
    } else if (file.endsWith('.hm-json')) {
        return file;
    } else if (file.endsWith('.cp-json')) {
        return file;
    }
    return null;
});

const MAPPING_VRM_HUMAN = {
    logicFile: 'mapping_vrm_human_logic.json',
    viewFile: 'mapping_vrm_human_view.json',
};

Asset.gltf2ozz('', '', [['GirlLocomotion.glb', 'config_vrm_human.json', MAPPING_VRM_HUMAN, null, 'Girl']]);
Asset.gltf2ozz('', '', [
    ['GirlAttack.glb', 'config_vrm_human.json', MAPPING_VRM_HUMAN, 'GirlAttack', 'Girl'],
]);

const MAPPING_SIMPLE = {
    logicFile: 'mapping_simple.json',
    viewFile: 'mapping_simple.json',
}

Asset.gltf2ozz('', '', [
    ['TrainingDummy.glb', 'config_simple.json', MAPPING_SIMPLE, 'TrainingDummy', 'TrainingDummy'],
]);

console.log('\nGenerate assets done\n');

import './instance';
import './template';
import './verify';

const extra = ['Aaa', 'Bbb', 'Ccc', 'Ddd', 'Eee', 'Fff', 'Ggg', 'Xxx', 'Yyy', 'Zzz', 'Empty'];

declare const __dirname: string;
Resource.write(`${__dirname}/../../test-tmp/test-template`, extra);

console.log('\nGenerate templates done\n');

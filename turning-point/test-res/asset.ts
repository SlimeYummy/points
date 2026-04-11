import { Asset, FORCE_GEN } from '../src';

if (!FORCE_GEN) {
    // Asset.enableIncrement();
}

const zones = ['/Zones'];
Asset.copyFiles('../test-asset/', (dir, file) => {
    if (zones.includes(dir)) {
        return `${dir}/${file}`;
    } else if (file.endsWith('.hm-json') || file.endsWith('.cp-json')) {
        return `${dir}/${file}`;
    }
    return null;
});

const MAPPING_VRM_HUMAN = {
    configFile: 'config_vrm_human.json',
    logicFile: 'mapping_vrm_human_logic.json',
    viewFile: 'mapping_vrm_human_view.json',
};

Asset.gltf2ozz('Girl/GirlLocomotion.glb', null, MAPPING_VRM_HUMAN, 'Girl', 'Girl/');
Asset.gltf2ozz('Girl/GirlAttack.glb', 'Girl/', MAPPING_VRM_HUMAN, 'Girl', 'Girl/');

const MAPPING_SIMPLE = {
    configFile: 'config_simple.json',
    logicFile: 'mapping_simple.json',
    viewFile: 'mapping_simple.json',
}

Asset.gltf2ozz('TrainingDummy/TrainingDummy.glb', 'TrainingDummy/', MAPPING_SIMPLE, 'TrainingDummy', 'TrainingDummy/');
Asset.gltf2ozz('Slime/Slime.glb', 'Slime/', MAPPING_SIMPLE, 'Slime', 'Slime/');

console.log('\nGenerate assets done\n');

import {
    ActionGeneral,
    ActionIdle,
    ActionMove,
    Attack1,
    Attack2,
    Capsule,
    Character,
    LEVEL_ACTION,
    LEVEL_ATTACK,
    Resource,
    Style,
    Zone,
} from '../src';

new Zone('Zone.Demo', {
    name: 'Demo',
    zone_file: 'stage-demo.json',
    view_zone_file: 'stage-demo.tscn',
});

const ONE = new Character('Character.DemoGirl', {
    name: 'Character One',
    level: [1, 6],
    styles: ['Style.DemoGirl/1'],
    equipments: [],
    bounding_capsule: new Capsule(0.5 * 1.0, 0.3),
    skeleton_files: 'girl',
    skeleton_toward: [0, 1],
});

const fixed_attributes = {
    damage_reduce_param_1: 0.05,
    damage_reduce_param_2: 100,
    guard_damage_ratio_1: 0.8,
    deposture_reduce_param_1: 0.05,
    deposture_reduce_param_2: 200,
    guard_deposture_ratio_1: 0.8,
    weak_damage_up: 0.25,
};

new Style('Style.DemoGirl/1', {
    name: 'Character Girl Type-1',
    character: ONE.id,
    attributes: {
        MaxHealth: [400, 550, 700, 850, 1000, 1200],
        MaxPosture: [100, 115, 130, 145, 160, 180],
        PostureRecovery: [10, 11, 12, 13, 14, 15],
        PhysicalAttack: [10, 15, 20, 25, 30, 35],
        PhysicalDefense: [15, 20, 25, 30, 35, 40],
        ElementalAttack: [8, 12, 16, 20, 24, 28],
        ElementalDefense: [10, 15, 20, 25, 30, 35],
        ArcaneAttack: [9, 13, 17, 21, 25, 30],
        ArcaneDefense: [5, 8, 11, 14, 17, 20],
        CriticalChance: ['10%', '10%', '10%', '10%', '10%', '10%'],
        CriticalDamage: ['30%', '30%', '30%', '30%', '30%', '30%'],
    },
    slots: [],
    fixed_attributes,
    perks: [],
    usable_perks: [],
    actions: [
        'Action.DemoGirl.Idle',
        'Action.DemoGirl.Run',
        'Action.DemoGirl.Attack1/1',
        'Action.DemoGirl.Attack2/1',
    ],
    view_model: 'StyleOne-1.vrm',
});

new ActionIdle('Action.DemoGirl.Idle', {
    character: ONE.id,
    styles: ONE.styles,
    anim_idle: {
        files: 'girl_stand_idle',
        duration: '3s',
    },
    anim_ready: {
        files: 'girl_stand_ready',
        duration: '2s',
    },
});

new ActionMove('Action.DemoGirl.Run', {
    character: ONE.id,
    styles: ONE.styles,
    anim_move: {
        files: 'girl_run',
        duration: '0.9s',
        fade_in: '0.2s',
    },
    yam_time: '0.15s',
    turn_time: '0.3s',
});

new ActionGeneral('Action.DemoGirl.Attack1/1', {
    anim_main: {
        files: 'girl_attack1_1',
        duration: '1.3s',
        root_motion: true,
    },
    character: ONE.id,
    styles: ['Style.DemoGirl/1'],
    enter_key: Attack1,
    enter_level: LEVEL_ATTACK,
    motion_distance: [0.3, 0.5],
    motion_toward: 60,
    attributes: {
        '0-1.3s': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        },
    },
    derive_levels: {
        '0-1.3s': LEVEL_ACTION,
        '1.15s-1.3s': LEVEL_ATTACK,
    },
});

new ActionGeneral('Action.DemoGirl.Attack2/1', {
    anim_main: {
        files: 'girl_attack2_1',
        duration: '2.2s',
        root_motion: true,
    },
    character: ONE.id,
    styles: ['Style.DemoGirl/1'],
    enter_key: Attack2,
    enter_level: LEVEL_ATTACK,
    attributes: {
        '0-2.2s': {
            damage_rdc: '40%',
            shield_dmg_rdc: 0,
            poise_level: 2,
        },
    },
    derive_levels: {
        '0-2.2s': LEVEL_ACTION,
        '2.1s-2.2s': LEVEL_ATTACK,
    },
});

declare const __dirname: string;
Resource.write(`${__dirname}/../../test-tmp/demo-template`);

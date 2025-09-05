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
    Run,
    Style,
    Zone,
} from '../src';

new Zone('Zone.Demo', {
    name: 'Demo',
    zone_file: 'demo_zone.json',
    view_zone_file: 'DemoZone.unity',
});

const ONE = new Character('Character.DemoGirl', {
    name: 'Character One',
    level: [1, 6],
    styles: ['Style.DemoGirl/1'],
    equipments: [],
    bounding_capsule: new Capsule(0.5 * 1.0, 0.3),
    skeleton_files: 'girl.*',
    skeleton_toward: [0, 1],
    body_file: 'body1.json',
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
        'Action.DemoGirl.Jog',
        'Action.DemoGirl.Attack1/1',
        'Action.DemoGirl.Attack2/1',
    ],
    view_model: 'StyleOne-1.vrm',
});

new ActionIdle('Action.DemoGirl.Idle', {
    character: ONE.id,
    styles: ONE.styles,
    anim_idle: {
        files: 'girl_idle.*',
    },
});

new ActionMove('Action.DemoGirl.Jog', {
    character: ONE.id,
    styles: ONE.styles,
    enter_key: Run,
    anim_move: {
        files: 'girl_jog.*',
        fade_in: '4F',
        root_motion: true,
    },
    move_speed: 3,
    starts: [
        {
            enter_angle: ['L15', 'R15'],
            files: 'girl_jog_start.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '2F',
            quick_stop_end: '20F',
        },
        {
            enter_angle: ['L15', 'L60'],
            files: 'girl_jog_start_turn_l45.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '4F',
            quick_stop_end: '22F',
        },
        {
            enter_angle: ['R15', 'R60'],
            files: 'girl_jog_start_turn_l45.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '4F',
            quick_stop_end: '22F',
        },
        {
            enter_angle: ['L60', 'L120'],
            files: 'girl_jog_start_turn_l90.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '5F',
            quick_stop_end: '23F',
        },
        {
            enter_angle: ['R60', 'R120'],
            files: 'girl_jog_start_turn_r90.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '5F',
            quick_stop_end: '23F',
        },
        {
            enter_angle: ['L120', 'L180'],
            files: 'girl_jog_start_turn_l180.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '6F',
            quick_stop_end: '24F',
        },
        {
            enter_angle: ['R120', 'R180'],
            files: 'girl_jog_start_turn_r180.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '6F',
            quick_stop_end: '24F',
        },
    ],
    turn_time: '12F',
    stops: [
        {
            enter_phase_table: [[0.75, 0.25, '2F']],
            files: 'girl_jog_stop_l.*',
            fade_in: '4F',
            root_motion: true,
            speed_down_end: '12F',
        },
        {
            enter_phase_table: [[0.25, 0.75, '2F']],
            files: 'girl_jog_stop_r.*',
            fade_in: '4F',
            root_motion: true,
            speed_down_end: '12F',
        },
    ],
    quick_stop_time: 0,
});

new ActionGeneral('Action.DemoGirl.Attack1/1', {
    anim_main: {
        files: 'girl_attack1_1.*',
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
        files: 'girl_attack2_1.*',
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
console.log('\nGenerate templates done\n');

import {
    ActionGeneral,
    ActionIdle,
    ActionMove,
    Attack1,
    Attack2,
    Character,
    LEVEL_ACTION,
    LEVEL_ATTACK,
    Resource,
    Run,
    Style,
    TaperedCapsule,
    Walk,
    Zone,
} from '../src';

new Zone('Zone.Demo', {
    name: 'Demo',
    zone_file: 'TestZone.json',
    view_zone_file: '.unity',
});

const ONE = new Character('Character.Demo', {
    name: 'Character One',
    level: [1, 6],
    styles: ['Style.Demo^1'],
    equipments: [],
    bounding: new TaperedCapsule(0.6, 0.3, 0.1),
    skeleton_files: 'Girl.*',
    skeleton_toward: [0, 1],
    body_file: 'Girl.body.json',
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

new Style('Style.Demo^1', {
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
        'Action.Demo.Idle',
        'Action.Demo.Run',
        'Action.Demo.Walk',
        'Action.Demo.Attack1',
        'Action.Demo.Attack2',
        'Action.Demo.Attack3',
        'Action.Demo.Attack4',
    ],
    view_model: 'StyleOne-1.vrm',
});

new ActionIdle('Action.Demo.Idle', {
    character: ONE.id,
    styles: ONE.styles,
    tags: ['Idle'],
    anim_idle: {
        files: 'Girl_Idle_Empty.*',
    },
});

new ActionMove('Action.Demo.Run', {
    character: ONE.id,
    styles: ONE.styles,
    tags: ['Run'],
    enter_key: Run,
    anim_move: {
        files: 'Girl_Run_Empty.*',
        fade_in: '4F',
        root_motion: true,
    },
    move_speed: 6,
    starts: [
        {
            enter_angle: ['L30', 'R30'],
            files: 'Girl_RunStart_Empty.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '4F',
            quick_stop_end: '22F',
        },
        {
            enter_angle: ['L30', 'L105'],
            files: 'Girl_RunStart_L90_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '6F',
            quick_stop_end: '24F',
        },
        {
            enter_angle: ['R30', 'R105'],
            files: 'Girl_RunStart_R90_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '6F',
            quick_stop_end: '24F',
        },
        {
            enter_angle: ['L105', 'L180'],
            files: 'Girl_RunStart_L180_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '8F',
            quick_stop_end: '26F',
        },
        {
            enter_angle: ['R105', 'R180'],
            files: 'Girl_RunStart_R180_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '8F',
            quick_stop_end: '26F',
        },
    ],
    turn_time: '12F',
    stops: [
        {
            enter_phase_table: [{ phase: [0.75, 0.25], offset: '2F' }],
            files: 'Girl_RunStop_l_Empty.*',
            fade_in: '4F',
            root_motion: true,
            leave_phase_table: [
                ['0F', 0.0],
                ['14F', 0.5],
                ['34F', 0.8],
            ],
        },
        {
            enter_phase_table: [{ phase: [0.25, 0.75], offset: '2F' }],
            files: 'Girl_RunStop_r_Empty.*',
            fade_in: '4F',
            root_motion: true,
            leave_phase_table: [
                ['0F', 0.5],
                ['14F', 0.0],
                ['34F', 0.3],
            ],
        },
    ],
    quick_stop_time: 0,
});

new ActionMove('Action.Demo.Walk', {
    character: ONE.id,
    styles: ONE.styles,
    tags: ['Walk'],
    enter_key: Walk,
    anim_move: {
        files: 'Girl_Walk_Empty.*',
        fade_in: '4F',
        root_motion: true,
    },
    move_speed: 3,
    starts: [
        {
            enter_angle: ['L30', 'R30'],
            files: 'Girl_WalkStart_Empty.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '6F',
            quick_stop_end: '22F',
        },
        {
            enter_angle: ['L30', 'L105'],
            files: 'Girl_WalkStart_L90_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '8F',
            quick_stop_end: '24F',
        },
        {
            enter_angle: ['R30', 'R105'],
            files: 'Girl_WalkStart_R90_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '8F',
            quick_stop_end: '24F',
        },
        {
            enter_angle: ['L105', 'L180'],
            files: 'Girl_WalkStart_L180_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '10F',
            quick_stop_end: '26F',
        },
        {
            enter_angle: ['R105', 'R180'],
            files: 'Girl_WalkStart_R180_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '10F',
            quick_stop_end: '26F',
        },
    ],
    turn_time: '16F',
    stops: [
        {
            enter_phase_table: [
                { phase: [0.83, 0.02], offset: '0F' },
                { phase: [0.02, 0.08], offset: '2F' },
            ],
            files: 'Girl_WalkStop_1_Empty.*',
            fade_in: '6F',
            root_motion: true,
        },
        {
            enter_phase_table: [
                { phase: [0.08, 0.27], offset: '0F' },
                { phase: [0.27, 0.33], offset: '2F' },
            ],
            files: 'Girl_WalkStop_2_Empty.*',
            fade_in: '6F',
            root_motion: true,
        },
        {
            enter_phase_table: [
                { phase: [0.33, 0.52], offset: '0F' },
                { phase: [0.52, 0.58], offset: '2F' },
            ],
            files: 'Girl_WalkStop_3_Empty.*',
            fade_in: '6F',
            root_motion: true,
        },
        {
            enter_phase_table: [
                { phase: [0.58, 0.77], offset: '0F' },
                { phase: [0.77, 0.83], offset: '2F' },
            ],
            files: 'Girl_WalkStop_4_Empty.*',
            fade_in: '6F',
            root_motion: true,
        },
    ],
    quick_stop_time: 0,
});

new ActionGeneral('Action.Demo.Attack1', {
    anim_main: {
        files: 'Girl_Attack_01A.*',
        duration: '168F',
        root_motion: true,
        weapon_motion: true,
    },
    character: ONE.id,
    styles: ONE.styles,
    tags: ['Attack'],
    enter_key: Attack1,
    enter_level: LEVEL_ATTACK,
    input_movements: [
        { time: '0F', duration: '12F', angle: 45 },
        { time: '48F', duration: '16F', angle: 45 },
        { time: '48F', move_ex: true },
    ],
    attributes: {
        '0-168F': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        }
    },
    derive_levels: {
        '0-124F': LEVEL_ACTION,
        '124F-168F': LEVEL_ATTACK,
    },
    derives: [
        { key: Attack1, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack3' },
        { key: Attack2, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack4' },
    ],
    custom_events: {
        '70F': 'SE_Slash',
        '68F': 'VFX_Slash'
    }
});

new ActionGeneral('Action.Demo.Attack2', {
    anim_main: {
        files: 'Girl_Attack_02A.*',
        duration: '168F',
        root_motion: true,
        weapon_motion: true,
    },
    character: ONE.id,
    styles: ONE.styles,
    tags: ['Attack'],
    enter_key: Attack2,
    enter_level: LEVEL_ATTACK,
    input_movements: [
        { time: '0F', duration: '12F', angle: 45 },
        { time: '48F', duration: '16F', angle: 45 },
        { time: '48F', move_ex: true },
    ],
    attributes: {
        '0-168F': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        }
    },
    derive_levels: {
        '0-124F': LEVEL_ACTION,
        '130F-168F': LEVEL_ATTACK,
    },
    derives: [
        { key: Attack1, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack3' },
        { key: Attack2, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack4' },
    ],
    custom_events: {
        '70F': 'SE_Slash',
        '68F': 'VFX_Slash'
    }
});

new ActionGeneral('Action.Demo.Attack3', {
    anim_main: {
        files: 'Girl_Attack_03A.*',
        duration: '170F',
        root_motion: true,
        weapon_motion: true,
    },
    character: ONE.id,
    styles: ONE.styles,
    tags: ['Attack'],
    enter_level: LEVEL_ATTACK,
    input_movements: {
        '0F': { duration: '12F', angle: 45 }
    },
    attributes: {
        '0-170F': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        }
    },
    derive_levels: {
        '0-130F': LEVEL_ACTION,
        '130F-170F': LEVEL_ATTACK,
    },
    derives: [
        { key: Attack1, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack1' },
        { key: Attack2, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack2' },
    ],
    custom_events: {
        '50F': 'SE_Slash',
        '48F': 'VFX_Slash'
    }
});

new ActionGeneral('Action.Demo.Attack4', {
    anim_main: {
        files: 'Girl_Attack_04A.*',
        duration: '170F',
        root_motion: true,
        weapon_motion: true,
    },
    character: ONE.id,
    styles: ONE.styles,
    tags: ['Attack'],
    enter_level: LEVEL_ATTACK,
    input_movements: {
        '0F': { duration: '12F', angle: 45 }
    },
    attributes: {
        '0-170F': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        }
    },
    derive_levels: {
        '0-124F': LEVEL_ACTION,
        '124F-170F': LEVEL_ATTACK,
    },
    derives: [
        { key: Attack1, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack1' },
        { key: Attack2, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack2' },
    ],
    custom_events: {
        '50F': 'SE_Slash',
        '48F': 'VFX_Slash'
    }
});

declare const __dirname: string;
Resource.write(`${__dirname}/../../test-tmp/demo-template`);
console.log('\nGenerate templates done\n');

import {
    ActionDodge,
    ActionGeneral,
    ActionHit,
    ActionIdle,
    ActionMoveFree,
    ActionMoveFreeStop2Args,
    Attack1,
    Attack2,
    Attack5,
    Character,
    Dodge,
    Hit1,
    LEVEL_ACTION,
    LEVEL_ATTACK,
    LEVEL_DERIVE,
    LEVEL_IDLE,
    LEVEL_MOVE,
    Run,
    Style,
    Walk,
} from '../src';

const PLAYER = new Character('Character.Demo', {
    name: 'Character One',
    level: [1, 6],
    styles: ['Style.Demo^1'],
    equipments: [],
    skeleton_files: 'Girl/Girl.*',
    joint_weights_table: true,
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
    character: PLAYER.id,
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
        'Action.Demo.Dodge',
        'Action.Demo.Attack1',
        'Action.Demo.Attack2',
        'Action.Demo.Attack3',
        'Action.Demo.Attack4',
        'Action.Demo.Hit1',
        'Action.Demo.Greet',
    ],
    view_model: 'StyleOne-1.vrm',
});

new ActionIdle('Action.Demo.Idle', {
    character: PLAYER.id,
    styles: PLAYER.styles,
    tags: ['Idle'],
    anim_idle: {
        files: 'Girl/Idle_Empty.*',
    },
});

function makeStopItem(enter_phase: [number, number], stop_files: string): ActionMoveFreeStop2Args {
    return {
        enter_phase_table: [enter_phase],
        prev_anim_time: ['6F', '12F'],
        anim_stop: {
            files: stop_files,
            fade_in: '4F',
            root_motion: true,
        },
        no_arm_fade_in: '6F',
        arm_fade_out: '14F',
        anim_arm_additive: {
            files: 'Girl/Run_Stop_Add_Empty.*',
            additive_blending: true,
        },
        arm_additive_fade_inout: ['12F', '20F'],
        leave_phase_table: [
            ['0F', 0.0],
            ['14F', 0.5],
            ['34F', 0.8],
        ],
    };
}

new ActionMoveFree('Action.Demo.Run', {
    character: PLAYER.id,
    styles: PLAYER.styles,
    tags: ['Run'],
    enter_key: Run,
    anim_move: {
        files: 'Girl/Run_Empty.*',
        fade_in: '4F',
        root_motion: true,
    },
    move_speed: 3,
    anim_starts: [
        {
            enter_angle: ['L30', 'R30'],
            files: 'Girl/Run_Start_Empty.*',
            fade_in: '4F',
            root_motion: true,
            turn_in_place_end: '4F',
            quick_stop_end: '22F',
        },
        {
            enter_angle: ['L30', 'L105'],
            files: 'Girl/Run_Start_L90_Empty.*',
            fade_in: '4F',
            root_motion: true,
            turn_in_place_end: '6F',
            quick_stop_end: '24F',
        },
        {
            enter_angle: ['R30', 'R105'],
            files: 'Girl/Run_Start_R90_Empty.*',
            fade_in: '4F',
            root_motion: true,
            turn_in_place_end: '6F',
            quick_stop_end: '24F',
        },
        {
            enter_angle: ['L105', 'L180'],
            files: 'Girl/Run_Start_L180_Empty.*',
            fade_in: '4F',
            root_motion: true,
            turn_in_place_end: '8F',
            quick_stop_end: '26F',
        },
        {
            enter_angle: ['R105', 'R180'],
            files: 'Girl/Run_Start_R180_Empty.*',
            fade_in: '4F',
            root_motion: true,
            turn_in_place_end: '8F',
            quick_stop_end: '26F',
        },
    ],
    turn_time: '12F',
    anim_stops: [
        makeStopItem([0.2, 0.5], 'Girl/Run_Stop_R1_Empty.*'),
        makeStopItem([0.5, 0.7], 'Girl/Run_Stop_R2_Empty.*'),
        makeStopItem([0.7, 1.0], 'Girl/Run_Stop_L1_Empty.*'),
        makeStopItem([0.0, 0.2], 'Girl/Run_Stop_L2_Empty.*'),
    ],
    quick_stop_time: 0,
    smooth_move_froms: ['Action.Demo.Run', 'Action.Demo.Walk'],
});

new ActionMoveFree('Action.Demo.Walk', {
    character: PLAYER.id,
    styles: PLAYER.styles,
    tags: ['Walk'],
    enter_key: Walk,
    anim_move: {
        files: 'Girl/Walk_Empty.*',
        fade_in: '4F',
        root_motion: true,
    },
    move_speed: 3,
    anim_starts: [
        {
            enter_angle: ['L30', 'R30'],
            files: 'Girl/Walk_Start_Empty.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '6F',
            quick_stop_end: '20F',
        },
        {
            enter_angle: ['L30', 'L105'],
            files: 'Girl/Walk_Start_L90_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '10F',
            quick_stop_end: '28F',
        },
        {
            enter_angle: ['R30', 'R105'],
            files: 'Girl/Walk_Start_R90_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '10F',
            quick_stop_end: '28F',
        },
        {
            enter_angle: ['L105', 'L180'],
            files: 'Girl/Walk_Start_L180_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '14F',
            quick_stop_end: '32F',
        },
        {
            enter_angle: ['R105', 'R180'],
            files: 'Girl/Walk_Start_R180_Empty.*',
            fade_in: '2F',
            root_motion: true,
            turn_in_place_end: '14F',
            quick_stop_end: '32F',
        },
    ],
    turn_time: '16F',
    anim_stops: [
        {
            enter_phase_table: [[0.75, 0.2]],
            files: 'Girl/Walk_Stop_1_Empty.*',
            duration: '22F',
            fade_in: '16F',
            root_motion: true,
        },
        {
            enter_phase_table: [[0.2, 0.4], [0.55, 0.75]],
            files: 'Girl/Walk_Stop_2_Empty.*',
            duration: '22F',
            fade_in: '16F',
            root_motion: true,
        },
        {
            enter_phase_table: [[0.4, 0.55]],
            files: 'Girl/Walk_Stop_2_Empty.*',
            duration: '24F!', // Adjust animation playback scale
            fade_in: '16F',
            root_motion: true,
        },
    ],
    quick_stop_time: 0,
    smooth_move_froms: ['Action.Demo.Run', 'Action.Demo.Walk'],
});

new ActionDodge('Action.Demo.Dodge', {
    character: PLAYER.id,
    tags: ['Dodge'],
    styles: PLAYER.styles,
    enter_key: Dodge,
    enter_level: LEVEL_DERIVE,
    anim_dodge: {
        files: 'Girl/Dodge_F_Empty.*',
        root_motion: true,
    },
    smooth_move_froms: ['Action.Demo.Run'],
    smooth_no_leg_fade_in: '8F',
    smooth_speed_duration: '10F',
    dodge_time: ['6F', '24F'],
    derive_times: {
        quick: '34F',
        normal: '48F',
    },
});

new ActionGeneral('Action.Demo.Attack1', {
    anim_main: {
        files: 'Girl/Attack_01A.*',
        duration: '160F',
        root_motion: true,
        weapon_control: true,
        hit_motion: true,
    },
    character: PLAYER.id,
    styles: PLAYER.styles,
    tags: ['Attack'],
    enter_key: Attack1,
    enter_level: LEVEL_ATTACK,
    input_movements: [
        { time: '0F', duration: '12F', max_angle: 45 },
        { time: '52F', duration: '16F', max_angle: 45 },
        { time: '52F', move: true, move_ex: true },
    ],
    attributes: {
        '0-160F': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        },
    },
    keep_levels: {
        '0-124F': LEVEL_ACTION,
        '124F-160F': LEVEL_ATTACK,
    },
    derives: [
        { key: Attack1, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack3' },
        { key: Attack2, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack4' },
    ],
    hits: [
        { group: 'Axe', box_max_times: 1 },
    ],
});

new ActionGeneral('Action.Demo.Attack2', {
    anim_main: {
        files: 'Girl/Attack_02A.*',
        duration: '160F',
        root_motion: true,
        weapon_control: true,
        hit_motion: true,
    },
    character: PLAYER.id,
    styles: PLAYER.styles,
    tags: ['Attack'],
    enter_key: Attack2,
    enter_level: LEVEL_ATTACK,
    input_movements: [
        { time: '0F', duration: '12F', max_angle: 45 },
        { time: '52F', duration: '16F', max_angle: 45 },
        { time: '52F', move: true, move_ex: true },
    ],
    attributes: {
        '0-160F': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        },
    },
    keep_levels: {
        '0-124F': LEVEL_ACTION,
        '124F-160F': LEVEL_ATTACK,
    },
    derives: [
        { key: Attack1, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack3' },
        { key: Attack2, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack4' },
    ],
    hits: [
        { group: 'Axe', box_max_times: 1 },
    ],
});

new ActionGeneral('Action.Demo.Attack3', {
    anim_main: {
        files: 'Girl/Attack_03A.*',
        duration: '166F',
        root_motion: true,
        weapon_control: true,
        hit_motion: true,
    },
    character: PLAYER.id,
    styles: PLAYER.styles,
    tags: ['Attack'],
    enter_level: LEVEL_ATTACK,
    input_movements: [
        { time: '0F', duration: '12F', max_angle: 45 },
        { time: '48F', duration: '16F', max_angle: 45 },
        { time: '48F', move: true, move_ex: true },
    ],
    attributes: {
        '0-166F': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        },
    },
    keep_levels: {
        '0-130F': LEVEL_ACTION,
        '130F-166F': LEVEL_ATTACK,
    },
    derives: [
        { key: Attack1, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack1' },
        { key: Attack2, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack2' },
    ],
    hits: [
        { group: 'Axe', box_max_times: 1 },
    ],
});

new ActionGeneral('Action.Demo.Attack4', {
    anim_main: {
        files: 'Girl/Attack_04A.*',
        duration: '166F',
        root_motion: true,
        weapon_control: true,
        hit_motion: true,
    },
    character: PLAYER.id,
    styles: PLAYER.styles,
    tags: ['Attack'],
    enter_level: LEVEL_ATTACK,
    input_movements: [
        { time: '0F', duration: '12F', max_angle: 45 },
        { time: '48F', duration: '16F', max_angle: 45 },
        { time: '48F', move: true, move_ex: true },
    ],
    attributes: {
        '0-166F': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        },
    },
    keep_levels: {
        '0-130F': LEVEL_ACTION,
        '130F-166F': LEVEL_ATTACK,
    },
    hits: [
        {
            group: 'Axe',
            box_max_times: 1,
        },
    ],
    derives: [
        { key: Attack1, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack1' },
        { key: Attack2, level: LEVEL_ATTACK + 1, action: 'Action.Demo.Attack2' },
    ],
});

new ActionHit('Action.Demo.Hit1', {
    character: PLAYER.id,
    styles: PLAYER.styles,
    tags: ['Hit'],
    enter_key: Hit1,
    anim_be_hits: [
        {
            enter_angle: -90,
            files: 'Girl/Hit1_Empty_R.*',
            fade_in: '6F',
            root_motion: true,
        },
        {
            enter_angle: 0,
            files: 'Girl/Hit1_Empty_F.*',
            fade_in: '6F',
            root_motion: true,
        },
        {
            enter_angle: 90,
            files: 'Girl/Hit1_Empty_L.*',
            fade_in: '6F',
            root_motion: true,
        },
        {
            enter_angle: 180,
            files: 'Girl/Hit1_Empty_B.*',
            fade_in: '6F',
            root_motion: true,
        },
    ],
});

new ActionGeneral('Action.Demo.Greet', {
    anim_main: {
        files: 'Girl/Greet.*',
        duration: '350F',
        root_motion: true,
    },
    character: PLAYER.id,
    styles: PLAYER.styles,
    tags: [],
    enter_key: Attack5,
    enter_level: LEVEL_MOVE,
    attributes: {
        '0-350F': { poise_level: 0 },
    },
    keep_levels: { '0-350F': LEVEL_IDLE },
});

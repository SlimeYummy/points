import {
    ActionDodgeNpc,
    ActionGeneral,
    ActionGeneralNpc,
    ActionHit,
    ActionIdle,
    ActionMoveFree,
    ActionMoveFreeNpc,
    AiBrain,
    AiRoutine,
    AiTaskGeneral,
    AiTaskIdle,
    AiTaskMoveToCharacter,
    AiTaskPatrol,
    AiTaskKeepDistance,
    Attack1,
    Attack2,
    Character,
    CharacterNpc,
    Equipment,
    Hit1,
    LEVEL_ACTION,
    LEVEL_ATTACK,
    Perk,
    Run,
    Slot1,
    Slot3,
    Style,
    Var,
    Walk,
} from '../src';

//
// Player
//

Var.define({
    '#.Action.Instance.AttackDerive^1A': [2, ['Character.Instance^1']],
    '#.Action.Instance.AttackUnused^1A': [1, ['Character.Instance^1']],
    '#.Perk.Instance^1A': [1, ['Style.Instance^1A']],
    '#.Perk.Instance^1B': [3, ['Style.Instance^1A']],
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

new Character('Character.Instance^1', {
    name: 'Character 1',
    level: [1, 6],
    styles: ['Style.Instance^1A'],
    equipments: ['Equipment.Instance^1A', 'Equipment.Instance^1B'],
    skeleton_files: 'Girl/Girl.*',
});

new Style('Style.Instance^1A', {
    name: 'Style 1',
    character: 'Character.Instance^1',
    tags: ['Player'],
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
    slots: ['A2D2', 'A2D2', 'A3D3', 'A3D3S2', 'A5D4S2', 'A5D4S3'],
    fixed_attributes,
    perks: ['Perk.Instance^1A', 'Perk.Instance^1B'],
    actions: [
        'Action.Instance.Idle^1A',
        'Action.Instance.Run^1A',
        'Action.Instance.Attack^1A',
        'Action.Instance.AttackDerive^1A',
        'Action.Instance.AttackUnused^1A',
    ],
    view_model: 'StyleOne-1.vrm',
});

new Equipment('Equipment.Instance^1A', {
    name: 'Weapon 1A',
    character: 'Character.Instance^1',
    slot: Slot1,
    level: [1, 4],
    attributes: {
        PhysicalAttack: [13, 19, 25, 31],
        ElementalAttack: [8, 12, 16, 20],
        ArcaneAttack: [13, 18, 23, 28],
        CriticalChance: ['2%', '3%', '4%', '5%'],
    },
    slots: ['', '', 'A1', 'A1'],
    entries: {
        'Entry.AttackUp': [
            [1, 0],
            [1, 1],
            [1, 2],
            [1, 3],
        ],
    },
});

new Equipment('Equipment.Instance^1B', {
    name: 'Weapon 1B',
    character: 'Character.Instance^1',
    slot: Slot3,
    level: [0, 3],
    attributes: {
        PhysicalAttack: [10, 15, 20, 25],
        ElementalAttack: [7, 10, 13, 16],
        ArcaneAttack: [8, 12, 16, 20],
        CriticalDamage: ['10%', '12%', '15%', '18%'],
    },
    slots: ['A1', 'A2', 'S1A1', 'S1A2'],
    entries: {
        'Entry.DefenseUp': [
            [2, 0],
            [2, 1],
            [2, 2],
            [2, 3],
        ],
    },
});

new Perk('Perk.Instance^1A', {
    name: 'Instance 1A',
    character: 'Character.Instance^1',
    style: 'Style.Instance^1A',
    max_level: 2,
    attributes: {
        AttackUp: ['10%', '15%'],
    },
    entries: {
        'Entry.AttackUp': [
            [1, 0],
            [1, 3],
        ],
        'Entry.DefenseUp': [
            [1, 3],
            [2, 6],
        ],
    },
    var_indexes: {
        '#.Perk.Instance^1A': [0, 1],
        '#.Perk.Instance^1B': [1, 2],
    },
});

new Perk('Perk.Instance^1B', {
    name: 'Instance 1A',
    character: 'Character.Instance^1',
    style: 'Style.Instance^1A',
    max_level: 3,
    slots: ['A1D1', 'S1A1D1', 'S1A2D2'],
    var_indexes: {
        '#.Perk.Instance^1B': [2, 3, 4],
    },
});

new ActionIdle('Action.Instance.Idle^1A', {
    character: 'Character.Instance^1',
    styles: ['Style.Instance^1A'],
    tags: ['Idle'],
    anim_idle: {
        files: 'Girl/Idle_Empty.*',
        duration: '2.5s!',
        fade_in: 0.2,
    },
    anim_ready: {
        files: 'Girl/Idle_Axe.*',
        duration: '2s!',
        fade_in: 0.4,
    },
});

new ActionMoveFree('Action.Instance.Run^1A', {
    character: 'Character.Instance^1',
    styles: ['Style.Instance^1A'],
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
            enter_angle: ['L15', 'R15'],
            files: 'Girl/RunStart_Empty.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '2F',
            quick_stop_end: '20F',
        },
        {
            enter_angle: ['L15', 'L180'],
            files: 'Girl/RunStart_L180_Empty.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '8F',
            quick_stop_end: '26F',
        },
        {
            enter_angle: ['R15', 'R180'],
            files: 'Girl/RunStart_R180_Empty.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '8F',
            quick_stop_end: '26F',
        },
    ],
    turn_time: '10F',
    anim_stops: [
        {
            enter_phase_table: [{ phase: [0.75, 0.25], offset: '2F' }],
            files: 'Girl/RunStop_L_Empty.*',
            fade_in: '4F',
            root_motion: true,
            leave_phase_table: [
                ['0F', 0.0],
                ['14F', 0.5],
            ],
        },
        {
            enter_phase_table: [{ phase: [0.25, 0.75], offset: '2F' }],
            files: 'Girl/RunStop_R_Empty.*',
            fade_in: '4F',
            root_motion: true,
            leave_phase_table: [
                ['0F', 0.5],
                ['14F', 0.0],
            ],
        },
    ],
    quick_stop_time: 0,
});

new ActionGeneral('Action.Instance.Attack^1A', {
    character: 'Character.Instance^1',
    styles: ['Style.Instance^1A'],
    tags: ['Attack'],
    anim_main: {
        files: 'Girl/Attack_Test.*',
        duration: '4s!',
        root_motion: true,
        hit_motion: true,
    },
    enter_key: Attack1,
    enter_level: LEVEL_ATTACK,
    input_movements: {
        '0F': { duration: '8F', max_angle: 60 },
        '24F': { move_ex: true },
    },
    attributes: {
        '0-4s': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        },
    },
    keep_levels: {
        '0-4s': LEVEL_ACTION,
        '2.5s-4.5s': LEVEL_ATTACK,
    },
    derives: [
        {
            key: Attack1,
            level: LEVEL_ATTACK + 1,
            action: 'Action.Instance.AttackDerive^1A',
        },
        {
            key: [Attack2, 'B60'],
            level: LEVEL_ATTACK + 1,
            action: 'Action.Instance.AttackDerive^1A',
        },
    ],
    hits: [
        {
            group: 'Health',
            box_max_times: 2,
            box_min_interval: '1F',
            group_max_times: 4,
        },
        {
            group: 'Counter',
            box_max_times: 1,
        },
    ],
});

new ActionGeneral('Action.Instance.AttackDerive^1A', {
    enabled: ['#.Action.Instance.AttackDerive^1A', [false, false, true]],
    character: 'Character.Instance^1',
    tags: ['Attack'],
    styles: ['Style.Instance^1A'],
    anim_main: {
        files: 'Girl/Attack_03A.*',
        duration: '5s!',
        root_motion: true,
    },
    attributes: {
        '0-5s': {},
    },
    keep_levels: {
        '0-5s': LEVEL_ATTACK,
    },
});

new ActionGeneral('Action.Instance.AttackUnused^1A', {
    enabled: ['#.Action.Instance.AttackUnused^1A', [false, true]],
    character: 'Character.Instance^1',
    tags: ['Attack'],
    styles: ['Style.Instance^1A'],
    anim_main: {
        files: 'Girl/attack_04A.*',
        duration: '5s!',
        root_motion: true,
    },
    attributes: {
        '0-5s': {},
    },
    keep_levels: {
        '0-5s': LEVEL_ATTACK,
    },
});

//
// NPC
//

new CharacterNpc('CharacterNpc.InstanceNpc^1', {
    name: 'CharacterNpc 1',
    tags: ['Npc'],
    level: [1, 3],
    attributes: {
        MaxHealth: [400, 700, 1000],
        MaxPosture: [100, 130, 160],
        PhysicalAttack: [10, 20, 30],
        PhysicalDefense: [15, 25, 35],
    },
    fixed_attributes,
    actions: [
        'Action.InstanceNpc.Idle^1A',
        'Action.InstanceNpc.Walk^1A',
        'Action.InstanceNpc.Attack^1A',
        'Action.InstanceNpc.Hit1^1A',
        'Action.InstanceNpc.Dodge',
    ],
    ai_brains: ['AiBrain.InstanceNpc^1'],
    skeleton_files: 'TrainingDummy/TrainingDummy.*',
    view_model: 'TrainingDummy.prefab',
});

new ActionIdle('Action.InstanceNpc.Idle^1A', {
    character_npcs: ['CharacterNpc.InstanceNpc^1'],
    tags: ['Idle'],
    anim_idle: {
        files: 'TrainingDummy/Idle.*',
        duration: '4s!',
        fade_in: 0.5,
    },
});

new ActionHit('Action.InstanceNpc.Hit1^1A', {
    character_npcs: ['CharacterNpc.InstanceNpc^1'],
    tags: ['Hit'],
    enter_key: Hit1,
    anim_be_hits: [
        {
            enter_angle: 15,
            files: 'TrainingDummy/Hit1_F.*',
            duration: '30F!',
            root_motion: true,
        },
    ],
    blend_be_hits: true,
});

new ActionGeneralNpc('Action.InstanceNpc.Attack^1A', {
    character_npcs: ['CharacterNpc.InstanceNpc^1'],
    tags: ['Attack'],
    anim_main: {
        files: 'Slime/Attack1A.*',
        duration: '168F',
        root_motion: true,
        weapon_motion: false,
        hit_motion: false,
    },
    adjust_movements: {
        '0F': { duration: '8F', max_angle: 45 },
        '20F': { duration: '20F', fade_ratio: 0.1, distance: [2, 5], speed_ratio: [0.8, 1.5] },
    },
    keep_levels: {
        '0-168F': LEVEL_ACTION,
        '150F-168F': LEVEL_ATTACK,
    },
});

const x = new ActionMoveFreeNpc('Action.InstanceNpc.Walk^1A', {
    character_npcs: ['CharacterNpc.InstanceNpc^1'],
    tags: ['Walk'],
    enter_key: Walk,
    move_speed: 1.5,
    anim_move: {
        files: 'Slime/WalkFrontLoop.*',
        duration: '80F',
        root_motion: true,
    },
    anim_start: {
        files: 'Slime/WalkFrontStart.*',
        duration: '40F',
        root_motion: true,
    },
    anim_stops: [
        {
            files: 'Slime/WalkFrontStop.*',
            duration: '40F',
            root_motion: true,
            enter_from_table: [
                { anim: 'Slime/WalkFrontStart.*', ratio: 1.0 },
                { anim: 'Slime/WalkFrontLoop.*', ratio: 0.5 },
                { anim: 'Slime/WalkFrontLoop.*', ratio: 1.0 },
            ],
        },
    ],
    turn_time: '12F',
});

new ActionDodgeNpc('Action.InstanceNpc.Dodge', {
    character_npcs: ['CharacterNpc.InstanceNpc^1'],
    tags: ['Dodge'],
    move_distance: [2.0, 4.0],
    anim_dodges: [
        {
            files: 'Slime/Dodge_L.*',
            duration: '110F',
            root_motion: true,
            shape_key: true,
            enter_angle: -90,
            rotation_reference: 'TargetCharacter' as const,
            rotation_start: '84F',
            rotation_duration: ['16F', '24F'],
            rotation_max_angle: 180,
            keep_levels: { '0-110F': LEVEL_ACTION },
        },
        {
            files: 'Slime/Dodge_R.*',
            duration: '110F',
            root_motion: true,
            shape_key: true,
            enter_angle: 90,
            rotation_reference: 'TargetCharacter' as const,
            rotation_start: '84F',
            rotation_duration: ['16F', '24F'],
            rotation_max_angle: 180,
            keep_levels: { '0-110F': LEVEL_ACTION },
        },
    ],
});

new AiBrain('AiBrain.InstanceNpc^1', {
    character_npc: 'CharacterNpc.InstanceNpc^1',
    alert_sphere: { radius: 5 },
    alert_cone: { radius: 10, half_angle: 45 },
    aggro_sphere: { radius: 10 },
    aggro_lost_time: '10s',
    tasks_from_script: true,
    execute: /*rust*/ `
        out.push((id!("AiTask.InstanceNpc.Idle^1"), 1.0, 1).into());
        out.push((id!("AiTask.InstanceNpc.Patrol^1"), 1.0, 1).into());
        out.push((id!("AiTask.InstanceNpc.MoveTo^1"), 1.0, 1).into());
        out.push((id!("AiTask.InstanceNpc.General^1"), 1.0, 1).into());
        out.push((id!("AiTask.InstanceNpc.KeepDistance^1"), 1.0, 1).into());
    `,
});

new AiTaskIdle('AiTask.InstanceNpc.Idle^1', {
    character_npc: 'CharacterNpc.InstanceNpc^1',
    action_idle: 'Action.InstanceNpc.Idle^1A',
});

new AiTaskPatrol('AiTask.InstanceNpc.Patrol^1', {
    character_npc: 'CharacterNpc.InstanceNpc^1',
    action_idle: 'Action.InstanceNpc.Idle^1A',
    action_move: 'Action.InstanceNpc.Walk^1A',
    route: [
        ['Move', [-3, 0, 0]],
        ['Idle', '2.5s'],
        ['Move', [0, 0, 3]],
    ],
});

new AiTaskMoveToCharacter('AiTask.InstanceNpc.MoveTo^1', {
    character_npc: 'CharacterNpc.InstanceNpc^1',
    expected_distance: [5, 8],
    expected_toward: 180,
    move_action: 'Action.InstanceNpc.Walk^1A',
    turn_action: 'Action.InstanceNpc.Walk^1A',
});

new AiTaskGeneral('AiTask.InstanceNpc.General^1', {
    character_npc: 'CharacterNpc.InstanceNpc^1',
    actions: ['Action.InstanceNpc.Idle^1A', 'Action.InstanceNpc.Walk^1A'],
});

new AiTaskKeepDistance('AiTask.InstanceNpc.KeepDistance^1', {
    character_npc: 'CharacterNpc.InstanceNpc^1',
    intention: 'Attack',
    next_intention: 'SquareOff',
    dodge_action: 'Action.InstanceNpc.Dodge',
    expected_distance: 0,
});

new AiRoutine('AiRoutine.InstanceNpc.Sequence^1', {
    character_npc: 'CharacterNpc.InstanceNpc^1',
    tasks: [
        'AiTask.InstanceNpc.Idle^1',
        'AiTask.InstanceNpc.Patrol^1',
        'AiTask.InstanceNpc.MoveTo^1',
    ],
});

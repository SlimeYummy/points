import {
    ActionDodgeNpc,
    ActionGeneralNpc,
    ActionHit,
    ActionIdle,
    ActionMoveFreeNpc,
    AiBrain,
    AiRoutine,
    AiTaskGeneral,
    AiTaskIdle,
    AiTaskKeepDistance,
    AiTaskMoveToCharacter,
    AiTaskPatrol,
    CharacterNpc,
    Hit1,
    If,
    LEVEL_ACTION,
    LEVEL_ATTACK,
} from '../src';

const fixed_attributes = {
    damage_reduce_param_1: 0.05,
    damage_reduce_param_2: 100,
    guard_damage_ratio_1: 0.8,
    deposture_reduce_param_1: 0.05,
    deposture_reduce_param_2: 200,
    guard_deposture_ratio_1: 0.8,
    weak_damage_up: 0.25,
};

//
// TrainingDummy
//

const NPC = new CharacterNpc('CharacterNpc.TrainingDummy', {
    name: 'TrainingDummy',
    tags: ['Npc'],
    level: [1, 1],
    attributes: {
        MaxHealth: [1000 * 1000 * 1000],
    },
    fixed_attributes,
    actions: ['Action.TrainingDummy.Idle', 'Action.TrainingDummy.Hit1'],
    ai_brains: ['AiBrain.TrainingDummy'],
    skeleton_files: 'TrainingDummy/TrainingDummy.*',
    view_model: 'TrainingDummy.prefab',
});

new ActionIdle('Action.TrainingDummy.Idle', {
    character_npcs: [NPC.id],
    tags: ['Idle'],
    anim_idle: {
        files: 'TrainingDummy/Idle.*',
        duration: '4s',
    },
});

new ActionHit('Action.TrainingDummy.Hit1', {
    character_npcs: [NPC.id],
    tags: ['Hit'],
    enter_key: Hit1,
    anim_be_hits: [
        {
            enter_angle: -90,
            files: 'TrainingDummy/Hit1_R.*',
            fade_in: '6F',
            root_motion: true,
        },
        {
            enter_angle: 0,
            files: 'TrainingDummy/Hit1_F.*',
            fade_in: '6F',
            root_motion: true,
        },
        {
            enter_angle: 90,
            files: 'TrainingDummy/Hit1_L.*',
            fade_in: '6F',
            root_motion: true,
        },
        {
            enter_angle: 180,
            files: 'TrainingDummy/Hit1_B.*',
            fade_in: '6F',
            root_motion: true,
        },
    ],
    blend_be_hits: true,
});

new AiBrain('AiBrain.TrainingDummy', {
    character_npc: 'CharacterNpc.TrainingDummy',
    alert_sphere: { radius: 3 },
    alert_cone: { radius: 5, half_angle: 45 },
    aggro_sphere: { radius: 5 },
    aggro_lost_time: '10s',
    execute: '',
});

//
// Slime
//

// new AiTaskIdle('AiTask.SlimeBlue.Idle', {
//     character_npc: 'CharacterNpc.TrainingDummy',
//     max_repeat: 1,
//     action_idle: 'Action.TrainingDummy.Idle',
//     duration: '4s-6s',
// });

const SLIME = new CharacterNpc('CharacterNpc.Slime', {
    name: 'Slime',
    tags: ['Npc', 'Enemy'],
    level: [1, 1],
    attributes: {
        MaxHealth: [100],
        MaxPosture: [50],
        PostureRecovery: [5],
        PhysicalAttack: [5],
        PhysicalDefense: [3],
    },
    fixed_attributes,
    actions: [
        'Action.Slime.Idle',
        'Action.Slime.Walk',
        'Action.Slime.Run',
        'Action.Slime.Dodge^F',
        'Action.Slime.Dodge^B',
        'Action.Slime.Attack1A',
        'Action.Slime.Attack1B',
        'Action.Slime.Attack2',
    ],
    ai_brains: ['AiBrain.Slime'],
    skeleton_files: 'Slime/Slime.*',
    view_model: 'Slime.prefab',
});

new ActionIdle('Action.Slime.Idle', {
    character_npcs: [SLIME.id],
    tags: ['Idle'],
    anim_idle: {
        files: 'Slime/Idle.*',
        shape_key: true,
    },
});

new ActionMoveFreeNpc('Action.Slime.Walk', {
    character_npcs: [SLIME.id],
    tags: ['Walk'],
    enter_key: 'Walk',
    move_speed: 1.5,
    anim_move: {
        files: 'Slime/WalkFrontLoop.*',
        duration: '80F',
        root_motion: true,
        shape_key: true,
    },
    anim_start: {
        files: 'Slime/WalkFrontStart.*',
        duration: '40F',
        root_motion: true,
        shape_key: true,
    },
    anim_stops: [
        {
            files: 'Slime/WalkFrontStop.*',
            duration: '40F',
            root_motion: true,
            shape_key: true,
            enter_from_table: [
                { anim: 'Slime/WalkFrontStart.*', ratio: 1.0 },
                { anim: 'Slime/WalkFrontLoop.*', ratio: 0.5 },
                { anim: 'Slime/WalkFrontLoop.*', ratio: 1.0 },
            ],
        },
    ],
    turn_time: '60F',
});

new ActionMoveFreeNpc('Action.Slime.Run', {
    character_npcs: [SLIME.id],
    tags: ['Run'],
    enter_key: 'Run',
    move_speed: 3,
    anim_move: {
        files: 'Slime/RunLoop.*',
        duration: '40F',
        root_motion: true,
        shape_key: true,
    },
    anim_start: {
        files: 'Slime/RunStart.*',
        duration: '50F',
        root_motion: true,
        shape_key: true,
    },
    anim_stops: [
        {
            files: 'Slime/RunStop.*',
            duration: '56F',
            root_motion: true,
            shape_key: true,
            enter_from_table: [
                { anim: 'Slime/RunStart.*', ratio: 1.0 },
                { anim: 'Slime/RunLoop.*', ratio: 1.0 },
            ],
        },
    ],
    turn_time: '30F',
});

const SLIME_DODGE_FORWARD = {
    duration: '110F',
    root_motion: true,
    shape_key: true,
    rotation_reference: 'TargetCharacter' as const,
    rotation_start: '0F',
    rotation_duration: ['16F', '24F'],
    rotation_max_angle: 180,
    keep_levels: { '0-110F': LEVEL_ACTION },
};

new ActionDodgeNpc('Action.Slime.Dodge^F', {
    character_npcs: [SLIME.id],
    tags: ['Dodge'],
    move_distance: [1.5, 4.0],
    anim_dodges: [
        {
            files: 'Slime/Dodge_F.*',
            enter_angle: 0,
            ...SLIME_DODGE_FORWARD,
        }
    ],
});

const SLIME_DODGE_BACKWARD = {
    ...SLIME_DODGE_FORWARD,
    rotation_start: '84F',
    rotation_duration: ['16F', '26F'],
};

new ActionDodgeNpc('Action.Slime.Dodge^B', {
    character_npcs: [SLIME.id],
    tags: ['Dodge'],
    move_distance: [1.5, 4.0],
    anim_dodges: [
        {
            files: 'Slime/Dodge_F.*',
            enter_angle: 0,
            ...SLIME_DODGE_BACKWARD,
        },
        {
            files: 'Slime/Dodge_B.*',
            enter_angle: 180,
            ...SLIME_DODGE_BACKWARD,
        },
        {
            files: 'Slime/Dodge_L.*',
            enter_angle: 90,
            ...SLIME_DODGE_BACKWARD,
        },
        {
            files: 'Slime/Dodge_R.*',
            enter_angle: -90,
            ...SLIME_DODGE_BACKWARD,
        },
    ],
});

new ActionGeneralNpc('Action.Slime.Attack1A', {
    character_npcs: [SLIME.id],
    tags: ['Attack'],
    anim_main: {
        files: 'Slime/Attack1A.*',
        duration: '168F',
        root_motion: true,
        hit_motion: true,
    },
    adjust_movements: [
        { time: '0F', duration: '30F', max_angle: 60 },
        { time: '52F', duration: '15F', max_angle: 30 },
        { time: '104F', duration: '26F', distance: [1.5, 4.0], speed_ratio: [0.6, 1.6] },
    ],
    keep_levels: {
        '0-150F': LEVEL_ACTION,
        '150F-168F': LEVEL_ATTACK,
    },
    hits: [
        { group: 'Hit', box_max_times: 1 },
    ],
});

new ActionGeneralNpc('Action.Slime.Attack1B', {
    character_npcs: [SLIME.id],
    tags: ['Attack'],
    anim_main: {
        files: 'Slime/Attack1B.*',
        duration: '138F',
        root_motion: true,
        hit_motion: true,
    },
    adjust_movements: [
        { time: '0F', duration: '15F', max_angle: 30 },
        { time: '42F', duration: '26F', distance: [1.5, 4.0], speed_ratio: [0.6, 1.6] },
    ],
    keep_levels: {
        '0-100F': LEVEL_ACTION,
        '100F-138F': LEVEL_ATTACK,
    },
    hits: [
        { group: 'Hit', box_max_times: 1 },
    ],
});

new ActionGeneralNpc('Action.Slime.Attack2', {
    character_npcs: [SLIME.id],
    tags: ['Attack'],
    anim_main: {
        files: 'Slime/Attack2.*',
        duration: '298F',
        root_motion: true,
        hit_motion: true,
    },
    adjust_movements: [
        { time: '0F', duration: '30F', max_angle: 60 },
        { time: '54F', duration: '22F', max_angle: 45 },
        { time: '110F', duration: '22F', max_angle: 45 },
        { time: '122F', duration: '44F', distance: [3.9, 7.8], speed_ratio: [0.75, 1.5] },
    ],
    keep_levels: {
        '0-100F': LEVEL_ACTION,
        '100F-298F': LEVEL_ATTACK,
    },
    hits: [
        { group: 'Hit', box_max_times: 1 },
    ],
});

new AiBrain('AiBrain.Slime', {
    character_npc: 'CharacterNpc.Slime',
    alert_sphere: { radius: 6 },
    alert_cone: { radius: 10, half_angle: 60 },
    aggro_sphere: { radius: 10 },
    aggro_lost_time: '10s',
    tasks_from_script: true,
    // execute: /*rust*/ `
    //     if target_physics.is_some() {
    //         let dist_sq = (target_physics.unwrap().position - chara_physics.position).length_squared();
    //         if dist_sq < square(4.5) {
    //             // Near
    //             out.push((id!("AiRoutine.Slime.Attack1"), 1.0, 1).into());
    //         } else if dist_sq < square(9.0) {
    //             // Mid
    //             out.push((id!("AiRoutine.Slime.Attack2"), 1.0, 1).into());
    //         } else {
    //             // Far
    //             out.push((id!("AiTask.Slime.Patrol"), 1.0, 1).into());
    //         }
    //     }
    //     if out.is_empty() {
    //         out.push((id!("AiTask.Slime.Idle"), 1.0, 1).into());
    //     }
    // `,
    execute: /*rust*/ `
        if target_physics.is_some() {
            let dist_sq = (target_physics.unwrap().position - chara_physics.position).length_squared();
            if dist_sq < square(4.5) {
                // Near
                out.push((id!("AiRoutine.Slime.Attack1"), 1.0, 1).into());
            } else if dist_sq < square(9.0) {
                // Mid
                out.push((id!("AiRoutine.Slime.Attack2"), 1.0, 1).into());
            } else {
                // Far
                out.push((id!("AiTask.Slime.Patrol"), 1.0, 1).into());
            }
        }
        if out.is_empty() {
            out.push((id!("AiTask.Slime.Idle"), 1.0, 1).into());
        }
    `,
});

new AiTaskIdle('AiTask.Slime.Idle', {
    character_npc: 'CharacterNpc.Slime',
    intention: 'Idle',
    action_idle: 'Action.Slime.Idle',
    duration: '2s-3s',
});

new AiTaskPatrol('AiTask.Slime.Patrol', {
    character_npc: 'CharacterNpc.Slime',
    action_idle: 'Action.Slime.Idle',
    action_move: 'Action.Slime.Run',
    route: [
        ['Move', [-4, 0, 4]],
        ['Idle', '2s'],
        ['Move', [-4, 0, -4]],
        ['Idle', '2s'],
        ['Move', [5, 0, -4]],
        ['Idle', '2s'],
        ['Move', [8, 0, 0]],
        ['Move', [4, 0, 4]],
        ['Idle', '2s'],
    ],
    target_exit: true,
});

new AiTaskMoveToCharacter('AiTask.Slime.MoveTo', {
    character_npc: 'CharacterNpc.Slime',
    expected_distance: [2, 4],
    expected_toward: 60,
    move_action: 'Action.Slime.Run',
    turn_action: 'Action.Slime.Run',
});

new AiTaskKeepDistance('AiTask.Slime.KeepDistance^F', {
    character_npc: 'CharacterNpc.Slime',
    expected_distance: 2.5,
    dodge_action: 'Action.Slime.Dodge^F',
});

new AiTaskKeepDistance('AiTask.Slime.KeepDistance^B', {
    character_npc: 'CharacterNpc.Slime',
    expected_distance: 2.5,
    dodge_action: 'Action.Slime.Dodge^B',
});

new AiRoutine('AiRoutine.Slime.Attack1', {
    character_npc: 'CharacterNpc.Slime',
    tasks: [
        If(/*rust*/`
            if let Some(target_physics) = target_physics {
                let dist_sq = (target_physics.position - chara_physics.position).length_squared();
                if dist_sq < square(1.5) {
                   return true; 
                }
            }
            return false;
        `, "AiTask.Slime.KeepDistance^B")
        .Elsif(/*rust*/`
            if let Some(target_physics) = target_physics {
                let dist_sq = (target_physics.position - chara_physics.position).length_squared();
                if dist_sq > square(3.5) {
                   return true; 
                }
            }
            return false;
        `, "AiTask.Slime.KeepDistance^F"),
        'AiTask.Slime.Attack1A',
        'AiTask.Slime.Attack1B',
        'AiTask.Slime.Idle',
    ],
});

new AiTaskGeneral('AiTask.Slime.Attack1A', {
    character_npc: 'CharacterNpc.Slime',
    actions: ['Action.Slime.Attack1A'],
});

new AiTaskGeneral('AiTask.Slime.Attack1B', {
    character_npc: 'CharacterNpc.Slime',
    actions: ['Action.Slime.Attack1B'],
});

new AiRoutine('AiRoutine.Slime.Attack2', {
    character_npc: 'CharacterNpc.Slime',
    tasks: [
        'AiTask.Slime.Attack2',
        'AiTask.Slime.Idle',
    ],
});

new AiTaskGeneral('AiTask.Slime.Attack2', {
    character_npc: 'CharacterNpc.Slime',
    actions: ['Action.Slime.Attack2'],
});

import {
    ActionGeneral,
    ActionIdle,
    ActionMove,
    Attack1,
    Attack2,
    Capsule,
    Character,
    Equipment,
    Forward,
    FPS,
    LEVEL_ACTION,
    LEVEL_ATTACK,
    Perk,
    Slot1,
    Slot3,
    Style,
    Var,
} from '../src';

Var.define({
    '#.Action.Instance.AttackDerive/1A': [2, ['Character.Instance/1']],
    '#.Action.Instance.AttackUnused/1A': [1, ['Character.Instance/1']],
    '#.Perk.Instance/1A': [1, ['Style.Instance/1A']],
    '#.Perk.Instance/1B': [3, ['Style.Instance/1A']],
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

new Character('Character.Instance/1', {
    name: 'Character 1',
    level: [1, 6],
    styles: ['Style.Instance/1A'],
    equipments: ['Equipment.Instance/1A', 'Equipment.Instance/1B'],
    bounding_capsule: new Capsule(0.5 * 1.35, 0.3),
    skeleton_files: 'girl',
    skeleton_toward: [0, 1],
});

new Style('Style.Instance/1A', {
    name: 'Style 1',
    character: 'Character.Instance/1',
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
    perks: ['Perk.Instance/1A', 'Perk.Instance/1B'],
    actions: [
        'Action.Instance.Idle/1A',
        'Action.Instance.Run/1A',
        'Action.Instance.Attack/1A',
        'Action.Instance.AttackDerive/1A',
        'Action.Instance.AttackUnused/1A',
    ],
    view_model: 'StyleOne-1.vrm',
});

new Equipment('Equipment.Instance/1A', {
    name: 'Weapon 1A',
    character: 'Character.Instance/1',
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

new Equipment('Equipment.Instance/1B', {
    name: 'Weapon 1B',
    character: 'Character.Instance/1',
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

new Perk('Perk.Instance/1A', {
    name: 'Instance 1A',
    character: 'Character.Instance/1',
    style: 'Style.Instance/1A',
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
        '#.Perk.Instance/1A': [0, 1],
        '#.Perk.Instance/1B': [1, 2],
    },
});

new Perk('Perk.Instance/1B', {
    name: 'Instance 1A',
    character: 'Character.Instance/1',
    style: 'Style.Instance/1A',
    max_level: 3,
    slots: ['A1D1', 'S1A1D1', 'S1A2D2'],
    var_indexes: {
        '#.Perk.Instance/1B': [2, 3, 4],
    },
});

new ActionIdle('Action.Instance.Idle/1A', {
    character: 'Character.Instance/1',
    styles: ['Style.Instance/1A'],
    anim_idle: {
        files: 'girl_stand_idle',
        duration: '2.5s',
    },
    anim_ready: {
        files: 'girl_stand_ready',
        duration: 2,
        fade_in: 0.4,
    },
});

new ActionMove('Action.Instance.Run/1A', {
    character: 'Character.Instance/1',
    styles: ['Style.Instance/1A'],
    anim_move: {
        files: 'girl_run',
        duration: 3,
        fade_in: 0.2,
    },
    yam_time: 0.4,
    turn_time: 1,
});

new ActionGeneral('Action.Instance.Attack/1A', {
    character: 'Character.Instance/1',
    styles: ['Style.Instance/1A'],
    anim_main: {
        files: 'girl_attack1_1',
        duration: 4,
        root_motion: true,
    },
    enter_key: Attack1,
    enter_level: LEVEL_ATTACK,
    motion_distance: [0.7, 1.2],
    motion_toward: 60,
    attributes: {
        '0-4s': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        },
    },
    derive_levels: {
        '0-4s': LEVEL_ACTION,
        '2.5s-4.5s': LEVEL_ATTACK,
    },
    derives: [
        [Attack1, 'Action.Instance.AttackDerive/1A'],
        [Attack2, [Forward, 60], 'Action.Instance.AttackDerive/1A'],
    ],
});

new ActionGeneral('Action.Instance.AttackDerive/1A', {
    enabled: ['#.Action.Instance.AttackDerive/1A', [false, false, true]],
    character: 'Character.Instance/1',
    styles: ['Style.Instance/1A'],
    anim_main: {
        files: 'girl_attack1_2',
        duration: '5s',
        root_motion: true,
    },
    enter_level: LEVEL_ATTACK,
    attributes: {
        '0-5s': {},
    },
    derive_levels: {
        '0-5s': LEVEL_ATTACK,
    },
});

new ActionGeneral('Action.Instance.AttackUnused/1A', {
    enabled: ['#.Action.Instance.AttackUnused/1A', [false, true]],
    character: 'Character.Instance/1',
    styles: ['Style.Instance/1A'],
    anim_main: {
        files: 'girl_attack1_2',
        duration: '5s',
        root_motion: true,
    },
    enter_level: LEVEL_ATTACK,
    attributes: {
        '0-5s': {},
    },
    derive_levels: {
        '0-5s': LEVEL_ATTACK,
    },
});

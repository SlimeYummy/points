import {
    Accessory,
    AccessoryPool,
    ActionGeneral,
    ActionIdle,
    ActionMove,
    Attack,
    Attack1,
    Attack2,
    Capsule,
    Character,
    Defense,
    Entry,
    Equipment,
    Forward,
    Jewel,
    LEVEL_ACTION,
    LEVEL_ATTACK,
    MAX_ENTRY_PLUS,
    Perk,
    Rare1,
    Rare2,
    Rare3,
    Slot1,
    Slot3,
    Special,
    Style,
    Var,
    Variant1,
    Variant2,
    Variant3,
    Zone,
} from '../src';

Var.define({
    '#.One.NormalAttack.Branch': [2, 'Character.One'],
    '#.Entry.Variable/1': [3, 'Character.One'],
    '#.Entry.Variable/2': [2, '*'],
});

//
// Character & Style
//

const fixed_attributes = {
    damage_reduce_param_1: 0.05,
    damage_reduce_param_2: 100,
    guard_damage_ratio_1: 0.8,
    deposture_reduce_param_1: 0.05,
    deposture_reduce_param_2: 200,
    guard_deposture_ratio_1: 0.8,
    weak_damage_up: 0.25,
};

const ONE = new Character('Character.One', {
    name: 'Character One',
    level: [1, 6],
    styles: ['Style.One/1', 'Style.One/2'],
    equipments: ['Equipment.No1', 'Equipment.No2', 'Equipment.No3'],
    bounding_capsule: new Capsule(0.5 * 1.35, 0.3),
    skeleton_files: 'girl',
    skeleton_toward: [0, 1],
});

new Style('Style.One/1', {
    name: 'Character One Type-1',
    character: 'Character.One',
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
    perks: ['Perk.One.NormalAttack.Branch', 'Perk.One.AttackUp'],
    usable_perks: ['Perk.One.FinalPerk'],
    actions: ['Action.One.Idle', 'Action.One.Run', 'Action.One.Attack/1', 'Action.One.Attack/2'],
    view_model: 'StyleOne-1.vrm',
});

new Style('Style.One/2', {
    name: 'Character One Type-2',
    character: 'Character.One',
    attributes: {},
    slots: ['A1', 'A1', 'A1', 'A1', 'A1', 'A1'],
    fixed_attributes,
    perks: ['Perk.One.FinalPerk'],
    usable_perks: ['Perk.One.AttackUp'],
    actions: ['Action.One.Idle', 'Action.One.Run', 'Action.One.Attack/1'],
    view_model: 'OneType-2.vrm',
});

new Character('Character.Two', {
    name: 'Character Two',
    level: [0, 5],
    styles: ['Style.Two/1'],
    equipments: ['Equipment.No4'],
    bounding_capsule: new Capsule(0.5 * 1.35, 0.3),
    skeleton_files: 'girl',
    skeleton_toward: [0, 1],
});

new Style('Style.Two/1', {
    name: 'Character Two Type-1',
    character: 'Character.Two',
    attributes: {},
    slots: ['A1', 'A1', 'A1', 'A1', 'A1', 'A1'],
    fixed_attributes,
    perks: [],
    // perks: ['Perk.No2.AttackUp'],
    actions: [],
    view_model: 'TwoType-1.vrm',
});

//
// Equipment
//

new Equipment('Equipment.No1', {
    name: 'Weapon No1',
    character: 'Character.One',
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

new Equipment('Equipment.No2', {
    name: 'Weapon No2',
    character: 'Character.One',
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

new Equipment('Equipment.No3', {
    name: 'Weapon No3',
    character: 'Character.One',
    slot: Slot1,
    level: [0, 3],
    attributes: {},
});

new Equipment('Equipment.No4', {
    name: 'Weapon No4',
    character: 'Character.Two',
    slot: Slot1,
    level: [0, 3],
    attributes: {},
});

//
// Action
//

new ActionIdle('Action.One.Idle', {
    character: ONE.id,
    styles: ONE.styles,
    anim_idle: {
        files: 'girl_stand_idle',
        duration: '2.5s',
    },
    anim_ready: {
        files: 'girl_stand_ready',
        duration: '2s',
    },
});

new ActionMove('Action.One.Run', {
    character: ONE.id,
    styles: ONE.styles,
    anim_move: {
        files: 'girl_run',
        duration: '3s',
        fade_in: '0.2s',
    },
    yam_time: '0.333s',
    turn_time: '1s',
});

new ActionGeneral('Action.One.Attack/1', {
    anim_main: {
        files: 'girl_attack1_1',
        duration: 4,
        root_motion: true,
    },
    character: ONE.id,
    styles: ['Style.One/1', 'Style.One/2'],
    enter_key: Attack1,
    enter_level: LEVEL_ATTACK,
    motion_distance: [0.5, 1.0],
    motion_toward: 45,
    attributes: {
        '0-4s': {
            damage_rdc: '20%',
            shield_dmg_rdc: 0,
            poise_level: 1,
        },
    },
    derive_levels: {
        '0-4s': LEVEL_ACTION,
        '2.5s-4s': LEVEL_ATTACK,
    },
    derives: [
        [Attack1, 'Action.One.Attack/2'],
        [Attack2, [Forward, 60], 'Action.One.Attack/2'],
    ],
});

new ActionGeneral('Action.One.Attack/2', {
    anim_main: {
        files: 'girl_attack2_1',
        duration: '5s',
        root_motion: true,
    },
    enabled: ['#.One.NormalAttack.Branch', [false, true, true]],
    character: ONE.id,
    styles: ['Style.One/1'],
    enter_key: Attack1,
    enter_level: LEVEL_ATTACK,
    attributes: {
        '0-5s': {},
    },
    derive_levels: {
        '0-5s': LEVEL_ACTION,
        '3s-5s': LEVEL_ATTACK,
    },
});

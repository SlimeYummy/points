import {
    Accessory,
    AccessoryPool,
    ActionGeneral,
    ActionIdle,
    ActionMove,
    Attack,
    Attack1,
    Attack2,
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
    Run,
    Slot1,
    Slot3,
    Special,
    Style,
    TaperedCapsule,
    Var,
    Variant1,
    Variant2,
    Variant3,
    Zone,
} from '../src';

Var.define({
    '#.One.NormalAttack.Branch': [2, 'Character.One'],
    '#.Entry.Variable^1': [3, 'Character.One'],
    '#.Entry.Variable^2': [2, '*'],
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
    styles: ['Style.One^1', 'Style.One^2'],
    equipments: ['Equipment.No1', 'Equipment.No2', 'Equipment.No3'],
    bounding: new TaperedCapsule(0.6, 0.3, 0.1),
    skeleton_files: 'Girl.*',
    skeleton_toward: [0, 1],
    body_file: 'Girl.body.json',
});

new Style('Style.One^1', {
    name: 'Character One Type-1',
    character: 'Character.One',
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
    perks: ['Perk.One.NormalAttack.Branch', 'Perk.One.AttackUp'],
    usable_perks: ['Perk.One.FinalPerk'],
    actions: [
        'Action.One.Idle',
        'Action.One.IdleX',
        'Action.One.Run',
        'Action.One.Attack^1',
        'Action.One.Attack^2',
    ],
    view_model: 'StyleOne-1.vrm',
});

new Style('Style.One^2', {
    name: 'Character One Type-2',
    character: 'Character.One',
    tags: ['Player'],
    attributes: {},
    slots: ['A1', 'A1', 'A1', 'A1', 'A1', 'A1'],
    fixed_attributes,
    perks: ['Perk.One.FinalPerk'],
    usable_perks: ['Perk.One.AttackUp'],
    actions: ['Action.One.Idle', 'Action.One.Run', 'Action.One.Attack^1'],
    view_model: 'OneType-2.vrm',
});

new Character('Character.Two', {
    name: 'Character Two',
    level: [0, 5],
    styles: ['Style.Two^1'],
    equipments: ['Equipment.No4'],
    bounding: new TaperedCapsule(0.6, 0.3, 0.1),
    skeleton_files: 'Girl.*',
    skeleton_toward: [0, 1],
    body_file: 'body2.json',
});

new Style('Style.Two^1', {
    name: 'Character Two Type-1',
    character: 'Character.Two',
    tags: ['Player'],
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
    tags: ['Idle'],
    anim_idle: {
        files: 'Girl_Idle_Empty.*',
        duration: '2.5s',
    },
    anim_ready: {
        files: 'Girl_Idle_Axe.*',
        duration: '2s',
    },
});

new ActionIdle('Action.One.IdleX', {
    character: ONE.id,
    styles: ['Style.One^1'],
    tags: ['Idle'],
    anim_idle: {
        files: 'Girl_Idle_Empty.*',
        duration: '2.5s',
    },
});

new ActionMove('Action.One.Run', {
    character: ONE.id,
    styles: ONE.styles,
    tags: ['Run'],
    enter_key: Run,
    anim_move: {
        files: 'Girl_Run_Empty.*',
        fade_in: '4F',
        root_motion: true,
    },
    move_speed: 3,
    starts: [
        {
            enter_angle: ['L15', 'R15'],
            files: 'Girl_RunStart_Empty.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '2F',
            quick_stop_end: '20F',
        },
        {
            enter_angle: ['L15', 'L180'],
            files: 'Girl_RunStart_L180_Empty.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '8F',
            quick_stop_end: '26F',
        },
        {
            enter_angle: ['R15', 'R180'],
            files: 'Girl_RunStart_R180_Empty.*',
            fade_in: 0,
            root_motion: true,
            turn_in_place_end: '8F',
            quick_stop_end: '26F',
        },
    ],
    turn_time: '10F',
    stops: [
        {
            enter_phase_table: [{ phase: [0.75, 0.25], offset: '2F' }],
            files: 'Girl_RunStop_L_Empty.*',
            fade_in: '4F',
            root_motion: true,
            leave_phase_table: [
                ['0F', 0.0],
                ['14F', 0.5],
            ],
        },
        {
            enter_phase_table: [{ phase: [0.25, 0.75], offset: '2F' }],
            files: 'Girl_RunStop_R_Empty.*',
            fade_in: '4F',
            root_motion: true,
            leave_phase_table: [
                ['0F', 0.5],
                ['14F', 0.0],
            ],
        },
    ],
    quick_stop_time: 0,
    smooth_move_froms: ['Action.One.Run'],
});

new ActionGeneral('Action.One.Attack^1', {
    anim_main: {
        files: 'Girl_Attack_01A.*',
        duration: 4,
        root_motion: true,
    },
    character: ONE.id,
    tags: ['Attack'],
    styles: ['Style.One^1', 'Style.One^2'],
    enter_key: Attack1,
    enter_level: LEVEL_ATTACK,
    input_movements: {
        '0F': { duration: '8F', angle: 45 },
        '20F': { move: true },
    },
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
        {
            key: Attack1,
            level: LEVEL_ATTACK + 1,
            action: 'Action.One.Attack^2'
        },
        {
            key: [Attack2, 'F60'],
            level: LEVEL_ATTACK + 1,
            action: 'Action.One.Attack^2'
        },
    ],
    custom_events: {
        '1s': 'CustomEvent',
    },
});

new ActionGeneral('Action.One.Attack^2', {
    anim_main: {
        files: 'Girl_attack_02A.*',
        duration: '5s',
        root_motion: true,
    },
    enabled: ['#.One.NormalAttack.Branch', [false, true, true]],
    character: ONE.id,
    tags: ['Attack'],
    styles: ['Style.One^1'],
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

//
// Perk
//

new Perk('Perk.One.NormalAttack.Branch', {
    name: 'Normal Attack Branch',
    character: ONE.id,
    style: 'Style.One^1',
    max_level: 2,
    var_indexes: {
        '#.One.NormalAttack.Branch': [1, 2],
    },
});

new Perk('Perk.One.AttackUp', {
    name: 'Attack Up',
    character: ONE.id,
    style: 'Style.One^1',
    usable_styles: ONE.styles,
    max_level: 3,
    attributes: {
        AttackUp: ['10%', '15%', '20%'],
    },
});

new Perk('Perk.One.FinalPerk', {
    name: 'Final Perk',
    character: ONE.id,
    style: 'Style.One^2',
    usable_styles: ONE.styles,
    parents: {
        'Perk.One.AttackUp': 3,
    },
    max_level: 1,
    slots: ['S1'],
    entries: {
        'Entry.AttackUp': [[1, MAX_ENTRY_PLUS]],
        'Entry.DefenseUp': [[1, MAX_ENTRY_PLUS]],
    },
});

//
// Jewel
//

new Jewel('Jewel.DefenseUp^1', {
    slot: Defense,
    rare: Rare1,
    entry: 'Entry.DefenseUp',
    piece: 1,
    variant: Variant1,
});

new Jewel('Jewel.AttackUp^1', {
    slot: Attack,
    rare: Rare1,
    entry: 'Entry.AttackUp',
    piece: 1,
    variant: Variant1,
});

new Jewel('Jewel.AttackUp^2', {
    slot: Attack,
    rare: Rare2,
    entry: 'Entry.AttackUp',
    piece: 2,
    variant: Variant2,
});

new Jewel('Jewel.SuperCritical', {
    slot: Special,
    rare: Rare3,
    entry: 'Entry.CriticalChance',
    piece: 2,
    sub_entry: 'Entry.CriticalDamage',
    sub_piece: 1,
    variant: Variant2,
});

//
// Accessory
//

const POOL_RARE1 = 'AccessoryPool.Rare1';
new AccessoryPool(POOL_RARE1, {
    rare: Rare1,
    patterns: 'S B B',
    max_level: 9,
    a_entries: {},
    b_entries: {
        'Entry.DefenseUp': 10,
        'Entry.ElementalDefenseUp': 10,
    },
});

const POOL_RARE2 = 'AccessoryPool.Rare2';
new AccessoryPool(POOL_RARE2, {
    rare: Rare2,
    patterns: 'S AB B B',
    max_level: 12,
    a_entries: {
        'Entry.AttackUp': 10,
        'Entry.CriticalChance': 10,
        'Entry.MaxHealthUp': 20,
    },
    b_entries: {
        'Entry.DefenseUp': 10,
        'Entry.ElementalDefenseUp': 10,
    },
});

const POOL_RARE3 = 'AccessoryPool.Rare3';
new AccessoryPool(POOL_RARE3, {
    rare: Rare3,
    patterns: 'S A AB AB B',
    max_level: 15,
    a_entries: {
        'Entry.AttackUp': 10,
        'Entry.CriticalChance': 10,
    },
    b_entries: {
        'Entry.DefenseUp': 10,
        'Entry.ElementalDefenseUp': 10,
        'Entry.MaxHealthUp': 10,
    },
});

new Accessory('Accessory.AttackUp^1', {
    pool: POOL_RARE1,
    rare: 'Rare1',
    entry: 'Entry.AttackUp',
    piece: 1,
    variant: Variant1,
});

new Accessory('Accessory.CriticalChance', {
    pool: POOL_RARE2,
    rare: 'Rare2',
    entry: 'Entry.CriticalChance',
    piece: 2,
    variant: Variant2,
});

new Accessory('Accessory.AttackUp^3', {
    pool: POOL_RARE3,
    rare: 'Rare3',
    entry: 'Entry.AttackUp',
    piece: 3,
    variant: Variant3,
});

//
// Entry
//

new Entry('Entry.Empty', { name: '', max_piece: 1 });

new Entry('Entry.MaxHealthUp', {
    name: 'MaxHealthUp',
    max_piece: 4,
    attributes: {
        MaxHealthUp: ['10%', '20%', '27.5%', '35%'],
        $MaxHealthUp: ['3.5%', '7%', '11%', '15%'],
    },
});

new Entry('Entry.AttackUp', {
    name: 'AttackUp',
    max_piece: 5,
    attributes: {
        AttackUp: ['4%', '8%', '12%', '16%', '20%'],
        $AttackUp: ['2%', '4%', '6%', '8%', '10%'],
    },
});

new Entry('Entry.DefenseUp', {
    name: 'DefenseUp',
    max_piece: 5,
    attributes: {
        DefenseUp: ['15%', '30%', '40%', '50%', '60%'],
        $DefenseUp: ['5%', '10%', '20%', '20%', '20%'],
        $MaxHealthUp: [0, 0, 0, '5%', '10%'],
    },
});

new Entry('Entry.ElementalDefenseUp', {
    name: 'ElementalDefenseUp',
    max_piece: 3,
    attributes: {
        ElementalDefenseUp: ['20%', '40%', '60%'],
        $ElementalDefenseUp: ['5%', '10%', '10%'],
        $AttackUp: ['1%', '2%', '4%'],
    },
});

new Entry('Entry.CriticalChance', {
    name: 'CriticalChance',
    max_piece: 4,
    attributes: {
        CriticalChance: ['5%', '10%', '15%', '20%'],
        $CriticalChance: ['2.5%', '5%', '7.5%', '10%'],
    },
});

new Entry('Entry.CriticalDamage', {
    name: 'CriticalDamage',
    max_piece: 3,
    attributes: {
        CriticalChance: ['8%', '16%', '25%'],
        $CriticalChance: ['5%', '10%', '15%'],
    },
});

new Entry('Entry.Variable', {
    name: 'Entry Variable',
    max_piece: 3,
    var_indexes: {
        '#.Entry.Variable^1': [1, 2, 3],
        '#.Entry.Variable^2': [0, 1, 2],
    },
});

//
// Zone
//

new Zone('Zone.Demo', {
    name: 'Demo',
    zone_file: 'TestZone.json',
    view_zone_file: 'stage-demo.tscn',
});

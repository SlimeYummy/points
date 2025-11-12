import { Character, Equipment, Perk, Slot1, Slot3, Style, TaperedCapsule } from '../src';

const fixed_attributes = {
    damage_reduce_param_1: 0.05,
    damage_reduce_param_2: 100,
    guard_damage_ratio_1: 0.8,
    deposture_reduce_param_1: 0.05,
    deposture_reduce_param_2: 200,
    guard_deposture_ratio_1: 0.8,
    weak_damage_up: 0.25,
};

new Character('Character.Verify/1', {
    name: 'Character 1',
    level: [1, 3],
    styles: ['Style.Verify/1A', 'Style.Verify/1B'],
    equipments: ['Equipment.Verify/1A', 'Equipment.Verify/1B', 'Equipment.Verify/1C'],
    bounding: new TaperedCapsule(0.6, 0.3, 0.1),
    skeleton_files: 'girl.*',
    skeleton_toward: [0, 1],
    body_file: 'body1.json',
});

new Style('Style.Verify/1A', {
    name: 'Style 1',
    character: 'Character.Verify/1',
    attributes: {},
    slots: ['A2D2', 'A2D2', 'S1A2D2'],
    fixed_attributes,
    perks: ['Perk.Verify/1A', 'Perk.Verify/1B'],
    actions: [],
    view_model: 'StyleOne-1.vrm',
});

new Style('Style.Verify/1B', {
    name: 'Style 1',
    character: 'Character.Verify/1',
    attributes: {},
    slots: ['A1', 'A1', 'A1'],
    fixed_attributes,
    perks: [],
    usable_perks: ['Perk.Verify/1A'],
    actions: [],
    view_model: 'StyleOne-1.vrm',
});

new Character('Character.Verify/2', {
    name: 'Character 2',
    level: [1, 3],
    styles: ['Style.Verify/2'],
    equipments: ['Equipment.Verify/2A'],
    bounding: new TaperedCapsule(0.6, 0.3, 0.1),
    skeleton_files: 'skel.*',
    skeleton_toward: [0, 1],
    body_file: 'body2.json',
});

new Style('Style.Verify/2', {
    name: 'Style 2',
    character: 'Character.Verify/2',
    attributes: {},
    slots: ['A1', 'A1', 'A1'],
    fixed_attributes,
    perks: ['Perk.Verify/2A'],
    actions: [],
    view_model: 'StyleOne-1.vrm',
});

new Equipment('Equipment.Verify/1A', {
    name: 'Weapon 1A',
    character: 'Character.Verify/1',
    slot: Slot1,
    level: [1, 4],
    attributes: {},
    slots: ['', '', 'A1', 'S1A1'],
});

new Equipment('Equipment.Verify/1B', {
    name: 'Weapon 1B',
    character: 'Character.Verify/1',
    slot: Slot1,
    level: [1, 4],
    attributes: {},
});

new Equipment('Equipment.Verify/1C', {
    name: 'Weapon 1B',
    character: 'Character.Verify/1',
    slot: Slot3,
    level: [1, 2],
    attributes: {},
    slots: ['', 'D2'],
});

new Equipment('Equipment.Verify/2A', {
    name: 'Weapon 2A',
    character: 'Character.Verify/2',
    slot: Slot1,
    level: [1, 4],
    attributes: {},
    slots: ['', '', 'A1', 'A1'],
    entries: {},
});

new Perk('Perk.Verify/1A', {
    name: 'Verify 1A',
    character: 'Character.Verify/1',
    style: 'Style.Verify/1A',
    max_level: 2,
    usable_styles: ['Style.Verify/1B'],
    slots: ['', 'A1'],
});

new Perk('Perk.Verify/1B', {
    name: 'Verify 1A',
    character: 'Character.Verify/1',
    style: 'Style.Verify/1A',
    max_level: 2,
});

new Perk('Perk.Verify/2A', {
    name: 'Verify 2A',
    character: 'Character.Verify/2',
    style: 'Style.Verify/2',
    max_level: 2,
});

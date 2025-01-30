from turning_point import *

fixed_attributes = FixedAttributes(
    damage_reduce_param_1=0.05,
    damage_reduce_param_2=100,
    guard_damage_ratio_1=0.8,
    deposture_reduce_param_1=0.05,
    deposture_reduce_param_2=200,
    guard_deposture_ratio_1=0.8,
    weak_damage_up=0.25,
)

Character(
    'Character.No1',
    name='No1',
    level=[1, 6],
    styles=['Style.No1-1', 'Style.No1-2'],
    equipments=[
        'Equipment.No1',
        'Equipment.No2',
        'Equipment.No3',
    ]
)

Style(
    'Style.No1-1',
    name='No1-1',
    character='Character.No1',
    attributes={
        MaxHealth: [400, 550, 700, 850, 1000, 1200],
        MaxPosture: [100, 115, 130, 145, 160, 180],
        PostureRecovery: [10, 11, 12, 13, 14, 15],
        PhysicalAttack: [10, 15, 20, 25, 30, 35],
        PhysicalDefense: [15, 20, 25, 30, 35, 40],
        ElementalAttack: [8, 12, 16, 20, 24, 28],
        ElementalDefense: [10, 15, 20, 25, 30, 35],
        ArcaneAttack: [9, 13, 17, 21, 25, 30],
        ArcaneDefense: [5, 8, 11, 14, 17, 20],
        CriticalChance: ['10%'] * 6,
        CriticalDamage: ['30%'] * 6,
    },
    slots=['A2D2', 'A2D2', 'A3D3', 'A3D3S2', 'A5D4S2', 'A5D4S3'],
    fixed_attributes=fixed_attributes,
    perks=[
        'Perk.No1.AttackUp',
        'Perk.No1.CriticalChance',
    ],
    usable_perks=[
        'Perk.No1.Slot',
        'Perk.No1.Empty',
    ],
    skeleton="girl_skeleton_logic.ozz",
    actions=[
        "Action.No1.Idle"
    ],
    icon="icon",
    view_model="No1-1.vrm",
)

Style(
    'Style.No1-2',
    name='No1-2',
    character='Character.No1',
    attributes={},
    slots=["A1", "A1", "A1", "A1", "A1", "A1"],
    fixed_attributes=fixed_attributes,
    perks=[
        'Perk.No1.Slot',
        'Perk.No1.Empty',
    ],
    usable_perks=[
        'Perk.No1.AttackUp',
        'Perk.No1.CriticalChance',
    ],
    skeleton="*.ozz",
    actions=[],
    icon="icon",
    view_model="No1-2.vrm",
)

Character(
    'Character.No2',
    name='No2',
    level=[0, 5],
    styles=['Style.No2-1'],
    equipments=['Equipment.No4'],
)

Style(
    'Style.No2-1',
    name='No2-1',
    character='Character.No2',
    attributes={},
    slots=["A1", "A1", "A1", "A1", "A1", "A1"],
    fixed_attributes=fixed_attributes,
    perks=['Perk.No2.AttackUp'],
    skeleton="*.ozz",
    actions=[],
    icon="icon",
    view_model="No2-1.vrm",
)

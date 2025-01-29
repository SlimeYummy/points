from turning_point import *

Equipment(
    'Equipment.No1',
    name="No1",
    icon='icon',
    character='Character.No1',
    position=Position1,
    level=[1, 4],
    materials=[],
    attributes={
        PhysicalAttack: [13, 19, 25, 31],
        ElementalAttack: [8, 12, 16, 20],
        ArcaneAttack: [13, 18, 23, 28],
        CriticalChance: ['2%', '3%', '4%', '5%'],
    },
    slots=['', '', 'A1', 'A1'],
    entries={
        "Entry.AttackUp": [(1,0), (1,1), (1,2), (1,3)],
    },
    script_args={
        "extra_def": [5, 10, 15, 20],
    },
    script="""
    on_assemble {
        secondary.final_skill_damage_ratio *= 110%
    }

    after_assemble {
        extra.cut_defense += A.extra_def
        extra.blunt_defense += A.extra_def
        extra.ammo_defense += A.extra_def
    }
    """
)

Equipment(
    'Equipment.No2',
    name="No2",
    icon='icon',
    character='Character.No1',
    position=Position3,
    level=[0, 3],
    materials=[],
    attributes={
        PhysicalAttack: [10, 15, 20, 25],
        ElementalAttack: [7, 10, 13, 16],
        ArcaneAttack: [8, 12, 16, 20],
        CriticalDamage: ['10%', '12%', '15%', '18%'],
    },
    slots=['A1', 'A2', 'S1A1', 'S1A2'],
    entries={
        "Entry.DefenseUp": [(2,0), (2,1), (2,2), (2,3)],
    },
)

Equipment(
    'Equipment.No3',
    name="No3",
    icon='icon',
    character='Character.No1',
    position=Position1,
    level=[0, 3],
    materials=[],
    attributes={},
)

Equipment(
    'Equipment.No4',
    name="No4",
    icon='icon',
    character='Character.No2',
    position=Position1,
    level=[0, 3],
    materials=[],
    attributes={},
)

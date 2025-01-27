from turning_point import *

Perk(
    'Perk.No1.AttackUp',
    name='AttackUp',
    icon='icon',
    style='Style.No1-1',
    usable_styles=['Style.No1-2'],
    attributes={
        AttackUp: '10%',
    },
    script_args={
        "physical_attack": 2,
        "elemental_attack": 2,
        "arcane_attack": 2,
    },
    script="""
    after_assemble {
        extra.physical_attack += A.physical_attack
        extra.elemental_attack += A.elemental_attack
        extra.arcane_attack += A.arcane_attack
    }
    """
)

Perk(
    'Perk.No1.CriticalChance',
    name='CriticalChance',
    icon='icon',
    style='Style.No1-1',
    usable_styles=['Style.No1-2'],
    entries={
        'Entry.CriticalChance': (1,3),
    },
    script="""
    on_assemble {
        secondary.critical_chance += 2%
    }
    """
)

Perk(
    'Perk.No1.Slot',
    name='Slot',
    icon='icon',
    style='Style.No1-2',
    usable_styles=['Style.No1-1'],
    slot='A2D2',
)

Perk(
    'Perk.No1.Empty',
    name='Empty',
    icon='icon',
    style='Style.No1-2',
    usable_styles=['Style.No1-1'],
    entries={
        'Entry.Empty': (0,0),
    },
)

Perk(
    'Perk.No2.AttackUp',
    name='AttackUp',
    icon='icon',
    style='Style.No2-1',
    attributes={
        AttackUp: 10,
    },
)

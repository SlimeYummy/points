from turning_point import *

Entry(
    'Entry.Empty',
    name='',
    icon='',
    color='#ffffff',
    max_piece=1
)

Entry(
    'Entry.MaxHealthUp',
    name='MaxHealthUp',
    icon='icon',
    color='#ffffff',
    max_piece=4,
    attributes={
        MaxHealthUp: ['10%', '20%', '30%', '40%'],
        'DefenseUp+': ['3%', '6%', '9%', '12%'],
    }
)

Entry(
    'Entry.AttackUp',
    name='AttackUp',
    icon='icon',
    color='#ffffff',
    max_piece=5,
    attributes={
        AttackUp: ['10%', '20%', '30%', '40%', '50%'],
        'AttackUp+': ['2%', '4%', '6%', '8%', '10%'],
    }
)

Entry(
    'Entry.DefenseUp',
    name='DefenseUp',
    icon='icon',
    color='#ffffff',
    max_piece=5,
    attributes={
        DefenseUp: ['20%', '40%', '60%', '80%', '100%'],
        'DefenseUp+': ['10%', '20%', '30%', '40%', '50%'],
    }
)

Entry(
    'Entry.CutDefenseUp',
    name='CutDefenseUp',
    icon='icon',
    color='#ffffff',
    max_piece=3,
    attributes={
        CutDefenseUp: ['30%', '60%', '90%'],
        'CutDefenseUp+': ['10%', '25%', '40%'],
    }
)

Entry(
    'Entry.CriticalChance',
    name='CriticalChance',
    icon='icon',
    color='#ffffff',
    max_piece=3,
    attributes={
        CriticalChance: ['10%', '20%', '30%'],
        'CriticalChance+': ['5%', '10%', '15%'],
    }
)

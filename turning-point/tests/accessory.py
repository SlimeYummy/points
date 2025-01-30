from turning_point import *

Pattern1 = 'AccessoryPattern.Rare1'
AccessoryPattern.new(Pattern1, Rare1, 'S B B', 9,
    a_pool={},
    b_pool={
        'Entry.DefenseUp': 10,
        'Entry.CutDefenseUp': 10,
    }
)

Pattern2 = 'AccessoryPattern.Rare2'
AccessoryPattern.new(Pattern2, Rare2, 'S AB B B', 12,
    a_pool={
        'Entry.AttackUp': 10,
        'Entry.CriticalChance': 10,
        'Entry.MaxHealthUp': 20,
    },
    b_pool={
        'Entry.DefenseUp': 10,
        'Entry.CutDefenseUp': 10,
    },
)

Pattern3 = 'AccessoryPattern.Rare3'
AccessoryPattern.new(Pattern3, Rare3, 'S A AB AB B', 15,
    a_pool={
        'Entry.AttackUp': 10,
        'Entry.CriticalChance': 10,
    },
    b_pool={
        'Entry.DefenseUp': 10,
        'Entry.CutDefenseUp': 10,
        'Entry.MaxHealthUp': 10,
    },
)

Accessory.new(Pattern1, 'Entry.AttackUp', 1)
Accessory.new(Pattern2, 'Entry.CriticalChance', 2, Variant2)
Accessory.new(Pattern3, 'Entry.AttackUp', 3, Variant3)

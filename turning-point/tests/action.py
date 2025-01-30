from turning_point import *

anime_empty = Animation("empty.ozz", '3s')

# # # # # # # # # # Idle # # # # # # # # # #

anime_idle = Animation(
    file="girl_animation_logic_stand_idle.ozz",
    duration='2s',
    times=0,
)

anime_ready = Animation(
    file="girl_animation_logic_stand_ready.ozz",
    duration='2s',
    times=0,
)

ActionIdle(
    "Action.No1.Idle",
    anime_idle=anime_idle,
    anime_ready=anime_ready,
)

ActionIdle(
    "Action.No1.Idle2",
    arguments={"flag":Switch},
    enabled="flag",
    anime_idle=anime_idle,
    anime_ready=anime_ready,
)

# # # # # # # # # # Move # # # # # # # # # #

anime_move = Animation(
    file="run.ozz",
    duration='2s',
    times=0,
    body_progress=0,
)

ActionMove(
    "Action.No1.Run",
    enter_key=Run,
    anime_move=anime_move,
)

# # # # # # # # # # Dodge # # # # # # # # # #

ActionDodge(
    "Action.DodgeEmpty",
    enter_key=Dodge,
    perfect_start='0.4s',
    perfect_duration='0.6s',
    anime_forward=anime_empty,
    anime_back=anime_empty,
    anime_left=anime_empty,
    anime_right=anime_empty,
)

anime_dodge_forward = Animation(
    file="dodge_forward.ozz",
    duration='2s',
)

anime_dodge_back = Animation(
    file="dodge_back.ozz",
    duration='2s',
)

anime_dodge_left = Animation(
    file="dodge_left.ozz",
    duration='2s',
)

anime_dodge_right = Animation(
    file="dodge_right.ozz",
    duration='2s',
)

ActionDodge(
    "Action.No1.Dodge",
    arguments={"level": [0, 2]},
    enter_key=Dodge,
    perfect_start=IL('level', '0.4s', '0.35s', '0.3s'),
    perfect_duration=IL('level', '0.6s', '0.7s', '0.8s'),
    anime_forward=anime_dodge_forward,
    anime_back=anime_dodge_back,
    anime_left=anime_dodge_left,
    anime_right=anime_dodge_right,
)

# # # # # # # # # # Guard # # # # # # # # # #

ActionGuard(
    "Action.GuardEmpty",
    enter_key=Guard,
    guard_start='0.3s',
    perfect_start='0.7s',
    perfect_duration='0.6s',
    anime_enter=anime_empty,
    anime_leave=anime_empty,
    anime_move_forward=anime_empty,
    anime_move_back=anime_empty,
    anime_move_left=anime_empty,
    anime_move_right=anime_empty
)

# # # # # # # # # # Aim # # # # # # # # # #

# # # # # # # # # # General # # # # # # # # # #

ActionGeneral(
    "Action.GeneralEmpty",
    anime=anime_empty,
)

ActionGeneral(
    "Action.No1.Atk1",
    enter_key=Attack1,
    enter_level=LevelFree,
    derives={
        Attack1: ("Action.No1.Atk2", True),
    },
    anime=Animation(
        file="atk1.ozz",
        duration='5s',
    )
)

ActionGeneral(
    "Action.No1.Atk2",
    anime=Animation(
        file="atk2.ozz",
        duration='3s',
    )
)

ActionGeneral(
    "Action.No1.Skill",
    arguments={"just": Switch, "insertion": Switch, "level": [1,2]},
    enabled=True,
    enter_key=Skill1,
    enter_level=IL('level', LevelFree, LevelAttack),
    base_derive_level=LevelProgressing,
    derive_level=LevelSkill,
    derive_start='6s',
    derive_duration='4s',
    derives={
        DeriveLight: ("Action.No1.Atk2", True),
    },
    just_enabled='just',
    just_start=IL('level', '7s', time('7s') + 1),
    just_duration=IL('level', 5, 7),
    insertion_enabled='insertion',
    insertion_actions=InsertDodge | InsertGuard,
    insertion_derive_duration='2s',
    anime=Animation(
        file="skill.ozz",
        duration='8s',
    )
)

import {
    float,
    FPS,
    ID,
    int,
    parseAngleXz,
    parseBool,
    parseString,
    parseTime,
    TimelinePoint,
    TimelinePointArgs,
    TimelineRange,
    TimelineRangeArgs,
} from '../common';
import { Resource } from '../resource';
import { parseVarFloat, parseVarInt, Var, VarValueArgs } from '../variable';
import { Aniamtion, AniamtionArgs } from './animation';
import {
    Action,
    ActionArgs,
    ActionAttributes,
    ActionAttributesArgs,
    DeriveContinue,
    LEVEL_IDLE,
    parseActionAttributes,
    parseActionLevel,
    parseVarDeriveContinueSet,
} from './base';
import {
    DeriveRule,
    DeriveRuleArgs,
    parseDeriveRuleArray,
    verifyDeriveRuleArray,
    VirtualKeyDir,
    VirtualKeyDirArgs,
} from './keys';

// export type TimelineGeneralArgs = AniamtionArgs & {
//     /** 详细数值时间轴 */
//     attributes: ReadonlyArray<ReadonlyArray<int | string>>;

//     /** 派生等级时间轴 */
//     derive_levels: ReadonlyArray<ReadonlyArray<int | string>>;

//     // /** Just判定窗口 */
//     // just_window?: readonly [int | string, int | string];
// };

// export class TimelineGeneral extends Aniamtion {
//     /** 详细数值时间轴 */
//     public readonly attributes: ReadonlyArray<TimeFragment>;

//     /** 派生等级时间轴 */
//     public readonly derive_levels: ReadonlyArray<TimeFragment>;

//     // /** Just判定窗口 */
//     // public readonly just_window?: readonly [int, int];

//     public constructor(args: TimelineGeneralArgs, where: string) {
//         super(args, where);
//         this.attributes = TimeFragment.parseArray(args.attributes, `${where}.attributes`, {
//             duration: this.duration,
//         });
//         this.derive_levels = TimeFragment.parseArray(args.derive_levels, `${where}.derive_levels`, {
//             duration: this.duration,
//             over_duration: this.duration + 10 * FPS,
//         });
//         // this.just_window = !args.just_window
//         //     ? undefined
//         //     : parseTimeRange(args.just_window, where, { min: 0 });
//     }

//     public static override fromFile(_file: string, _where: string): TimelineGeneral {
//         return null as any;
//     }
// }

export type ActionGeneralMovetionArgs = ActionGeneralRootMotionArgs | ActionGeneralRotationArgs;
export type ActionGeneralMovetion = ActionGeneralRootMotion | ActionGeneralRotation;

export type ActionGeneralRootMotionArgs = {
    /** 是否启用Move轨道 */
    move?: boolean;
    /** 是否启用MoveEx轨道 */
    move_ex?: boolean;
};

export class ActionGeneralRootMotion {
    /** 是否启用Move轨道 */
    move: boolean;
    /** 是否启用MoveEx轨道 */
    move_ex: boolean;

    public constructor(args: ActionGeneralRootMotionArgs, where: string) {
        this.move = parseBool(args.move != null ? args.move : false, `${where}.move`);
        this.move_ex = parseBool(args.move_ex != null ? args.move_ex : false, `${where}.move_ex`);
    }

    public toJSON() {
        return { T: 'RootMotion', ...this };
    }
}

export type ActionGeneralRotationArgs = {
    /** 转身所需时间 */
    duration: float | string;
    /** 转身角度范围[-angle, angle] +angle表示前进移动 -angle表示后退移动 负号旋转范围不变 反转输入方向 */
    angle: float | string;
};

export class ActionGeneralRotation {
    /** 转身所需时间 */
    public duration: float;
    /** 转身角度范围[-angle, angle] +angle表示前进移动 -angle表示后退移动 负号旋转范围不变 反转输入方向 */
    public angle: float;

    public constructor(args: ActionGeneralRotationArgs, where: string) {
        this.duration = parseTime(args.duration, `${where}.duration`, { min: 0, max: 1000 });
        this.angle = parseAngleXz(args.angle, `${where}.angle`);
    }

    public toJSON() {
        return { T: 'Rotation', ...this };
    }
}

export type ActionGeneralArgs = ActionArgs & {
    anim_main: AniamtionArgs;

    /** 进入按键 */
    enter_key?: VirtualKeyDirArgs;

    /** 进入等级 */
    enter_level?: int;

    /** 冷却时间 每一轮冷却所需的时间 */
    cool_down_time?: float | VarValueArgs<float>;

    /** 冷却轮数 冷却后可储存的释放次数 */
    cool_down_round?: int | VarValueArgs<int>;

    /** 初始冷却轮数 初始状态下储存的释放次数 */
    cool_down_init_round?: int | VarValueArgs<int>;

    /** 用户输入控制的移动与旋转 */
    input_movements?: TimelinePointArgs<ActionGeneralMovetionArgs>;

    /** 各阶段详细数值配置 */
    attributes: TimelineRangeArgs<ActionAttributesArgs>;

    /** 各阶段派生等级 */
    derive_levels: TimelineRangeArgs<int | VarValueArgs<int>>;

    /** 派生列表 */
    derives?: ReadonlyArray<DeriveRuleArgs>;

    /** 可以继续当前动作派生的行为 */
    derive_continues?: ReadonlyArray<DeriveContinue> | VarValueArgs<ReadonlyArray<DeriveContinue>>;

    /** 攻击判定数值 */
    // hits: string | TimelineGeneralArgs;

    /** 动作过程中触发的事件 */
    custom_events?: TimelinePointArgs<string | VarValueArgs<string>>;
};

/**
 * 最普通的单次攻击动作
 */
export class ActionGeneral extends Action {
    public static override find(id: string, where: string): ActionGeneral {
        const res = Resource.find(id, where);
        if (!(res instanceof ActionGeneral)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 动画配置文件 */
    public readonly anim_main: Aniamtion;

    /** 进入按键 */
    public readonly enter_key?: VirtualKeyDir;

    /** 进入等级 */
    public readonly enter_level: int;

    /** 冷却时间 每一轮冷却所需的时间 */
    public readonly cool_down_time: float | Var<float>;

    /** 冷却轮数 冷却后可储存的释放次数 */
    public readonly cool_down_round: int | Var<int>;

    /** 初始冷却轮数 初始状态下储存的释放次数 */
    public readonly cool_down_init_round: int | Var<int>;

    /** 用户输入控制的移动与旋转 */
    public readonly input_movements?: TimelinePoint<ActionGeneralMovetion>;

    /** 各阶段详细数值配置 */
    public readonly attributes: TimelineRange<ActionAttributes>;

    /** 各阶段派生等级 */
    public readonly derive_levels: TimelineRange<int | Var<int>>;

    /** 派生列表 */
    public readonly derives?: ReadonlyArray<DeriveRule>;

    /** 可以继续当前动作派生的行为 */
    public readonly derive_continues?:
        | ReadonlyArray<DeriveContinue>
        | Var<ReadonlyArray<DeriveContinue>>;

    /** 动作过程中触发的事件 */
    public readonly custom_events?: TimelinePoint<string | Var<string>>;

    public constructor(id: ID, args: ActionGeneralArgs) {
        super(id, args);
        this.anim_main = new Aniamtion(args.anim_main, this.w('anim_main'), { root_motion: true });
        this.enter_key =
            args.enter_key == null
                ? undefined
                : new VirtualKeyDir(args.enter_key, this.w('enter_key'));
        this.enter_level = parseActionLevel(args.enter_level || LEVEL_IDLE, this.w('enter_level'));
        this.cool_down_time = parseVarFloat(args.cool_down_time || 0, this.w('cool_down_time'));
        this.cool_down_round = parseVarInt(args.cool_down_round || 1, this.w('cool_down_round'));
        this.cool_down_init_round = parseVarInt(
            args.cool_down_init_round || args.cool_down_round || 1,
            this.w('cool_down_init_round'),
        );
        this.input_movements = !args.input_movements
            ? undefined
            : new TimelinePoint(
                  args.input_movements,
                  this.w('input_movements'),
                  { duration: this.anim_main.duration },
                  {},
                  ActionGeneral.parseInputMovement,
              );
        this.attributes = new TimelineRange(
            args.attributes,
            this.w('attributes'),
            { duration: this.anim_main.duration },
            {},
            parseActionAttributes,
        );
        this.derive_levels = new TimelineRange(
            args.derive_levels,
            this.w('derive_levels'),
            { duration: this.anim_main.duration, over_duration: 5 * FPS },
            {},
            parseActionLevel,
        );
        this.derives = !args.derives
            ? undefined
            : parseDeriveRuleArray(args.derives, this.w('derives'));
        this.derive_continues = !args.derive_continues
            ? undefined
            : parseVarDeriveContinueSet(args.derive_continues, this.w('derive_continues'));
        this.custom_events = !args.custom_events
            ? undefined
            : new TimelinePoint(
                  args.custom_events,
                  this.w('custom_events'),
                  { duration: this.anim_main.duration },
                  {},
                  parseString,
              );

        Aniamtion.generateLocalID([this.anim_main]);
    }

    private static parseInputMovement(
        args: ActionGeneralMovetionArgs,
        where: string,
    ): ActionGeneralMovetion {
        if (
            (args as ActionGeneralRootMotionArgs).move != null ||
            (args as ActionGeneralRootMotionArgs).move_ex != null
        ) {
            return new ActionGeneralRootMotion(args as ActionGeneralRootMotionArgs, where);
        } else if ((args as ActionGeneralRotationArgs).angle != null) {
            return new ActionGeneralRotation(args as ActionGeneralRotationArgs, where);
        } else {
            throw new Error(`${where}: invalid input movement`);
        }
    }

    // private parseInputRotations(
    //     input_turns: undefined | ReadonlyArray<ActionGeneralRotationArgs>,
    //     duration: float,
    //     where: string,
    // ) {
    //     if (input_turns == null) {
    //         return [];
    //     }

    //     checkArray(input_turns, where, { max_len: 3 });
    //     let iter_time = 0;
    //     return input_turns.map((arg, idx) => {
    //         const rot = new ActionGeneralRotation(arg, `${where}[${idx}]`, { duration });
    //         if (rot.input_time < iter_time) {
    //             throw new Error(`${where}[${idx}].input_time must be ascend`);
    //         }
    //         return rot;
    //     });
    // }

    public override verify(): void {
        super.verify();

        if (this.derives) {
            verifyDeriveRuleArray(this.derives, { styles: this.styles }, this.w('derives'));
        }
    }
}

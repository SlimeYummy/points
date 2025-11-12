import {
    float,
    FPS,
    ID,
    int,
    parseFloat,
    parseFloatRange,
    Timeline,
    TimelineArgs,
} from '../common';
import { Resource } from '../resource';
import * as native from '../native';
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

    /** 运动极限距离范围 [min,max] */
    motion_distance?: float | readonly [float, float];

    /** 运动朝向角度范围 XZ平面内的旋转角度 */
    motion_toward?: float;

    /** 各阶段详细数值配置 */
    attributes: TimelineArgs<ActionAttributesArgs>;

    /** 各阶段派生等级 */
    derive_levels: TimelineArgs<int | VarValueArgs<int>>;

    /** 派生列表 */
    derives?: ReadonlyArray<DeriveRuleArgs>;

    /** 可以继续当前动作派生的行为 */
    derive_continues?: ReadonlyArray<DeriveContinue> | VarValueArgs<ReadonlyArray<DeriveContinue>>;

    /** 攻击判定数值 */
    // hits: string | TimelineGeneralArgs;
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

    /** 运动极限距离范围 [min,max] */
    public readonly motion_distance: readonly [float, float];

    /** 运动朝向角度范围 XZ平面内的旋转角度 */
    public readonly motion_toward: float;

    /** 各阶段详细数值配置 */
    public readonly attributes: Timeline<ActionAttributes>;

    /** 各阶段派生等级 */
    public readonly derive_levels: Timeline<int | Var<int>>;

    /** 派生列表 */
    public readonly derives?: ReadonlyArray<DeriveRule>;

    /** 可以继续当前动作派生的行为 */
    public readonly derive_continues?:
        | ReadonlyArray<DeriveContinue>
        | Var<ReadonlyArray<DeriveContinue>>;

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
        this.motion_distance = this.parseMotionDistance(args.motion_distance, this.anim_main);
        this.motion_toward =
            args.motion_toward == null
                ? 0
                : parseFloat(args.motion_toward, this.w('motion_toward'), { min: 0, max: 180 });
        this.attributes = new Timeline(
            args.attributes,
            this.w('attributes'),
            { duration: this.anim_main.duration },
            {},
            parseActionAttributes,
        );
        this.derive_levels = new Timeline(
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

        Aniamtion.generateLocalID([this.anim_main]);
    }

    private parseMotionDistance(
        motion_distance: undefined | float | readonly [float, float],
        anim_main: Aniamtion,
    ): readonly [float, float] {
        if (motion_distance == null) {
            if (anim_main.root_motion) {
                const meta = native.loadRootMotionMeta(anim_main.files);
                return [meta.whole_distance_xz, meta.whole_distance_xz];
            } else {
                return [0, 0];
            }
        } else if (typeof motion_distance === 'number') {
            const num = parseFloat(motion_distance, this.w('motion_distance'), {
                min: 0,
                max: 1000,
            });
            return [num, num];
        } else {
            return parseFloatRange(motion_distance, this.w('motion_distance'), {
                min: 0,
                max: 1000,
            });
        }
    }

    public override verify(): void {
        super.verify();

        if (this.derives) {
            verifyDeriveRuleArray(this.derives, { styles: this.styles }, this.w('derives'));
        }
    }
}

import { float, ID, int, parseBool, parseIDArray, parseTime, parseTimeRange } from '../common';
import { Resource } from '../resource';
import { Animation, AnimationArgs } from './animation';
import {
    Action,
    ActionArgs,
    LEVEL_ACTION,
    LEVEL_DERIVE,
    LEVEL_MOVE,
    parseActionLevel,
} from './base';
import {
    DeriveRule,
    DeriveRuleArgs,
    parseDeriveRuleArray,
    parseVirtualKey,
    verifyDeriveRuleArray,
    VirtualKey,
} from './keys';

// export type ActionDodgeRootMotionArgs = {
//     /** 是否启用Move轨道 */
//     move?: boolean;

//     /** 是否启用MoveEx轨道 */
//     move_ex?: boolean;
// };

// export class ActionDodgeRootMotion {
//     /** 是否启用Move轨道 */
//     move: boolean;

//     /** 是否启用MoveEx轨道 */
//     move_ex: boolean;

//     public constructor(args: ActionDodgeRootMotionArgs, where: string) {
//         this.move = parseBool(args.move != null ? args.move : false, `${where}.move`);
//         this.move_ex = parseBool(args.move_ex != null ? args.move_ex : false, `${where}.move_ex`);
//     }

//     public toJSON() {
//         return { T: 'RootMotion', ...this };
//     }
// }

export type ActionDodgeArgs = ActionArgs & {
    /** 进入按键 */
    enter_key?: VirtualKey;

    /** 进入等级 */
    enter_level?: int;

    /** 动画配置 */
    anim_dodge: AnimationArgs;

    /**
     * 平滑切换到闪避的动作列表
     * 主要平滑移动速度 动作过度尽量平滑
     */
    smooth_move_froms?: ReadonlyArray<ID>;

    /** 平滑切换时的不含腿部分的淡入时间 （腿部会硬切换 不淡入） */
    smooth_no_leg_fade_in?: float | string;

    /** 平滑切换时平滑移动速度的时间 */
    smooth_speed_duration?: float | string;

    /** 闪避帧判定的时间范围 */
    dodge_time: [float | string, float | string];

    /**
     * 派生时间
     * - quick: 快速派生 用于冲刺/闪攻等
     * - normal: 普通派生
     */
    derive_times: {
        quick: float | string;
        normal: float | string;
    };

    /**
     * 维持等级
     * - dodge: 闪避阶段
     * - quick: 快速派生阶段
     * - normal: 普通阶段
     */
    keep_levels?: {
        dodge: int;
        quick: int;
        normal: int;
    };

    /** 派生列表 */
    derives?: ReadonlyArray<DeriveRuleArgs>;
};

/**
 * 闪避动作 玩家专用
 */
export class ActionDodge extends Action {
    public static override find(id: string, where: string): ActionDodge {
        const res = Resource.find(id, where);
        if (!(res instanceof ActionDodge)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 进入按键 */
    public readonly enter_key?: VirtualKey;

    /** 进入等级 */
    public readonly enter_level: int;

    /** 闪避动画 */
    public readonly anim_dodge: Animation;

    /**
     * 平滑切换到闪避的动作列表
     * 主要平滑移动速度 动作过度尽量平滑
     */
    public readonly smooth_move_froms?: ReadonlyArray<ID>;

    /** 平滑切换时的不含腿部分的淡入时间 （腿部会硬切换 不淡入） */
    public readonly smooth_no_leg_fade_in: float;

    /** 平滑切换时平滑移动速度的时间 */
    public readonly smooth_speed_duration: float;

    /** 闪避判定的时间范围 */
    public readonly dodge_time: readonly [float, float];

    /**
     * 派生时间
     * - quick: 快速派生 用于冲刺/闪攻等
     * - normal: 普通派生
     */
    public readonly derive_times: {
        readonly quick: float;
        readonly normal: float;
    };

    /**
     * 维持等级
     * - dodge: 闪避阶段
     * - quick: 快速派生阶段
     * - normal: 普通阶段
     */
    public readonly keep_levels: {
        readonly dodge: int;
        readonly quick: int;
        readonly normal: int;
    };

    /** 派生列表 */
    public readonly derives?: ReadonlyArray<DeriveRule>;

    public constructor(id: ID, args: ActionDodgeArgs) {
        super(id, args, { character: 'player' });
        this.enter_key =
            args.enter_key == null
                ? undefined
                : parseVirtualKey(args.enter_key, this.w('enter_key'));
        this.enter_level = parseActionLevel(
            args.enter_level || LEVEL_DERIVE,
            this.w('enter_level'),
        );

        this.anim_dodge = new Animation(args.anim_dodge, this.w('anim_dodge'), {
            root_motion: true,
        });
        this.smooth_move_froms = !args.smooth_move_froms
            ? undefined
            : parseIDArray(args.smooth_move_froms, 'Action', this.w('smooth_move_froms'));
        this.smooth_no_leg_fade_in = parseTime(
            args.smooth_no_leg_fade_in ?? 0,
            this.w('smooth_no_leg_fade_in'),
            {
                min: 0,
                max: this.anim_dodge.duration,
                type: 'f32',
            },
        );
        this.smooth_speed_duration = parseTime(
            args.smooth_speed_duration ?? 0,
            this.w('smooth_speed_duration'),
            {
                min: 0,
                max: this.anim_dodge.duration,
                type: 'f32',
            },
        );
        this.dodge_time = parseTimeRange(args.dodge_time, this.w('dodge_time'), {
            min: 0,
            max: this.anim_dodge.duration,
            type: 'f32',
        });

        const derive_time_normal = parseTime(
            args.derive_times.normal,
            this.w('derive_times.normal'),
            {
                min: 0,
                max: this.anim_dodge.duration,
                type: 'f32',
            },
        );
        const derive_time_quick = parseTime(
            args.derive_times.quick || args.derive_times.normal,
            this.w('derive_times.quick'),
            { min: 0, max: derive_time_normal, type: 'f32' },
        );
        this.derive_times = {
            quick: derive_time_quick,
            normal: derive_time_normal,
        };

        this.keep_levels = {
            dodge: parseActionLevel(
                args.keep_levels?.dodge ?? LEVEL_ACTION,
                this.w('keep_levels.dodge'),
            ),
            quick: parseActionLevel(
                args.keep_levels?.quick ?? LEVEL_DERIVE,
                this.w('keep_levels.quick'),
            ),
            normal: parseActionLevel(
                args.keep_levels?.normal ?? LEVEL_MOVE,
                this.w('keep_levels.normal'),
            ),
        };

        this.derives = !args.derives
            ? undefined
            : parseDeriveRuleArray(args.derives, this.w('derives'));
        Animation.generateLocalID([this.anim_dodge]);
    }

    public override verify() {
        super.verify();
        if (this.derives) {
            verifyDeriveRuleArray(this.derives, { styles: this.styles }, this.w('derives'));
        }
    }
}

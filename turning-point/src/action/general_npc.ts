import {
    float,
    ID,
    int,
    parseAngleXz,
    parseFloat,
    parseFloatRange,
    parseString,
    parseTime,
    TimelinePoint,
    TimelinePointArgs,
    TimelineRange,
    TimelineRangeArgs,
} from '../common';
import { Resource } from '../resource';
import { Animation, AnimationArgs } from './animation';
import {
    Action,
    ActionArgs,
    ActionAttributes,
    ActionAttributesArgs,
    parseActionAttributes,
    parseActionLevel,
} from './base';
import { Hit, HitArgs } from './hit_attr';

export type ActionGeneralNpcTranslationArgs = {
    /** 调节移动距离的时长 */
    duration: float | string;

    /** 淡入淡出的时间比例 */
    fade_ratio?: float | string;

    /** 与目标的距离区间 根据该区间选择速度 */
    distance: ReadonlyArray<float | string>;

    /** 移动速度系数区间 */
    speed_ratio: ReadonlyArray<float | string>;
};

export class ActionGeneralNpcTranslation {
    /** 调节移动距离所需时间 */
    public readonly duration: float;

    /** 淡入淡出的时间比例 */
    public readonly fade_ratio: float;

    /** 与目标的距离区间 根据该区间选择速度 */
    public readonly distance: readonly [float, float];

    /** 移动速度系数区间 */
    public readonly speed_ratio: readonly [float, float];

    public constructor(args: ActionGeneralNpcTranslationArgs, where: string) {
        this.duration = parseTime(args.duration, `${where}.duration`, { min: 0, type: 'f32' });
        this.fade_ratio = parseFloat(args.fade_ratio ?? 0.1, `${where}.fade_ratio`, {
            min: 0,
            max: 0.5,
            type: 'f32',
        });
        this.distance = parseFloatRange(args.distance, `${where}.distance`, {
            min: 0,
            type: 'f32',
        });
        this.speed_ratio = parseFloatRange(args.speed_ratio, `${where}.speed_ratio`, {
            min: 0,
            max: 100,
            type: 'f32',
        });
    }

    public toJSON() {
        return { T: 'Translation', ...this };
    }
}

export type ActionGeneralNpcRotationArgs = {
    /** 调节旋转方向所需时间 */
    duration: float | string;

    /** 最大旋转角度 表示区间[-angle, angle]内角度范围 */
    max_angle: float | string;
};

export class ActionGeneralNpcRotation {
    /** 调节旋转方向所需时间 */
    public readonly duration: float;

    /** 最大旋转角度 表示区间[-angle, angle]内角度范围 */
    public readonly max_angle: float;

    public constructor(args: ActionGeneralNpcRotationArgs, where: string) {
        this.duration = parseTime(args.duration, `${where}.duration`, { min: 0, type: 'f32' });
        this.max_angle = parseAngleXz(args.max_angle, `${where}.max_angle`);
    }

    public toJSON() {
        return { T: 'Rotation', ...this };
    }
}

export type ActionGeneralNpcMovementArgs =
    | ActionGeneralNpcTranslationArgs
    | ActionGeneralNpcRotationArgs;
export type ActionGeneralNpcMovement = ActionGeneralNpcTranslation | ActionGeneralNpcRotation;

export type ActionGeneralNpcArgs = ActionArgs & {
    /** 动画配置 */
    anim_main: AnimationArgs;

    /** AI控制的移动调节 */
    adjust_movements?: TimelinePointArgs<ActionGeneralNpcMovementArgs>;

    // /** 各阶段详细数值配置 */
    // attributes: TimelineRangeArgs<ActionAttributesArgs>;

    /** 各阶段维持等级 */
    keep_levels: TimelineRangeArgs<int>;

    /** 攻击判定表 */
    hits?: ReadonlyArray<HitArgs>;

    /** 动作过程中触发的事件 */
    custom_events?: TimelinePointArgs<string>;
};

/**
 * 最普通的单次攻击动作 NPC专用
 */
export class ActionGeneralNpc extends Action {
    public static override find(id: string, where: string): ActionGeneralNpc {
        const res = Resource.find(id, where);
        if (!(res instanceof ActionGeneralNpc)) {
            throw new Error(`${where}: Resource type mismatch`);
        }
        return res;
    }

    /** 动画配置 */
    public readonly anim_main: Animation;

    /** AI控制的移动调节 */
    public readonly adjust_movements?: TimelinePoint<ActionGeneralNpcMovement>;

    // /** 各阶段详细数值配置 */
    // public readonly attributes: TimelineRange<ActionAttributes>;

    /** 各阶段维持等级 */
    public readonly keep_levels: TimelineRange<int>;

    /** 攻击判定表 */
    public readonly hits?: ReadonlyArray<Hit>;

    /** 动作过程中触发的事件 */
    public readonly custom_events?: TimelinePoint<string>;

    public constructor(id: ID, args: ActionGeneralNpcArgs) {
        super(id, args, { character: 'npc' });
        this.anim_main = new Animation(args.anim_main, this.w('anim_main'), {
            root_motion: true,
            hit_motion: args.hits != null,
        });
        this.adjust_movements = !args.adjust_movements
            ? undefined
            : new TimelinePoint(
                  args.adjust_movements,
                  this.w('adjust_movements'),
                  { duration: this.anim_main.duration, type: 'f32' },
                  {},
                  ActionGeneralNpc.parseMovement,
              );
        // this.attributes = new TimelineRange(
        //     args.attributes,
        //     this.w('attributes'),
        //     { duration: this.anim_main.duration, type: 'f32' },
        //     {},
        //     parseActionAttributes,
        // );
        this.keep_levels = new TimelineRange(
            args.keep_levels,
            this.w('keep_levels'),
            { duration: this.anim_main.duration, type: 'f32' },
            {},
            parseActionLevel,
        );
        this.hits =
            args.hits == null
                ? undefined
                : Hit.parseArray(args.hits ?? [], this.w('hits'), { files: this.anim_main.files });
        this.custom_events = !args.custom_events
            ? undefined
            : new TimelinePoint(
                  args.custom_events,
                  this.w('custom_events'),
                  { duration: this.anim_main.duration, type: 'f32' },
                  {},
                  parseString,
              );

        Animation.generateLocalID([this.anim_main]);
    }

    private static parseMovement(
        args: ActionGeneralNpcMovementArgs,
        where: string,
    ): ActionGeneralNpcMovement {
        if ((args as any).T === 'Translation' || (args as any).distance != null) {
            return new ActionGeneralNpcTranslation(args as ActionGeneralNpcTranslationArgs, where);
        } else if ((args as any).T === 'Rotation' || (args as any).max_angle != null) {
            return new ActionGeneralNpcRotation(args as ActionGeneralNpcRotationArgs, where);
        } else {
            throw new Error(`${where}: invalid adjust movement`);
        }
    }

    public override verify(): void {
        super.verify();
        if (this.hits) {
            Hit.verifyArray(this.hits, { styles: this.styles }, this.w('hits'));
        }
    }
}

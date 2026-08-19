import {
    float,
    ID,
    int,
    parseAngleXz,
    parseFloatRange,
    parseString,
    parseTime,
    parseTimeRange,
    TimelineRange,
    TimelineRangeArgs,
} from '../common';
import { Resource } from '../resource';
import { Animation, AnimationArgs } from './animation';
import { Action, ActionArgs, parseActionLevel } from './base';

export type ActionDodgeNpcDodgeArgs = AnimationArgs & {
    /** 进入该动画的角度 角色朝向与闪避方向的夹角（右手系XZ平面） */
    enter_angle: float | string;

    /**
     * 旋转参考 角色自身/锁定目标角色
     * 注意：和其他动作的旋转不同，闪避的旋转仅影响角色朝向，不影响实际位移方向。
     */
    rotation_reference?: 'None' | 'Character' | 'TargetCharacter';

    /** 旋转开始时间 */
    rotation_start?: float | string;

    /** 旋转所需时间范围 转角越大时间越长 [min,max] */
    rotation_duration?: ReadonlyArray<float | string>;

    /** 最大旋转角度 表示区间[-angle, angle]内角度范围 */
    rotation_max_angle?: float | string;

    /** 各阶段维持等级 */
    keep_levels: TimelineRangeArgs<int>;
};

export class ActionDodgeNpcDodge {
    /** 闪避动画 */
    public readonly anim: Animation;

    /** 进入该动画的角度 角色朝向与闪避方向的夹角（右手系XZ平面） */
    public readonly enter_angle: float;

    /**
     * 旋转参考 角色自身/锁定目标角色
     * 注意：和其他动作的旋转不同，闪避的旋转仅影响角色朝向，不影响实际位移方向。
     */
    public readonly rotation_reference: 'None' | 'Character' | 'TargetCharacter';

    /** 旋转开始时间 */
    public readonly rotation_start: float;

    /** 旋转所需时间范围 转角越大时间越长 [min,max] */
    public readonly rotation_duration: readonly [float, float];

    /** 最大旋转角度 表示区间[-angle, angle]内角度范围 */
    public readonly rotation_max_angle: float;

    /** 各阶段维持等级 */
    public readonly keep_levels: TimelineRange<int>;

    public constructor(args: ActionDodgeNpcDodgeArgs, where: string) {
        this.anim = new Animation(args, where, { root_motion: true });
        this.enter_angle = parseAngleXz(args.enter_angle, `${where}.enter_angle`);
        this.rotation_reference = parseString(
            args.rotation_reference ?? 'None',
            `${where}.rotation_reference`,
            {
                includes: ['None', 'Character', 'TargetCharacter'],
            },
        ) as any;
        this.rotation_start = parseTime(args.rotation_start ?? 0, `${where}.rotation_start`, {
            min: 0,
            type: 'f32',
        });
        this.rotation_duration = parseTimeRange(
            args.rotation_duration ?? [0, 0],
            `${where}.rotation_duration`,
            { min: 0, type: 'f32' },
        );
        this.rotation_max_angle = parseAngleXz(
            args.rotation_max_angle ?? 0,
            `${where}.rotation_max_angle`,
        );
        this.keep_levels = new TimelineRange(
            args.keep_levels,
            `${where}.keep_levels`,
            { duration: this.anim.duration, type: 'f32' },
            {},
            parseActionLevel,
        );
    }

    public static parseArray(
        args: ReadonlyArray<ActionDodgeNpcDodgeArgs>,
        where: string,
    ): ReadonlyArray<ActionDodgeNpcDodge> {
        if (args.length < 1) {
            throw new Error(`${where}: length must >= 1`);
        }
        const dodges = args.map((args, idx) => new ActionDodgeNpcDodge(args, `${where}[${idx}]`));
        dodges.sort((a, b) => a.enter_angle - b.enter_angle);
        return dodges;
    }
}

export type ActionDodgeNpcArgs = ActionArgs & {
    /** 移动距离范围 AI在此范围内调节闪避距离 [min, max] */
    move_distance: ReadonlyArray<float | string>;

    /** 各方向闪避动画 */
    anim_dodges: ReadonlyArray<ActionDodgeNpcDodgeArgs>;
};

/**
 * 闪避动作 NPC专用
 */
export class ActionDodgeNpc extends Action {
    public static override find(id: string, where: string): ActionDodgeNpc {
        const res = Resource.find(id, where);
        if (!(res instanceof ActionDodgeNpc)) {
            throw new Error(`${where}: Resource type mismatch`);
        }
        return res;
    }

    /** 移动距离范围（m） */
    public readonly move_distance: readonly [float, float];

    /** 各方向闪避动画 */
    public readonly anim_dodges: ReadonlyArray<ActionDodgeNpcDodge>;

    public constructor(id: ID, args: ActionDodgeNpcArgs) {
        super(id, args, { character: 'npc' });
        this.move_distance = parseFloatRange(args.move_distance, this.w('move_distance'), {
            min: 0,
            type: 'f32',
        });
        this.anim_dodges = ActionDodgeNpcDodge.parseArray(args.anim_dodges, this.w('anim_dodges'));
        Animation.generateLocalID(this.anim_dodges.map((d) => d.anim));
    }
}

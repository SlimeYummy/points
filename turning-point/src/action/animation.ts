import { FilePath, float, int, parseBool, parseFile, parseTime } from '../common';
import * as native from '../native';

export type AnimationArgs = {
    /**
     * 动画文件 一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.la-ozz 逻辑动画
     * - xxx.va-ozz 视图动画
     * - xxx.rm-ozz 根运动RootMotion
     * - xxx.wm-ozz 武器轨迹
     * - xxx.hm-rkyv/xxx.hm-json 攻击判定盒
     */
    files: FilePath;

    /** 动画时长（单位秒） */
    duration?: float | string;

    /** 淡入动画时间（单位秒） */
    fade_in?: float | string;

    /** 是否启用RootMotion */
    root_motion?: boolean;

    /** 是否启用武器轨迹 */
    weapon_motion?: boolean;

    /** 是否启用命中判定盒 */
    hit_motion?: boolean;
};

export class Animation {
    /**
     * 动画文件 一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.la-ozz 逻辑动画
     * - xxx.va-ozz 视图动画
     * - xxx.rm-ozz 根运动RootMotion
     * - xxx.wm-ozz 武器轨迹
     * - xxx.hm-rkyv/xxx.hm-json 攻击判定盒
     */
    public readonly files: FilePath;

    /** 动作内部的短ID */
    public readonly local_id: int;

    /**
     * 动画时长（单位秒）
     * 当动画文件内时常与duration不一致时 会将时长缩放为duration
     */
    public readonly duration: float;

    /** 淡入动画时间（单位秒） */
    public readonly fade_in: float;

    /** 是否启用RootMotion */
    public readonly root_motion: boolean;

    /** 是否启用武器轨迹 */
    public readonly weapon_motion: boolean;

    /** 是否启用命中判定盒 */
    public readonly hit_motion: boolean;

    public constructor(
        args: AnimationArgs,
        where: string,
        opts: {
            root_motion?: boolean;
            weapon_motion?: boolean;
            hit_motion?: boolean;
        } = {},
    ) {
        this.files = parseFile(args.files, `${where}.files`, { extension: '.*' });
        // 确保动画存在
        const anim = native.loadAnimationMeta(
            this.files,
            `${where}.files: file not found (${this.files})`,
        );

        this.duration = this.parseDuration(anim, args.duration, `${where}.duration`);
        this.fade_in =
            args.fade_in == null ? 0.1 : parseTime(args.fade_in, `${where}.fade_in`, { min: 0 });
        this.fade_in = Math.min(this.fade_in, this.duration);

        this.root_motion =
            args.root_motion == null ? false : parseBool(args.root_motion, `${where}.root_motion`);
        if (opts.root_motion !== undefined && this.root_motion !== opts.root_motion) {
            throw new Error(`${where}.root_motion: must be ${!!opts.root_motion}`);
        }
        if (this.root_motion) {
            // 确保RootMotion存在
            native.loadRootMotionMeta(this.files, `${where}.files: file not found (${this.files})`);
        }

        this.weapon_motion =
            args.weapon_motion == null
                ? false
                : parseBool(args.weapon_motion, `${where}.weapon_motion`);
        if (opts.weapon_motion !== undefined && this.weapon_motion !== opts.weapon_motion) {
            throw new Error(`${where}.weapon_motion: must be ${!!opts.weapon_motion}`);
        }

        this.hit_motion =
            args.hit_motion == null ? false : parseBool(args.hit_motion, `${where}.hit_motion`);
        if (opts.hit_motion !== undefined && this.hit_motion !== opts.hit_motion) {
            throw new Error(`${where}.hit_motion: must be ${!!opts.hit_motion}`);
        }

        this.local_id = 65535;
    }

    private parseDuration(
        anim: native.AnimationMeta,
        duration: undefined | float | string,
        where: string,
    ): float {
        if (duration == null) {
            return anim.duration;
        } else if (typeof duration === 'string' && duration.endsWith('!')) {
            return parseTime(duration.slice(0, -1), `${where}.duration`, { min: 0 });
        } else {
            const dura = parseTime(duration, `${where}.duration`, { min: 0 });
            if (Math.abs(dura - anim.duration) > 1e-4) {
                console.warn(`Warning: ${where}: duration mismatch (${duration} != ${anim.duration}s)`);
            }
            return dura;
        }
    }

    public static parseArray(
        raw: ReadonlyArray<AnimationArgs>,
        where: string,
    ): ReadonlyArray<Animation> {
        return raw.map((args, idx) => new Animation(args, `${where}[${idx}]`));
    }

    public static generateLocalID(animation: Array<Animation | undefined | null>) {
        for (let pos = 0; pos < animation.length; ++pos) {
            if (animation[pos]) {
                (animation[pos] as any).local_id = pos;
            }
        }
    }
}

import { FilePath, float, int, parseBool, parseFile, parseString, parseTime } from '../common';
import * as native from '../native';

export type AnimationArgs = {
    /**
     * 动画文件 一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.la-ozz 逻辑动画
     * - xxx.va-ozz 视图动画
     * - xxx.rm-ozz 根运动RootMotion
     * - xxx.wc-ozz 武器轨迹
     * - xxx.sk-ozz 形态键
     * - xxx.hm-rkyv/xxx.hm-json 攻击判定盒
     */
    files: FilePath;

    /**
     * 动画时长
     * 默认要与动画文件一致 使用'10s!'形式强制指定时长
     */
    duration?: float | string;

    /**
     * 默认淡入动画时间
     * 主要用于Action动画间切换 同一Action内由Action自身逻辑空控制
     */
    fade_in?: float | string;

    /** 是否启用RootMotion */
    root_motion?: boolean;

    /** 是否启用武器轨迹 */
    weapon_control?: boolean;

    /** 是否启用命中判定盒 */
    hit_motion?: boolean;

    /** 是否启用形态键 */
    shape_key?: boolean;

    /** 是否是叠加混合 */
    additive_blending?: boolean;

    /** 关节权重配置名称 */
    joint_weights?: string;
};

export class Animation {
    /**
     * 动画文件 一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.la-ozz 逻辑动画
     * - xxx.va-ozz 视图动画
     * - xxx.rm-ozz 根运动RootMotion
     * - xxx.wc-ozz 武器轨迹
     * - xxx.hm-rkyv/xxx.hm-json 攻击判定盒
     */
    public readonly files: FilePath;

    /** 动作内部的短ID */
    public readonly local_id: int;

    /**
     * 动画时长
     * 默认要与动画文件一致 使用'10s!'形式强制指定时长
     */
    public readonly duration: float;

    /** 默认淡入动画时间 */
    public readonly fade_in: float;

    /** 是否启用RootMotion */
    public readonly root_motion: boolean;

    /** 是否启用武器轨迹 */
    public readonly weapon_control: boolean;

    /** 是否启用命中判定盒 */
    public readonly hit_motion: boolean;

    /** 是否启用形态键 */
    public readonly shape_key: boolean;

    /** 是否是叠加混合 */
    public readonly additive_blending: boolean;

    public constructor(
        args: AnimationArgs,
        where: string,
        opts: {
            root_motion?: boolean;
            weapon_control?: boolean;
            hit_motion?: boolean;
            additive_blending?: boolean;
        } = {},
    ) {
        this.files = parseFile(args.files, `${where}.files`, { extension: '.*' });
        // 确保动画存在
        const anim = native.loadAnimationMeta(
            this.files,
            `${where}.files: file corrupted or not found (${this.files})`,
        );

        this.duration = this.parseDuration(anim, args.duration, `${where}.duration`);
        this.fade_in = parseTime(args.fade_in ?? 0.1, `${where}.fade_in`, {
            min: 0,
            type: 'f32',
        });
        this.fade_in = Math.min(this.fade_in, this.duration);

        this.root_motion = parseBool(args.root_motion ?? false, `${where}.root_motion`);
        if (opts.root_motion !== undefined && this.root_motion !== opts.root_motion) {
            throw new Error(`${where}.root_motion: must be ${!!opts.root_motion}`);
        }
        if (this.root_motion) {
            // 确保RootMotion存在
            native.loadRootMotionMeta(
                this.files,
                `${where}.files: file corrupted or not found (${this.files})`,
            );
        }

        this.weapon_control = parseBool(args.weapon_control ?? false, `${where}.weapon_control`);
        if (opts.weapon_control !== undefined && this.weapon_control !== opts.weapon_control) {
            throw new Error(`${where}.weapon_control: must be ${!!opts.weapon_control}`);
        }

        this.hit_motion = parseBool(args.hit_motion ?? false, `${where}.hit_motion`);
        if (opts.hit_motion !== undefined && this.hit_motion !== opts.hit_motion) {
            throw new Error(`${where}.hit_motion: must be ${!!opts.hit_motion}`);
        }

        this.shape_key = parseBool(args.shape_key ?? false, `${where}.shape_key`);

        this.additive_blending = parseBool(
            args.additive_blending ?? false,
            `${where}.additive_blending`,
        );
        if (
            opts.additive_blending !== undefined &&
            this.additive_blending !== opts.additive_blending
        ) {
            throw new Error(`${where}.additive_blending: must be ${!!opts.additive_blending}`);
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
            return parseTime(duration.slice(0, -1), where, { min: 0, type: 'f32' });
        } else {
            const dura = parseTime(duration, where, { min: 0, type: 'f32' });
            if (Math.abs(dura - anim.duration) > 1e-4) {
                console.warn(
                    `Warning: ${where}: duration mismatch (${duration} != ${anim.duration}s)`,
                );
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

    public calcSpeedRatio(move_speed: float, where: string): float {
        if (!this.root_motion) {
            throw new Error(`${where}: Animation has no root motion`);
        }

        const meta = native.loadRootMotionMeta(this.files);
        if (meta.position_default == null) {
            throw new Error(`${where}: no 'Default' in root motion`);
        }
        const whole_distance_xz = Math.abs(meta.position_default.whole_distance_xz);
        if (whole_distance_xz <= 0) {
            return 1;
        }
        const rm_speed = whole_distance_xz / this.duration;
        return move_speed / rm_speed;
    }
}

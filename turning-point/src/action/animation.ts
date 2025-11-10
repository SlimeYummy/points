import { FilePath, float, int, parseBool, parseFile, parseTime } from '../common';
import * as native from '../native';

export type AniamtionArgs = {
    /**
     * 动画文件 一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.la-ozz 逻辑动画
     * - xxx.va-ozz 视图动画
     * - xxx.rm-ozz 根运动RootMotion
     * - xxx.wm-ozz 武器轨迹
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
};

export class Aniamtion {
    /**
     * 动画文件 一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.la-ozz 逻辑动画
     * - xxx.va-ozz 视图动画
     * - xxx.rm-ozz 根运动RootMotion
     * - xxx.wm-ozz 武器轨迹
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

    public constructor(
        args: AniamtionArgs,
        where: string,
        opts?: {
            root_motion?: boolean;
            weapon_motion?: boolean;
        },
    );
    public constructor(
        files: string,
        duration: int,
        fade_in?: int,
        root_motion?: boolean,
        weapon_motion?: boolean,
    );
    public constructor() {
        if (typeof arguments[0] === 'object') {
            const args: AniamtionArgs = arguments[0];
            const where: string = arguments[1];
            const opts = arguments[2] || {};

            this.files = parseFile(args.files, `${where}.files`, { extension: '.*' });
            const anim = native.loadAnimationMeta(this.files); // 确保动画存在

            this.duration = parseTime(
                args.duration == null ? anim.duration : args.duration,
                `${where}.duration`,
                { min: 0 },
            );
            this.fade_in =
                args.fade_in == null
                    ? 0.1
                    : parseTime(args.fade_in, `${where}.fade_in`, { min: 0 });

            this.root_motion =
                args.root_motion == null
                    ? false
                    : parseBool(args.root_motion, `${where}.root_motion`);
            if (opts.root_motion !== undefined && this.root_motion !== opts.root_motion) {
                throw new Error(`${where}.root_motion: must be ${!!opts.root_motion}`);
            }

            this.weapon_motion = args.weapon_motion == null ? false : parseBool(args.weapon_motion, `${where}.weapon_motion`);
            if (opts.weapon_motion !== undefined && this.weapon_motion !== opts.weapon_motion) {
                throw new Error(`${where}.weapon_motion: must be ${!!opts.weapon_motion}`);
            }
        } else {
            this.files = arguments[0];
            this.duration = arguments[1];
            this.fade_in = arguments[2] || 0.2;
            this.root_motion = arguments[3] || false;
            this.weapon_motion = arguments[4] || false;
        }

        if (this.root_motion) {
            native.loadRootMotionMeta(this.files); // 确保RootMotion存在
        }

        this.local_id = 65535;
    }

    public static fromFile(_file: string, _where: string): Aniamtion {
        return null as any;
    }

    public static parseArray(
        raw: ReadonlyArray<AniamtionArgs>,
        where: string,
    ): ReadonlyArray<Aniamtion> {
        return raw.map((args, idx) => new Aniamtion(args, `${where}[${idx}]`));
    }

    public static generateLocalID(animation: Array<Aniamtion | undefined | null>) {
        for (let pos = 0; pos < animation.length; ++pos) {
            if (animation[pos]) {
                (animation[pos] as any).local_id = pos;
            }
        }
    }
}

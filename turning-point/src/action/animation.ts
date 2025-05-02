import { float, int, parseBool, parseFile, parseTime } from '../common';
import * as native from '../native';

export type AniamtionArgs = {
    /**
     * 动画文件 一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.logic-anim.ozz 逻辑动画
     * - xxx.logic-moti.ozz 逻辑RootMotion
     * - xxx.view-anim.ozz 视图动画
     * - xxx.view-moti.ozz 视图RootMotion
     */
    files: string;

    /** 动画时长（单位秒） */
    duration: float | string;

    /** 淡入动画时间（单位秒） */
    fade_in?: float | string;

    /** 是否启用RootMotion */
    root_motion?: boolean;
};

export class Aniamtion {
    /**
     * 动画文件 一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.logic-anim.ozz 逻辑动画
     * - xxx.logic-moti.ozz 逻辑RootMotion
     * - xxx.view-anim.ozz 视图动画
     * - xxx.view-moti.ozz 视图RootMotion
     * - xxx.hits.rkyv (.json)判定时间轴
     */
    public readonly files: string;

    /**
     * 动画时长（单位秒）
     * 当动画文件内时常与duration不一致时 会将时长缩放为duration
     */
    public readonly duration: float;

    /** 淡入动画时间（单位秒） */
    public readonly fade_in: float;

    /** 是否启用RootMotion */
    public readonly root_motion: boolean;

    /** RootMotion中Root在xz平面的最大移动距离 */
    public readonly root_max_distance: float;

    public constructor(
        args: AniamtionArgs,
        where: string,
        opts?: {
            root_motion?: boolean;
        },
    );
    public constructor(files: string, duration: int, fade_in?: int, root_motion?: boolean);
    public constructor() {
        if (typeof arguments[0] === 'object') {
            const args: AniamtionArgs = arguments[0];
            const where: string = arguments[1];
            const opts = arguments[2] || {};

            this.files = parseFile(args.files, `${where}.files`);
            this.duration = parseTime(args.duration, `${where}.duration`, { min: 0 });
            this.fade_in =
                args.fade_in == null
                    ? 0.2
                    : parseTime(args.fade_in, `${where}.fade_in`, { min: 0 });

            this.root_motion =
                args.root_motion == null
                    ? false
                    : parseBool(args.root_motion, `${where}.root_motion`);
            if (opts.root_motion !== undefined && this.root_motion !== opts.root_motion) {
                throw new Error(`${where}.root_motion: must be ${!!opts.root_motion}`);
            }
        } else {
            this.files = arguments[0];
            this.duration = arguments[1];
            this.fade_in = arguments[2] || 0.2;
            this.root_motion = arguments[3] || false;
        }

        if (this.root_motion) {
            const { max_distance } = native.loadRootMotionMeta(this.files);
            this.root_max_distance = max_distance;
        } else {
            this.root_max_distance = 0;
        }
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
}

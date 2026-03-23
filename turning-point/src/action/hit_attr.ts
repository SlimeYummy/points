import {
    float,
    ID,
    int,
    MAX_HIT_TIMES,
    MAX_HIT_TIMES_PER_FRAME,
    parseString,
    SPF,
} from '../common';
import * as native from '../native';
import { parseVarInt, parseVarTime, Var, VarValueArgs, verifyVarValue } from '../variable';

export type HitArgs = {
    /** 该判定在HitMotion中对应的分组 */
    group: string;

    /** 单个判定体(HitBox)的最大判定次数 */
    box_max_times?: int | VarValueArgs<int>;

    /** 单个判定体(HitBox)两次判定间的最小时间间隔 */
    box_min_interval?: float | string | VarValueArgs<float | string>;

    /** 整个判定组(HitGroup)内所有判定体的共计最大判定次数 */
    group_max_times?: int | VarValueArgs<int>;

    /** 属性 伤害等效果 */
    attributes?: any;
};

export class Hit {
    /** 该判定在HitMotion中对应的分组 */
    public group: string;

    /** 单个判定体(HitBox)的最大判定次数 */
    public box_max_times: int | Var<int>;

    /** 单个判定体(HitBox)两次判定间的最小时间间隔 */
    public box_min_interval: float | Var<float>;

    /** 整个判定组(HitGroup)内所有判定体的共计最大判定次数 */
    public group_max_times: int | Var<int>;

    #default: boolean = false;

    public constructor(
        args: HitArgs,
        where: string,
        opts: {
            files?: string;
        } = {},
    ) {
        const hm =
            opts.files &&
            native.loadHitMotionMeta(opts.files, `${where}.files: file not found (${opts.files})`);

        this.group = parseString(args.group, `${where}.group`, {
            includes: !hm ? undefined : hm.groups.map((g) => g.group),
        });
        this.box_max_times =
            args.box_max_times == null
                ? 0
                : parseVarInt(args.box_max_times, `${where}.box_max_times`, {
                      min: 0,
                      max: MAX_HIT_TIMES,
                  });
        this.box_min_interval =
            args.box_min_interval == null
                ? 1e10
                : parseVarTime(args.box_min_interval, `${where}.box_min_interval`, {
                      min: SPF / MAX_HIT_TIMES_PER_FRAME,
                  });
        this.group_max_times =
            args.group_max_times == null
                ? this.box_max_times
                : parseVarInt(args.group_max_times, `${where}.group_max_times`, {
                      min: 0,
                      max: MAX_HIT_TIMES,
                  });
    }

    public verify(
        consumers: {
            character?: ID;
            styles?: ReadonlyArray<ID>;
        },
        where: string,
    ) {
        verifyVarValue(this.box_max_times, consumers, where);
        verifyVarValue(this.box_min_interval, consumers, where);
    }

    public static parseArray(
        raw: ReadonlyArray<HitArgs>,
        where: string,
        opts: {
            files: string;
        },
    ): ReadonlyArray<Hit> {
        const hm = native.loadHitMotionMeta(
            opts.files,
            `${where}.files: file not found (${opts.files})`,
        );
        const hits: Array<Hit> = [];
        for (const gp of hm.groups) {
            const hit = new Hit({ group: gp.group }, `${where}.???`);
            hit.#default = true;
            hits.push(hit);
        }

        for (const r of raw) {
            const idx = hits.findIndex((h) => h.group === r.group);
            if (idx < 0) {
                throw new Error(`${where}[${idx}]: no group (${r.group}) in (${opts.files})`);
            } else if (!hits[idx]!.#default) {
                throw new Error(`${where}[${idx}]: duplicate group (${r.group})`);
            } else {
                hits[idx] = new Hit(r, `${where}[${idx}]`);
            }
        }
        return hits;
    }

    public static verifyArray(
        hits: ReadonlyArray<Hit>,
        consumers: {
            character?: ID;
            styles?: ReadonlyArray<ID>;
        },
        where: string,
    ) {
        for (const hit of hits) {
            hit.verify(consumers, `${where}["${hit.group}"]`);
        }
    }
}

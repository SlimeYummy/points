import {
    checkArray,
    float,
    ID,
    int,
    parseArray,
    parseFloat,
    parseInt,
    parseString,
    parseTime,
} from '../common';
import { calcRootMotionDistances } from '../native';
import { Animation, AnimationArgs } from './animation';
import { Action, ActionArgs } from './base';

export type ActionMoveNpcStopFromArgs =
    | {
          /** 从哪个动画进入Stop */
          anim: string;

          /** 该动画进入Stop时的播放进度 */
          ratio: float | string;
      }
    | [string, float | string];

export type ActionMoveNpcStopArgs = AnimationArgs & {
    /** 进入该动画的来源表 */
    enter_from_table: Array<ActionMoveNpcStopFromArgs>;
};

export class ActionMoveNpcStop {
    /** Stop动画 */
    public readonly anim: Animation;

    /** 进入该动画的来源和进度表 */
    public readonly enter_from_table: ReadonlyArray<{
        /** 从哪个动画进入Stop */
        readonly anim: string;
        /** 该动画进入Stop时的播放进度 */
        readonly ratio: float;
    }>;

    public constructor(
        args: ActionMoveNpcStopArgs,
        using_actions: ReadonlyArray<string>,
        where: string,
    ) {
        this.anim = new Animation(args, where, { root_motion: true });
        this.enter_from_table = this.parseEnterFromTable(
            args.enter_from_table,
            using_actions,
            `${where}.enter_from_table`,
        );
    }

    private parseEnterFromTable(
        table: ReadonlyArray<ActionMoveNpcStopFromArgs>,
        using_actions: ReadonlyArray<string>,
        where: string,
    ) {
        checkArray(table, where, { min_len: 1 });
        return table.map((item, idx) => {
            let anim: string, ratio: float;
            if (Array.isArray(item)) {
                anim = parseString(item[0], `${where}[${idx}][0]`);
                ratio = parseFloat(item[1], `${where}[${idx}][1]`, { min: 0, max: 1, type: 'f32' });
            } else {
                anim = parseString(item.anim, `${where}[${idx}].anim`);
                ratio = parseFloat(item.ratio, `${where}[${idx}].ratio`, {
                    min: 0,
                    max: 1,
                    type: 'f32',
                });
            }
            if (!using_actions.includes(anim)) {
                throw new Error(`${where}[${idx}]: animation not used`);
            }
            return { anim, ratio };
        });
    }
}

export type ActionMoveNpcArgs = ActionArgs & {
    /** 进入按键 */
    enter_key: 'Run' | 'Walk' | 'Dash';

    /** 韧性等级 */
    poise_level?: int;

    /** 前向移动动画 */
    anim_move: AnimationArgs;

    /** 移动速度（m/s） 以anim_move为参考 影响Action内全部动画 */
    move_speed: float;

    /** 移动开始动画 */
    anim_start: AnimationArgs;

    /** 移动停止动画 */
    anim_stops: ReadonlyArray<ActionMoveNpcStopArgs>;

    /** 转身180°所需时间 */
    turn_time: float | string;
};

export class ActionMoveNpc extends Action {
    /** 进入按键 */
    public readonly enter_key: 'Run' | 'Walk' | 'Dash';

    /** 韧性等级 */
    public readonly poise_level: int;

    /** 前向移动动画 */
    public readonly anim_move: Animation;

    /** 移动速度（m/s） 以anim_move为参考 影响Action内全部动画 */
    public readonly move_speed: float;

    /** 移动速度倍率 */
    public readonly speed_ratio: float;

    /** 移动开始动画 */
    public readonly anim_start: Animation;

    /** 移动停止动画 */
    public readonly stops: ReadonlyArray<ActionMoveNpcStop>;

    /** 转身180°所需时间 */
    public readonly turn_time: float;

    /** 触发移动的最小距离（m） */
    public readonly min_distance: float;

    /** 每步移动的距离（m） */
    public readonly step_length: float;

    public constructor(id: ID, args: ActionMoveNpcArgs) {
        super(id, args, { character: 'npc' });
        this.enter_key = parseString(args.enter_key as string, this.w('enter_key'), {
            includes: ['Run', 'Walk', 'Dash'],
        }) as any;
        this.poise_level =
            args.poise_level == null
                ? 0
                : parseInt(args.poise_level, this.w('poise_level'), { min: 0, type: 'u16' });
        this.anim_move = new Animation(args.anim_move, this.w('anim_move'), {
            root_motion: true,
        });
        this.move_speed = parseFloat(args.move_speed, this.w('move_speed'), {
            min: 0,
            max: 1000,
            type: 'f32',
        });
        this.speed_ratio = this.anim_move.calcSpeedRatio(this.move_speed, this.w('anim_move'));
        this.anim_start = new Animation(args.anim_start, this.w('anim_start'), {
            root_motion: true,
        });
        this.stops = parseArray(
            args.anim_stops,
            this.w('anim_stops'),
            (item, idx) =>
                new ActionMoveNpcStop(
                    item,
                    [this.anim_start.files, this.anim_move.files],
                    this.w(`anim_stops[${idx}]`),
                ),
            { min_len: 1 },
        );
        this.turn_time = parseTime(args.turn_time || '12F', this.w('turn_time'), {
            min: 0,
            type: 'f32',
        });
        [this.min_distance, this.step_length] = this.calcMinDistanceAndStepLength();

        Animation.generateLocalID([
            this.anim_start,
            this.anim_move,
            ...this.stops.map((s) => s.anim),
        ]);
    }

    private calcMinDistanceAndStepLength(): [float, float] {
        let start_ratio = 1.0;
        const stop_ratios = [];
        for (const stop of this.stops) {
            for (const from of stop.enter_from_table) {
                if (from.anim === this.anim_start.files) {
                    start_ratio = Math.min(start_ratio, from.ratio);
                } else if (from.anim === this.anim_move.files) {
                    stop_ratios.push(from.ratio);
                }
            }
        }
        if (stop_ratios.length === 0) {
            throw this.e('stops', 'move animation not used in stops');
        }
        stop_ratios.sort();

        const stop_ranges = [];
        for (let i = 0; i < stop_ratios.length - 1; i++) {
            stop_ranges.push({ from: stop_ratios[i]!, to: stop_ratios[i + 1]! });
        }
        stop_ranges.push({ from: stop_ratios[stop_ratios.length - 1]!, to: stop_ratios[0]! });
        if (start_ratio === 1) {
            // No start used in stops
            stop_ranges.push({ from: 0, to: stop_ratios[0]! });
        }
        const stop_dists = calcRootMotionDistances(this.anim_move.files, stop_ranges);
        const start_dist = calcRootMotionDistances(this.anim_start.files, [
            { from: 0, to: start_ratio },
        ]);

        let min_distance = start_dist[0]!;
        if (start_ratio === 1) {
            // No start used in stops, min_distance = start + move(stop)
            min_distance += stop_dists[stop_dists.length - 1]!;
            stop_dists.pop();
        }

        let step_length = Math.max(...stop_dists);
        return [min_distance, step_length];
    }
}

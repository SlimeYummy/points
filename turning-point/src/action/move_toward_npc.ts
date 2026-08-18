import {
    float,
    ID,
    int,
    parseAngleXz,
    parseArray,
    parseFloat,
    parseInt,
    parseString,
    parseTime,
} from '../common';
import { calcRootMotionDistances } from '../native';
import { Animation, AnimationArgs } from './animation';
import { Action, ActionArgs } from './base';
import { ActionMoveNpcStop, ActionMoveNpcStopArgs } from './move_free_npc';

export type ActionMoveTowardNpcDirArgs = {
    /** 进入该动画的移动方向角度（右手系XZ平面） */
    enter_angle: float | string;

    /** 移动动画 */
    anim_move: AnimationArgs;

    /** 开始动画 */
    anim_start: AnimationArgs;

    /** 移动停止动画 */
    anim_stops: ReadonlyArray<ActionMoveNpcStopArgs>;

    /** 移动速度（m/s） 影响Dir内全部动画 */
    move_speed: float;
};

export class ActionMoveTowardNpcDir {
    /** 进入该动画的移动方向角度（右手系XZ平面） */
    public readonly enter_angle: float;

    /** 前向移动动画 */
    public readonly anim_move: Animation;

    /** 移动速度（m/s） 影响Dir内全部动画 */
    public readonly move_speed: float;

    /** 移动速度倍率 */
    public readonly speed_ratio: float;

    /** 移动开始动画 */
    public readonly anim_start: Animation;

    /** 移动停止动画 */
    public readonly anim_stops: ReadonlyArray<ActionMoveNpcStop>;

    /** 触发移动的最小距离（m） */
    public readonly min_distance: float;

    /** 每步移动的距离（m） */
    public readonly step_length: float;

    public constructor(args: ActionMoveTowardNpcDirArgs, where: string) {
        this.enter_angle = parseAngleXz(args.enter_angle, `${where}.enter_angle`);
        this.anim_move = new Animation(args.anim_move, `${where}.anim_move`, {
            root_motion: true,
        });
        this.anim_start = new Animation(args.anim_start, `${where}.anim_start`, {
            root_motion: true,
        });
        this.move_speed = parseFloat(args.move_speed, `${where}.move_speed`, {
            min: 0,
            max: 1000,
            type: 'f32',
        });
        this.speed_ratio = this.anim_move.calcSpeedRatio(this.move_speed, `${where}.anim_move`);
        this.anim_stops = parseArray(
            args.anim_stops,
            `${where}.anim_stops`,
            (item, idx) =>
                new ActionMoveNpcStop(
                    item,
                    [this.anim_start.files, this.anim_move.files],
                    `${where}.anim_stops[${idx}]`,
                ),
            { min_len: 1 },
        );
        [this.min_distance, this.step_length] = this.calcMinDistanceAndStepLength(where);

        Animation.generateLocalID([
            this.anim_start,
            this.anim_move,
            ...this.anim_stops.map((s) => s.anim),
        ]);
    }

    private calcMinDistanceAndStepLength(where: string): [float, float] {
        let start_ratio = 1.0;
        const stop_ratios = [];
        for (const stop of this.anim_stops) {
            for (const from of stop.enter_from_table) {
                if (from.anim === this.anim_start.files) {
                    start_ratio = Math.min(start_ratio, from.ratio);
                } else if (from.anim === this.anim_move.files) {
                    stop_ratios.push(from.ratio);
                }
            }
        }
        if (stop_ratios.length === 0) {
            throw new Error(`${where}.anim_stops: move animation not used in stops`);
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

        const step_length = Math.max(...stop_dists);
        return [min_distance, step_length];
    }
}

export type ActionMoveTowardNpcArgs = ActionArgs & {
    /** 进入按键 */
    enter_key: 'Run' | 'Walk' | 'Dash';

    /** 韧性等级 */
    poise_level?: int;

    /** 各方向移动动画 */
    directions: ReadonlyArray<ActionMoveTowardNpcDirArgs>;

    /** 转身180°所需时间 */
    turn_time: float | string;
};

export class ActionMoveTowardNpc extends Action {
    /** 进入按键 */
    public readonly enter_key: 'Run' | 'Walk' | 'Dash';

    /** 韧性等级 */
    public readonly poise_level: int;

    /** 各方向移动动画 */
    public readonly directions: ReadonlyArray<ActionMoveTowardNpcDir>;

    /** 转身180°所需时间 */
    public readonly turn_time: float;

    public constructor(id: ID, args: ActionMoveTowardNpcArgs) {
        super(id, args, { character: 'npc' });
        this.enter_key = parseString(args.enter_key as string, this.w('enter_key'), {
            includes: ['Run', 'Walk', 'Dash'],
        }) as any;
        this.poise_level =
            args.poise_level == null
                ? 0
                : parseInt(args.poise_level, this.w('poise_level'), { min: 0, type: 'u16' });
        this.directions = parseArray(
            args.directions,
            this.w('directions'),
            (item, idx) => new ActionMoveTowardNpcDir(item, this.w(`directions[${idx}]`)),
            { min_len: 1 },
        );
        this.turn_time = parseTime(args.turn_time || '12F', this.w('turn_time'), {
            min: 0,
            type: 'f32',
        });
    }
}

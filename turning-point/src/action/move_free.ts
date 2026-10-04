import {
    checkArray,
    float,
    ID,
    int,
    LOGIC_SPF,
    parseAngleXzRange,
    parseArray,
    parseBool,
    parseFloat,
    parseIDArray,
    parseString,
    parseTime,
    parseTimeRange,
} from '../common';
import { Character } from '../character';
import * as native from '../native';
import { Animation, AnimationArgs } from './animation';
import { Action, ActionArgs, LEVEL_MOVE, parseActionLevel } from './base';

export type ActionMoveFreeStartArgs = AnimationArgs & {
    /** 进入该动画的移动朝向角度（右手系XZ平面） */
    enter_angle: [float | string, float | string];

    /** 移动开始时原地转身的结束时间 */
    turn_in_place_end?: float | string;

    /** 可以触发快速停止的结束时间 */
    quick_stop_end?: float | string;
};

export class ActionMoveFreeStart {
    /** Start动画 */
    public readonly anim: Animation;

    /** 进入该动画的移动朝向角度（右手系XZ平面） */
    public readonly enter_angle: readonly [float, float];

    /** 移动开始时原地转身的结束时间 */
    public readonly turn_in_place_end: float;

    /** 可以触发快速停止的结束时间 */
    public readonly quick_stop_end: float;

    public constructor(args: ActionMoveFreeStartArgs, where: string) {
        this.anim = new Animation(args, where, { root_motion: true });
        this.enter_angle = parseAngleXzRange(args.enter_angle, `${where}.enter_angle`);
        this.turn_in_place_end =
            args.turn_in_place_end == null
                ? 0
                : parseTime(args.turn_in_place_end, `${where}.turn_in_place_end`, {
                      min: LOGIC_SPF,
                      type: 'f32',
                  });
        this.quick_stop_end =
            args.quick_stop_end == null
                ? this.anim.duration / 2
                : parseTime(args.quick_stop_end, `${where}.quick_stop_end`, {
                      min: 0,
                      type: 'f32',
                  });
    }
}

export type ActionMoveFreeTurnArgs = AnimationArgs & {
    /** 进入该动画的转向角度（右手系XZ平面） */
    enter_angle: [float | string, float | string];

    /** 转身开始时原地转身的结束时间 */
    turn_in_place_end: float | string;
};

export class ActionMoveFreeTurn {
    /** Turn动画 */
    public readonly anim: Animation;

    /** 进入该动画的转向角度（右手系XZ平面） */
    public readonly enter_angle: readonly [float, float];

    public constructor(args: ActionMoveFreeTurnArgs, where: string) {
        this.anim = new Animation(args, where, { root_motion: true });
        this.enter_angle = parseAngleXzRange(args.enter_angle, `${where}.enter_angle`);
    }
}

/** 进入该动画的相位 [开始, 结束] */
export type ActionMoveFreeStopEnterArgs = [float | string, float | string];

export type ActionMoveFreeStopLeaveArgs =
    | {
          /** 动画时间 */
          time: float | string;
          /** 对应相位 */
          phase: float | string;
      }
    | [float | string, float | string];

export type ActionMoveFreeStop1Args = AnimationArgs & {
    /** 停止动画淡入时间 */
    fade_in?: float | string;

    /** 进入该动画的相位表  */
    enter_phase_table: Array<ActionMoveFreeStopEnterArgs>;

    /** 离开该动画的相位表  */
    leave_phase_table?: Array<ActionMoveFreeStopLeaveArgs>;
};

function parseStopEnterPhaseTable(
    table: ReadonlyArray<ActionMoveFreeStopEnterArgs>,
    duration: float,
    where: string,
): ReadonlyArray<readonly [float, float]> {
    checkArray(table, `${where}.enter_phase_table`, { min_len: 1 });
    return table.map((item, idx) => {
        checkArray(item, `${where}[${idx}]`, { len: 2 });
        return [
            parseFloat(item[0], `${where}[${idx}][0]`, { min: 0, max: 1, type: 'f32' }),
            parseFloat(item[1], `${where}[${idx}][1]`, { min: 0, max: 1, type: 'f32' }),
        ] as const;
    });
}

function parseStopLeavePhaseTable(
    table: undefined | Array<ActionMoveFreeStopLeaveArgs>,
    duration: float,
    where: string,
): ReadonlyArray<{ readonly time: float; readonly phase: float }> {
    if (table == null) {
        return [];
    }

    checkArray(table, `${where}.leave_phase_table`, { min_len: 2 });
    let iter_time = 0;
    return table.map((item, idx) => {
        let time: float, phase: float, time_where;
        if (Array.isArray(item)) {
            time = parseTime(item[0], `${where}[${idx}][0]`, {
                min: 0,
                max: duration,
                type: 'f32',
            });
            phase = parseFloat(item[1], `${where}[${idx}][1]`, { min: 0, max: 1, type: 'f32' });
            time_where = `${where}[${idx}][0]`;
        } else {
            time = parseTime(item.time, `${where}[${idx}].time`, {
                min: 0,
                max: duration,
                type: 'f32',
            });
            phase = parseFloat(item.phase, `${where}[${idx}].phase`, {
                min: 0,
                max: 1,
                type: 'f32',
            });
            time_where = `${where}[${idx}].time`;
        }
        if (idx === 0 && time !== 0) {
            throw new Error(`${time_where} must == 0`);
        }
        if (time < iter_time) {
            throw new Error(`${time_where} must be ascend`);
        }
        iter_time = time;
        return { time, phase };
    });
}

export class ActionMoveFreeStop1 {
    public readonly T: string;

    /** Stop动画 */
    public readonly anim: Animation;

    /** 停止动画淡入时间 */
    public readonly fade_in: float;

    /** 进入该动画的相位表 [开始, 结束] */
    public readonly enter_phase_table: ReadonlyArray<readonly [float, float]>;

    /** 停止动画减速阶段的结束时间 */
    public readonly leave_phase_table: ReadonlyArray<{
        /** 动画时间 */
        readonly time: float;
        /** 对应相位 */
        readonly phase: float;
    }>;

    public constructor(args: ActionMoveFreeStop1Args, where: string) {
        this.T = 'Stop1';
        this.anim = new Animation(args, where, { root_motion: true });
        this.fade_in =
            args.fade_in == null
                ? 0
                : parseTime(args.fade_in, `${where}.fade_in`, {
                      min: 0,
                      max: this.anim.duration,
                      type: 'f32',
                  });
        this.enter_phase_table = parseStopEnterPhaseTable(
            args.enter_phase_table,
            this.anim.duration,
            `${where}.enter_phase_table`,
        );
        this.leave_phase_table = parseStopLeavePhaseTable(
            args.leave_phase_table,
            this.anim.duration,
            `${where}.leave_phase_table`,
        );
    }
}

/**
 * 特殊Stop动作模式 重平滑了手臂动作
 *
 * 我们把身体拆分为两部分 分别处理
 * 对于身体部分 简单插值过度
 *
 * 对于手臂部分 我们维持前动画继续播放一段时间（但速度逐渐减慢） 以保持动画平滑
 * 再逐步削减前动画权重 直到stop结束 完全切换至stop动画的最终姿态
 *
 * 另外手臂动画支持以additive模式 附加一个额外动画偏移
 */
export type ActionMoveFreeStop2Args = {
    /**
     * 前动画继续播放的时间 [动画时间, 实际播放时间]
     * 因为播放速度会逐渐减慢 所以要求：动画时间<实际播放时间
     */
    prev_anim_time: readonly [float | string, float | string];

    /** 停止动画 */
    anim_stop: AnimationArgs;

    /** 不含手臂部分的淡入时间 */
    no_arm_fade_in: float | string;

    /** 手臂部分的淡出时间 */
    arm_fade_out: float | string;

    /** 手臂附加增加动画 */
    anim_arm_additive?: AnimationArgs;

    /** 手臂附加增加动画的淡入/淡出时间 [淡入结束时间，淡出开始时间] */
    arm_additive_fade_inout?: readonly [float | string, float | string];

    /** 进入该动画的相位表  */
    enter_phase_table: Array<ActionMoveFreeStopEnterArgs>;

    /** 离开该动画的相位表  */
    leave_phase_table?: Array<ActionMoveFreeStopLeaveArgs>;
};

export class ActionMoveFreeStop2 {
    public readonly T: string;

    /** 前动画继续播放的时间 [动画时间, 实际播放时间] */
    public readonly prev_anim_time: readonly [float, float];

    /** Stop动画 */
    public readonly anim_stop: Animation;

    /** 不含手臂部分的淡入时间 */
    public readonly no_arm_fade_in: float;

    /** 手臂部分的淡出时间 */
    public readonly arm_fade_out: float;

    /** 手臂附加增加动画 */
    public readonly anim_arm_additive?: Animation;

    /** 手臂附加增加动画的淡入/淡出时间 [淡入结束时间，淡出开始时间] */
    public readonly arm_additive_fade_inout?: readonly [float, float];

    /** 进入该动画的相位表 [开始, 结束] */
    public readonly enter_phase_table: ReadonlyArray<readonly [float, float]>;

    /** 离开该动画的相位表 */
    public readonly leave_phase_table: ReadonlyArray<{
        /** 动画时间 */
        readonly time: float;
        /** 对应相位 */
        readonly phase: float;
    }>;

    public constructor(args: ActionMoveFreeStop2Args, where: string) {
        this.T = 'Stop2';

        this.prev_anim_time = parseTimeRange(args.prev_anim_time, `${where}.prev_anim_time`, {
            min: 0,
            type: 'f32',
        });

        this.anim_stop = new Animation(args.anim_stop, `${where}.anim_stop`, { root_motion: true });
        this.no_arm_fade_in = parseTime(args.no_arm_fade_in, `${where}.no_arm_fade_in`, {
            min: 0,
            max: this.anim_stop.duration,
            type: 'f32',
        });
        this.arm_fade_out = parseTime(args.arm_fade_out, `${where}.arm_fade_out`, {
            min: 0,
            max: this.anim_stop.duration,
            type: 'f32',
        });

        this.anim_arm_additive =
            args.anim_arm_additive == null
                ? undefined
                : new Animation(args.anim_arm_additive, `${where}.anim_arm_additive`, {
                      root_motion: false,
                      weapon_control: false,
                      hit_motion: false,
                      additive_blending: true,
                  });
        if (this.anim_arm_additive != null) {
            this.arm_additive_fade_inout = parseTimeRange(
                args.arm_additive_fade_inout as any,
                `${where}.arm_additive_fade_inout`,
                {
                    min: 0,
                    max: this.anim_stop.duration,
                    type: 'f32',
                },
            );
        }

        this.enter_phase_table = parseStopEnterPhaseTable(
            args.enter_phase_table,
            this.anim_stop.duration,
            `${where}.enter_phase_table`,
        );
        this.leave_phase_table = parseStopLeavePhaseTable(
            args.leave_phase_table,
            this.anim_stop.duration,
            `${where}.leave_phase_table`,
        );
    }
}

export type ActionMoveFreeArgs = ActionArgs & {
    /** 进入按键 */
    enter_key: 'Run' | 'Walk' | 'Dash';

    /** 进入等级 */
    enter_level?: int;

    /** 通常状态维持等级 */
    keep_level?: int;

    /**
     * 特殊状态维持等级 包括：
     * - Start [0, turn_in_place_end] 的转身阶段
     * - Stop [0, speed_down_end] 的停止减速阶段
     * - Turn 全部阶段
     **/
    keep_level_special?: int;

    /** 前向移动动画 */
    anim_move: AnimationArgs;

    /** 移动速度（m/s） 影响Action内全部动画 */
    move_speed: float;

    /** 移动开始动画 */
    anim_starts: ReadonlyArray<ActionMoveFreeStartArgs>;

    /** 移动停止动画 */
    anim_stops: ReadonlyArray<ActionMoveFreeStop1Args | ActionMoveFreeStop2Args>;

    /** 快速停止时间 */
    quick_stop_time?: float | string;

    /** 转身动画 */
    anim_turns?: ReadonlyArray<ActionMoveFreeTurnArgs>;

    /** 转身180°所需时间 仅在未匹配到turns时生效 */
    turn_time: float | string;

    /** 是否继承上个动作派生 */
    derive_keeping?: boolean | int;

    /**
     * 平滑切换移动动作列表
     * 从下列移动动作进入时 不会从Start开始 而是参考前一个动作的状态：
     * - 前移动状态为Move 进入当前Move状态
     * - 前移动状态为Start 且不在[0, turn_in_place_end] 进入当前Start状态
     */
    smooth_move_froms?: ReadonlyArray<ID>;

    /** 平滑切换移动持续时间 */
    smooth_move_duration?: float | string;
};

export class ActionMoveFree extends Action {
    /** 进入按键 */
    public readonly enter_key: 'Run' | 'Walk' | 'Dash';

    /** 进入等级 */
    public readonly enter_level: int;

    /** 通常状态维持等级 */
    public readonly keep_level: int;

    /**
     * 特殊状态维持等级 包括：
     * - Start [0, turn_in_place_end] 的转身阶段
     * - Stop [0, speed_down_end] 的停止减速阶段
     * - Turn [0, turn_in_place_end] 的转身阶段
     **/
    public readonly keep_level_special: int;

    /** 韧性等级 */
    public readonly poise_level: int;

    /** 前向移动动画 */
    public readonly anim_move: Animation;

    /** 移动速度（m/s） 影响Action内全部动画 */
    public readonly move_speed: float;

    /** 速度倍率 */
    public readonly speed_ratio: float;

    /** 移动开始动画 */
    public readonly starts: ReadonlyArray<ActionMoveFreeStart>;

    /** 移动停止动画 */
    public readonly stops: ReadonlyArray<ActionMoveFreeStop1 | ActionMoveFreeStop2>;

    /** 快速停止时间 */
    public readonly quick_stop_time: float;

    /** 转身动画 */
    public readonly turns: ReadonlyArray<ActionMoveFreeTurn>;

    /** 转身180°所需时间 仅在未匹配到turns时生效 */
    public readonly turn_time: float;

    /** 是否继承上个动作派生 */
    public readonly derive_keeping: boolean;

    /**
     * 平滑切换移动动作列表
     * 从下列移动动作进入时 不会从Start开始 而是参考前一个动作的状态：
     * - 前移动状态为Move 进入当前Move状态
     * - 前移动状态为Start 且不在[0, turn_in_place_end] 进入当前Start状态
     */
    public readonly smooth_move_froms: ReadonlyArray<ID>;

    /** 平滑切换移动持续时间 */
    public readonly smooth_move_duration: float;

    public constructor(id: ID, args: ActionMoveFreeArgs) {
        super(id, args, { character: 'player' });
        this.enter_key = parseString(args.enter_key as string, this.w('enter_key'), {
            includes: ['Run', 'Walk', 'Dash'],
        }) as any;
        this.enter_level = parseActionLevel(args.enter_level || LEVEL_MOVE, this.w('enter_level'));
        this.keep_level = parseActionLevel(
            args.keep_level || LEVEL_MOVE - 10,
            this.w('keep_level'),
        );
        this.keep_level_special = parseActionLevel(
            args.keep_level_special || LEVEL_MOVE + 10,
            this.w('keep_level_special'),
        );
        this.poise_level = 0;

        this.anim_move = new Animation(args.anim_move, this.w('anim_move'), {
            root_motion: true,
        });
        this.move_speed = parseFloat(args.move_speed, this.w('move_speed'), {
            min: 0,
            max: 1000,
            type: 'f32',
        });
        this.speed_ratio = this.anim_move.calcSpeedRatio(this.move_speed, this.w('anim_move'));

        this.starts = parseArray(
            args.anim_starts,
            this.w('anim_starts'),
            (item, idx) => new ActionMoveFreeStart(item, this.w(`anim_starts[${idx}]`)),
            { min_len: 1 },
        );

        this.stops = this.parseStops(args.anim_stops);
        this.quick_stop_time = parseTime(args.quick_stop_time || 0, this.w('quick_stop_time'), {
            min: 0,
            type: 'f32',
        });

        this.turns =
            args.anim_turns == null
                ? []
                : parseArray(args.anim_turns, this.w('anim_turns'), (item, idx) => {
                      return new ActionMoveFreeTurn(item, this.w(`anim_turns[${idx}]`));
                  });
        this.turn_time = parseTime(args.turn_time || '12F', this.w('turn_time'), {
            min: 0,
            type: 'f32',
        });

        this.derive_keeping =
            args.derive_keeping == null
                ? true
                : parseBool(args.derive_keeping, this.w('derive_keeping'));
        this.smooth_move_froms = parseIDArray(
            args.smooth_move_froms || [],
            'Action',
            this.w('smooth_move_froms'),
        );
        this.smooth_move_duration = parseTime(
            args.smooth_move_duration || '10F',
            this.w('smooth_move_duration'),
            { min: 0, type: 'f32' },
        );

        Animation.generateLocalID([
            this.anim_move,
            ...this.starts.map((s) => s.anim),
            ...this.turns.map((s) => s.anim),
            ...this.stops.flatMap((s) =>
                s instanceof ActionMoveFreeStop2 ? [s.anim_stop, s.anim_arm_additive] : [s.anim],
            ),
        ]);
    }

    private parseStops(
        anim_stops: ActionMoveFreeArgs['anim_stops'],
    ): ReadonlyArray<ActionMoveFreeStop1 | ActionMoveFreeStop2> {
        const where = this.w('anim_stops');
        if (!Array.isArray(anim_stops) || anim_stops.length < 1) {
            throw new Error(`${where}: length must >= 1`);
        }
        const stops: Array<ActionMoveFreeStop1 | ActionMoveFreeStop2> = [];
        for (const [idx, item] of anim_stops.entries()) {
            const item_where = `${where}[${idx}]`;
            if ('anim_stop' in item) {
                stops.push(new ActionMoveFreeStop2(item, item_where));
            } else {
                stops.push(new ActionMoveFreeStop1(item, item_where));
            }
        }
        return stops;
    }

    public override verify() {
        super.verify();
        for (const [idx, id] of this.smooth_move_froms.entries()) {
            const act = Action.find(id, this.w(`smooth_move_froms[${idx}]`));
            if (!(act instanceof ActionMoveFree)) {
                throw this.e(`smooth_move_froms[${idx}]`, 'must not be ActionMoveFree');
            }
        }

        if (this.stops.some((s) => s instanceof ActionMoveFreeStop2)) {
            const character = Character.find(this.character!, this.w('character'));
            if (!character.joint_weights_table) {
                throw this.e('character', `joint_weights_table not enabled`);
            }
            const where = this.w('character');
            native.checkJointWeightsTableName(
                character.skeleton_files,
                'Arm',
                `${where}: "Arm" not found in joint weights table`,
            );
            native.checkJointWeightsTableName(
                character.skeleton_files,
                'NoArm',
                `${where}: "NoArm" not found in joint weights table`,
            );
        }
    }
}

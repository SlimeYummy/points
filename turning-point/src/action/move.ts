import {
    checkArray,
    float,
    ID,
    int,
    LOGIC_SPF,
    parseAngleXZRange,
    parseBool,
    parseFloat,
    parseString,
    parseTime,
} from '../common';
import { Aniamtion, AniamtionArgs } from './animation';
import { Action, ActionArgs, LEVEL_MOVE } from './base';

export type ActionMoveStartArgs = AniamtionArgs & {
    /** 进入该动画的角度（右手系XZ平面） */
    enter_angle: [float | string, float | string];

    /** 移动开始时原地转身的结束时间 */
    turn_in_place_end?: float | string;

    /** 可以触发快速停止的结束时间 */
    quick_stop_end?: float | string;
};

export class ActionMoveStart extends Aniamtion {
    /** 进入该动画的角度（右手系XZ平面） */
    public enter_angle: readonly [float, float];

    /** 移动开始时原地转身的结束时间 */
    public turn_in_place_end: float;

    /** 可以触发快速停止的结束时间 */
    public quick_stop_end: float;

    public constructor(
        args: ActionMoveStartArgs,
        where: string,
        opts?: {
            root_motion?: boolean;
        },
    ) {
        super(args, where, opts);
        this.enter_angle = parseAngleXZRange(args.enter_angle, `${where}.enter_angle`);
        this.turn_in_place_end = parseTime(
            args.turn_in_place_end || 0,
            `${where}.turn_in_place_end`,
            {
                min: LOGIC_SPF,
            },
        );
        this.quick_stop_end = parseTime(
            args.quick_stop_end || this.duration / 2,
            `${where}.quick_stop_end`,
            { min: 0 },
        );
    }
}

export type ActionMoveTurnArgs = AniamtionArgs & {
    /** 进入该动画的角度（右手系XZ平面） */
    enter_angle: [float | string, float | string];

    /** 转身开始时原地转身的结束时间 */
    turn_in_place_end: float | string;
};

export class ActionMoveTurn extends Aniamtion {
    /** 进入该动画的角度（右手系XZ平面） */
    public enter_angle: readonly [float, float];

    /** 转身开始时原地转身的结束时间 */
    public turn_in_place_end: float;

    public constructor(
        args: ActionMoveTurnArgs,
        where: string,
        opts?: {
            root_motion?: boolean;
        },
    ) {
        super(args, where, opts);
        this.enter_angle = parseAngleXZRange(args.enter_angle, `${where}.enter_angle`);
        this.turn_in_place_end = parseTime(args.turn_in_place_end, `${where}.turn_in_place_end`, {
            min: 0,
        });
    }
}

export type ActionMoveStopArgs = AniamtionArgs & {
    /** 进入该动画的相位表 [开始相位, 结束相位, 动画偏移时间] */
    enter_phase_table: Array<[float | string, float | string, float | string]>;

    /** 停止动画减速阶段的结束时间 */
    speed_down_end: float | string;
};

export class ActionMoveStop extends Aniamtion {
    /** 进入该动画的相位表 [开始相位, 结束相位, 动画偏移时间] */
    public enter_phase_table: ReadonlyArray<readonly [float, float, float]>;

    /** 停止动画减速阶段的结束时间 */
    public speed_down_end: float;

    public constructor(
        args: ActionMoveStopArgs,
        where: string,
        opts?: {
            root_motion?: boolean;
        },
    ) {
        super(args, where, opts);
        this.enter_phase_table = this.parseEnterPhaseTable(
            args.enter_phase_table,
            this.duration,
            `${where}.enter_phase_table`,
        );
        this.speed_down_end = parseTime(args.speed_down_end, `${where}.speed_down_end`, { min: 0 });
    }

    private parseEnterPhaseTable(
        table: Array<[float | string, float | string, float | string]>,
        duration: float,
        where: string,
    ): ReadonlyArray<readonly [float, float, float]> {
        checkArray(table, `${where}.enter_phase_table`, { min_len: 1 });
        return table.map((item, idx) => {
            checkArray(item, `${where}[${idx}]`, { len: 3 });
            return [
                parseFloat(item[0], `${where}[${idx}][0]`, { min: 0, max: 1 }),
                parseFloat(item[1], `${where}[${idx}][1]`, { min: 0, max: 1 }),
                parseTime(item[2], `${where}[${idx}][2]`, { min: 0, max: duration }),
            ];
        });
    }
}

export type ActionMoveArgs = ActionArgs & {
    /** 进入按键 */
    enter_key: 'Run' | 'Walk' | 'Dash';

    /** 前向移动动画 */
    anim_move: AniamtionArgs;

    /** 移动速度（m/s） 以anim_move为参考 影响Action内全部动画 */
    move_speed: float;

    /** 移动开始动画 */
    starts?: Array<ActionMoveStartArgs>;

    /** 移动开始时间 仅在未匹配到starts时生效 */
    start_time?: float | string;

    /** 转身动画 */
    turns?: Array<ActionMoveTurnArgs>;

    /** 转身180°所需时间 仅在未匹配到turns时生效 */
    turn_time: float | string;

    /** 移动停止动画 */
    stops?: Array<ActionMoveStopArgs>;

    /** 移动停止时间 仅在未匹配到stops时生效 */
    stop_time?: float | string;

    /** 快速停止时间 */
    quick_stop_time?: float | string;

    /** 是否继承上个动作派生 */
    derive_keeping?: boolean | int;
};

export class ActionMove extends Action {
    /** 前向移动动画 */
    public readonly anim_move: Aniamtion;

    /** 移动速度（m/s） 以anim_move为参考 影响Action内全部动画 */
    public readonly move_speed: float;

    /** 移动开始动画 */
    public readonly starts: ReadonlyArray<ActionMoveStart>;

    /** 移动开始时间 仅在未匹配到starts时生效 */
    public readonly start_time: float;

    /** 转身动画 */
    public readonly turns: ReadonlyArray<ActionMoveTurn>;

    /** 转身180°所需时间 仅在未匹配到turns时生效 */
    public readonly turn_time: float;

    /** 移动停止动画 */
    public readonly stops: ReadonlyArray<ActionMoveStop>;

    /** 移动停止时间 仅在未匹配到stops时生效 */
    public readonly stop_time: float;

    /** 快速停止时间 */
    public readonly quick_stop_time: float;
    
    /** 进入按键 */
    public readonly enter_key?: 'Run' | 'Walk' | 'Dash';

    /** 进入等级 */
    public readonly enter_level: int;

    /** 派生等级 */
    public readonly derive_level: int;

    /** 是否继承上个动作派生 */
    public readonly derive_keeping: boolean;

    /** 韧性等级 */
    public readonly poise_level: int;

    public constructor(id: ID, args: ActionMoveArgs) {
        super(id, args);
        this.anim_move = new Aniamtion(args.anim_move, this.w('anim_move'), { root_motion: true });
        this.move_speed = parseFloat(args.move_speed, this.w('move_speed'), { min: 0, max: 1000 });
        this.starts = (args.starts || []).map(
            (args, idx) =>
                new ActionMoveStart(args, this.w(`starts[${idx}]`), { root_motion: true }),
        );
        this.start_time = parseTime(args.start_time || '4F', this.w('start_time'), { min: 0 });
        this.turns = (args.turns || []).map(
            (args, idx) => new ActionMoveTurn(args, this.w(`turns[${idx}]`), { root_motion: true }),
        );
        this.turn_time = parseTime(args.turn_time || '8F', this.w('turn_time'), { min: 0 });
        this.stops = (args.stops || []).map(
            (args, idx) => new ActionMoveStop(args, this.w(`stops[${idx}]`), { root_motion: true }),
        );
        this.stop_time = parseTime(args.stop_time || '4F', this.w('stop_time'), { min: 0 });
        this.quick_stop_time = parseTime(args.quick_stop_time || 0, this.w('quick_stop_time'), {
            min: 0,
        });
        this.enter_key = parseString(args.enter_key as string, this.w('enter_key'), { includes: ['Run', 'Walk', 'Dash'] }) as any;
        this.enter_level = LEVEL_MOVE;
        this.derive_level = LEVEL_MOVE;
        this.derive_keeping =
            args.derive_keeping == null
                ? true
                : parseBool(args.derive_keeping, this.w('derive_keeping'));
        this.poise_level = 0;

        Aniamtion.generateLocalID([this.anim_move, ...this.starts, ...this.turns, ...this.stops]);
    }
}

import { float, ID, int, parseBool, parseTime } from '../common';
import { Aniamtion, AniamtionArgs } from './animation';
import { Action, ActionArgs, LEVEL_MOVE } from './base';

export type ActionMoveArgs = ActionArgs & {
    /** 前向移动动画 */
    anim_move: AniamtionArgs;

    /** 左侧转身动画 */
    anim_turn_left?: AniamtionArgs;

    /** 右侧转身动画 */
    anim_turn_right?: AniamtionArgs;

    /** 移动停止动画 */
    anim_stop?: AniamtionArgs;

    /** 转向（90°）所需时间 */
    yam_time: float | string;

    /** 转身所需时间 */
    turn_time: float | string;

    /** 是否继承上个动作派生 */
    derive_keeping?: boolean | int;
};

export class ActionMove extends Action {
    /** 前向移动动画 */
    public readonly anim_move: Aniamtion;

    /** 左侧转身动画 */
    public readonly anim_turn_left?: Aniamtion;

    /** 右侧转身动画 */
    public readonly anim_turn_right?: Aniamtion;

    /** 移动停止动画 */
    public readonly anim_stop?: Aniamtion;

    /** 转向（90°）所需时间 */
    public readonly yam_time: float;

    /** 转身所需时间 */
    public readonly turn_time: float;

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
        this.anim_move = new Aniamtion(args.anim_move, this.w('anim_move'), { root_motion: false });
        this.anim_turn_left = !args.anim_turn_left
            ? undefined
            : new Aniamtion(args.anim_turn_left, this.w('anim_turn_left'), { root_motion: false });
        this.anim_turn_right = !args.anim_turn_right
            ? undefined
            : new Aniamtion(args.anim_turn_right, this.w('anim_turn_right'), {
                  root_motion: false,
              });
        this.anim_stop = !args.anim_stop
            ? undefined
            : new Aniamtion(args.anim_stop, this.w('anim_stop'), { root_motion: false });
        this.yam_time = parseTime(args.yam_time, this.w('yam_time'));
        this.turn_time = parseTime(args.turn_time, this.w('turn_time'));
        this.enter_level = LEVEL_MOVE;
        this.derive_level = LEVEL_MOVE;
        this.derive_keeping =
            args.derive_keeping == null
                ? true
                : parseBool(args.derive_keeping, this.w('derive_keeping'));
        this.poise_level = 0;
    }
}

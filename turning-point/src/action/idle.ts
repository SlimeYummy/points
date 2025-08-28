import { float, ID, int, parseBool, parseTime } from '../common';
import { Aniamtion, AniamtionArgs } from './animation';
import { Action, ActionArgs, LEVEL_IDLE } from './base';

export type ActionIdleArgs = ActionArgs & {
    /** 非战斗状态站立动画 */
    anim_idle: AniamtionArgs;

    /** 战斗状态站立动画 */
    anim_ready?: AniamtionArgs;

    /** 非战斗状态 随机小动作动画 */
    anim_randoms?: ReadonlyArray<AniamtionArgs>;

    /** 从Ready进入Idle的延迟时间 */
    auto_idle_delay?: float | string;

    /** 是否继承上个动作派生 */
    derive_keeping?: boolean | int;
};

export class ActionIdle extends Action {
    /** 非战斗状态站立动画 */
    public readonly anim_idle: Aniamtion;

    /** 战斗状态站立动画 */
    public readonly anim_ready?: Aniamtion;

    /** 非战斗状态 随机小动作动画 */
    public readonly anim_randoms?: ReadonlyArray<Aniamtion>;

    /** 从Ready进入Idle的延迟时间 */
    public readonly auto_idle_delay: float;

    /** 进入等级 */
    public readonly enter_level: int;

    /** 派生等级 */
    public readonly derive_level: int;

    /** 是否继承上个动作派生 */
    public readonly derive_keeping: boolean;

    /** 韧性等级 */
    public readonly poise_level: int;

    public constructor(id: ID, args: ActionIdleArgs) {
        super(id, args);
        this.anim_idle = new Aniamtion(args.anim_idle, this.w('anim_idle'), { root_motion: false });
        this.anim_ready = !args.anim_ready
            ? undefined
            : new Aniamtion(args.anim_ready, this.w('anim_ready'), {
                  root_motion: false,
              });
        this.anim_randoms = !args.anim_randoms
            ? undefined
            : args.anim_randoms.map(
                  (args) => new Aniamtion(args, this.w('anim_randoms'), { root_motion: false }),
              );
        this.auto_idle_delay = parseTime(args.auto_idle_delay || '10s', this.w('auto_idle_delay'));
        this.enter_level = LEVEL_IDLE;
        this.derive_level = LEVEL_IDLE;
        this.derive_keeping =
            args.derive_keeping == null
                ? true
                : parseBool(args.derive_keeping, this.w('derive_keeping'));
        this.poise_level = 0;

        Aniamtion.generateLocalID([this.anim_idle, this.anim_ready, ...(this.anim_randoms || [])]);
    }
}

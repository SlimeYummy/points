import { float, ID, parseBool, parseID, parseTimeRange } from '../common';
import { Action, ActionIdle } from '../action';
import { AiIntention, AiTask, AiTaskArgs, parseAiIntention } from './task_base';

export type AiTaskIdleArgs = AiTaskArgs & {
    /** AI意图 */
    intention?: AiIntention;

    /** AI意图（动作完成后） */
    next_intention?: AiIntention;

    /** 待机动作ID */
    action_idle: ID;

    /** 待机持续时间（秒）（随机范围） */
    duration?: string | ReadonlyArray<float | string>;

    /** 在目标改变时退出动作 */
    target_exit?: boolean;
};

/**
 * AI任务（待机）
 */
export class AiTaskIdle extends AiTask {
    /** AI意图 */
    public readonly intention: AiIntention;

    /** AI意图（动作完成后） */
    public readonly next_intention: AiIntention;

    /** 待机动作ID */
    public readonly action_idle: ID;

    /** 待机持续时间（秒）（随机范围） */
    public readonly duration?: readonly [float, float];

    /** 在目标改变时退出动作 */
    public readonly target_exit: boolean;

    public constructor(id: ID, args: AiTaskIdleArgs) {
        super(id, args);
        this.intention = parseAiIntention(args.intention ?? 'Idle', this.w('intention'));
        this.next_intention = parseAiIntention(
            args.next_intention ?? 'Idle',
            this.w('next_intention'),
        );
        this.action_idle = parseID(args.action_idle, 'Action', this.w('action_idle'));
        this.duration =
            args.duration == null
                ? undefined
                : parseTimeRange(args.duration, this.w('duration'), { min: 0, type: 'f32' });
        this.target_exit = parseBool(args.target_exit ?? false, this.w('target_exit'));
    }

    public override verify() {
        super.verify();

        const idle = Action.find(this.action_idle, this.w('action_idle'));
        if (!(idle instanceof ActionIdle)) {
            throw this.e('action_idle', 'must be an ActionIdle');
        }
        if (!idle.character_npcs?.includes(this.character_npc)) {
            throw this.e('action_idle', 'AiTaskIdle and ActionIdle mismatch');
        }
    }
}

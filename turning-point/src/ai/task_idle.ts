import { float, ID, int, parseID, parseInt, parseTimeRange } from '../common';
import { Action, ActionIdle } from '../action';
import { AiTask, AiTaskArgs } from './task_base';

export type AiTaskIdleArgs = AiTaskArgs & {
    /** 最大重复次数 */
    max_repeat?: int;

    /** 待机动作ID */
    action_idle: ID;

    /** 持续时间范围 */
    duration: string | readonly [float | string, float | string];
};

/**
 * AI任务（待机）
 */
export class AiTaskIdle extends AiTask {
    /** 最大重复次数 */
    public readonly max_repeat: int;

    /** 待机动作ID */
    public readonly action_idle: ID;

    /** 持续时间范围 */
    public readonly duration: readonly [float, float];

    public constructor(id: ID, args: AiTaskIdleArgs) {
        super(id, args);
        this.max_repeat =
            args.max_repeat == null
                ? 1
                : parseInt(args.max_repeat, this.w('max_repeat'), { min: 0 });
        this.action_idle = parseID(args.action_idle, 'Action', this.w('action_idle'));
        this.duration = parseTimeRange(args.duration, this.w('duration'), { min: 0 });
    }

    public override verify() {
        super.verify();

        const idle = Action.find(this.action_idle, this.w('action_idle'));
        if (!(idle instanceof ActionIdle)) {
            throw this.error('action_idle', 'must be an ActionIdle');
        }
        if (!idle.character_npcs?.includes(this.character_npc)) {
            throw this.error('action_idle', 'AiTaskIdle and ActionIdle mismatch');
        }
    }
}

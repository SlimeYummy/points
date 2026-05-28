import { ID, int, parseID, parseInt } from '../common';
import { Action, ActionIdle, LEVEL_IDLE } from '../action';
import { AiTask, AiTaskArgs } from './task_base';

export type AiTaskIdleArgs = AiTaskArgs & {
    /** 进入等级 */
    enter_level?: int;

    /** 维持等级 */
    keep_level?: int;

    /** 待机动作ID */
    action_idle: ID;
};

/**
 * AI任务（待机）
 */
export class AiTaskIdle extends AiTask {
    /** 进入等级 */
    public readonly enter_level: int;

    /** 维持等级 */
    public readonly keep_level: int;

    /** 待机动作ID */
    public readonly action_idle: ID;

    public constructor(id: ID, args: AiTaskIdleArgs) {
        super(id, args);
        this.enter_level = parseInt(args.enter_level ?? LEVEL_IDLE, this.w('enter_level'), {
            type: 'u16',
        });
        this.keep_level = parseInt(args.keep_level ?? LEVEL_IDLE, this.w('keep_level'), {
            type: 'u16',
        });
        this.action_idle = parseID(args.action_idle, 'Action', this.w('action_idle'));
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

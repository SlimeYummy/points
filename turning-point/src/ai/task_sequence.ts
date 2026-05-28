import { ID, int, parseArray, parseID, parseInt } from '../common';
import { LEVEL_ATTACK } from '../action';
import { AiTask, AiTaskArgs } from './task_base';

export type AiTaskSequenceArgs = AiTaskArgs & {
    /** 进入等级 */
    enter_level?: int;

    /** 子任务列表（AiTask ID）（子任务不能是另一个 AiTaskSequence） */
    tasks: ReadonlyArray<ID>;
};

/**
 * AI任务（序列）
 */
export class AiTaskSequence extends AiTask {
    /** 进入等级 */
    public readonly enter_level: int;

    /** 子任务列表（AiTask ID）（子任务不能是另一个 AiTaskSequence） */
    public readonly tasks: ReadonlyArray<ID>;

    public constructor(id: ID, args: AiTaskSequenceArgs) {
        super(id, args);
        this.enter_level = parseInt(args.enter_level ?? LEVEL_ATTACK, this.w('enter_level'), {
            type: 'u16',
        });
        this.tasks = parseArray(
            args.tasks,
            this.w('tasks'),
            (task, where) => parseID(task, 'AiTask', where),
            { min_len: 1 },
        );
    }

    public override verify() {
        super.verify();

        for (const [idx, task_id] of this.tasks.entries()) {
            const task = AiTask.find(task_id, this.w(`tasks[${idx}]`));
            if (task.character_npc !== this.character_npc) {
                throw this.e(`tasks[${idx}]`, 'AiTaskSequence and AiTask character_npc mismatch');
            }
            if (task instanceof AiTaskSequence) {
                throw this.e(
                    `tasks[${idx}]`,
                    'AiTaskSequence cannot contain another AiTaskSequence',
                );
            }
        }
    }
}

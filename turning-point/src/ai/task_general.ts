import { ID, parseArray, parseID } from '../common';
import { Action } from '../action';
import { AiIntention, AiTask, AiTaskArgs, parseAiIntention } from './task_base';

export type AiTaskGeneralArgs = AiTaskArgs & {
    /** AI意图 */
    intention?: AiIntention;

    /** AI意图（动作完成后） */
    next_intention?: AiIntention;

    /** 顺序执行的动作列表 */
    actions: ReadonlyArray<ID>;
};

/**
 * AI任务（顺序执行动作）
 */
export class AiTaskGeneral extends AiTask {
    /** AI意图 */
    public readonly intention: AiIntention;

    /** AI意图（动作完成后） */
    public readonly next_intention: AiIntention;

    /** 顺序执行的动作列表 */
    public readonly actions: ReadonlyArray<ID>;

    public constructor(id: ID, args: AiTaskGeneralArgs) {
        super(id, args);
        this.intention = parseAiIntention(args.intention ?? 'Attack', this.w('intention'));
        this.next_intention = parseAiIntention(
            args.next_intention ?? 'SquareOff',
            this.w('next_intention'),
        );
        this.actions = parseArray(
            args.actions,
            this.w('actions'),
            (action, where) => parseID(action, 'Action', where),
            { min_len: 1 },
        );
    }

    public override verify() {
        super.verify();

        for (const [idx, action_id] of this.actions.entries()) {
            const action = Action.find(action_id, this.w(`actions[${idx}]`));
            if (!action.character_npcs?.includes(this.character_npc)) {
                throw this.e(`actions[${idx}]`, 'AiTaskGeneral and Action mismatch');
            }
        }
    }
}

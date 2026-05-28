import { ID, int, parseArray, parseID, parseInt } from '../common';
import { Action, LEVEL_ATTACK } from '../action';
import { AiTask, AiTaskArgs } from './task_base';

export type AiTaskGeneralArgs = AiTaskArgs & {
    /** 进入等级 */
    enter_level?: int;

    /** 维持等级 */
    keep_level?: int;

    /** 顺序执行的动作列表 */
    actions: ReadonlyArray<ID>;
};

/**
 * AI任务（顺序执行动作）
 */
export class AiTaskGeneral extends AiTask {
    /** 进入等级 */
    public readonly enter_level: int;

    /** 维持等级 */
    public readonly keep_level: int;

    /** 顺序执行的动作列表 */
    public readonly actions: ReadonlyArray<ID>;

    public constructor(id: ID, args: AiTaskGeneralArgs) {
        super(id, args);
        this.enter_level = parseInt(args.enter_level ?? LEVEL_ATTACK, this.w('enter_level'), {
            type: 'u16',
        });
        this.keep_level = parseInt(args.keep_level ?? LEVEL_ATTACK, this.w('keep_level'), {
            type: 'u16',
        });
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

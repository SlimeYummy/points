import { float, ID, parseFloat, parseID } from '../common';
import { Action, ActionDodgeNpc } from '../action';
import { AiIntention, AiTask, AiTaskArgs, parseAiIntention } from './task_base';

export type AiTaskKeepDistanceArgs = AiTaskArgs & {
    /** AI意图 */
    intention?: AiIntention;

    /** AI意图（动作完成后） */
    next_intention?: AiIntention;

    /** 闪避动作ID */
    dodge_action: ID;

    /** 期望与目标保持的距离 */
    expected_distance?: float | string;
};

/**
 * AI保持距离任务
 */
export class AiTaskKeepDistance extends AiTask {
    /** AI意图 */
    public readonly intention: AiIntention;

    /** AI意图（动作完成后） */
    public readonly next_intention: AiIntention;

    /** 闪避动作ID */
    public readonly dodge_action: ID;

    /** 期望与目标保持的距离 */
    public readonly expected_distance: float;

    public constructor(id: ID, args: AiTaskKeepDistanceArgs) {
        super(id, args);
        this.intention = parseAiIntention(args.intention ?? 'Attack', this.w('intention'));
        this.next_intention = parseAiIntention(
            args.next_intention ?? 'SquareOff',
            this.w('next_intention'),
        );
        this.dodge_action = parseID(args.dodge_action, 'Action', this.w('dodge_action'));
        this.expected_distance = parseFloat(
            args.expected_distance ?? '0',
            this.w('expected_distance'),
            {
                type: 'f32',
                min: 0,
            },
        );
    }

    public override verify() {
        super.verify();

        const dodge = Action.find(this.dodge_action, this.w('dodge_action'));
        if (!(dodge instanceof ActionDodgeNpc)) {
            throw this.e('dodge_action', 'must be an ActionDodgeNpc');
        }
        if (!dodge.character_npcs?.includes(this.character_npc)) {
            throw this.e('dodge_action', 'AiTaskKeepDistance and ActionDodgeNpc mismatch');
        }
    }
}

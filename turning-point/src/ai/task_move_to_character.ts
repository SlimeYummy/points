import { float, ID, parseAngleXz, parseBool, parseFloatRange, parseID } from '../common';
import { Action, ActionMoveNpc } from '../action';
import { AiIntention, AiTask, AiTaskArgs, parseAiIntention } from './task_base';

export type AiTaskMoveToCharacterArgs = AiTaskArgs & {
    /** AI意图 */
    intention?: AiIntention;

    /** AI意图（动作完成后） */
    next_intention?: AiIntention;

    /** 移动动作 */
    move_action: ID;

    /** 转向动作 */
    turn_action: ID;

    /** 与目标的理想距离 */
    expected_distance: readonly [float | string, float | string];

    /** 与目标的理想朝向（XZ平面上 角色朝向与向量(目标-角色)的夹角半角） */
    expected_toward: float | string;

    /** 在目标改变时退出动作 */
    target_exit?: boolean;
};

/**
 * AI任务（移动到角色）
 */
export class AiTaskMoveToCharacter extends AiTask {
    /** AI意图 */
    public readonly intention: AiIntention;

    /** AI意图（动作完成后） */
    public readonly next_intention: AiIntention;

    /** 移动动作 */
    public readonly move_action: ID;

    /** 转向动作 */
    public readonly turn_action: ID;

    /** 与目标的理想距离 */
    public readonly expected_distance: readonly [float, float];

    /** 与目标的理想朝向（XZ平面上 角色朝向与向量(目标-角色)的夹角半角） */
    public readonly expected_toward: float;

    /** 在目标改变时退出动作 */
    public readonly target_exit: boolean;

    public constructor(id: ID, args: AiTaskMoveToCharacterArgs) {
        super(id, args);
        this.intention = parseAiIntention(args.intention ?? 'Move', this.w('intention'));
        this.next_intention = parseAiIntention(
            args.next_intention ?? 'Idle',
            this.w('next_intention'),
        );
        this.move_action = parseID(args.move_action, 'Action', this.w('move_action'));
        this.turn_action = parseID(args.turn_action, 'Action', this.w('turn_action'));
        this.expected_distance = parseFloatRange(
            args.expected_distance,
            this.w('expected_distance'),
            {
                min: 0,
                type: 'f32',
            },
        );
        this.expected_toward = parseAngleXz(args.expected_toward, this.w('expected_toward'), {
            min: 0,
        });
        this.target_exit = parseBool(args.target_exit ?? false, this.w('target_exit'));
    }

    public override verify() {
        super.verify();

        const move_action = Action.find(this.move_action, this.w('move_action'));
        if (!(move_action instanceof ActionMoveNpc)) {
            throw this.e('move_action', 'must be an ActionMoveNpc');
        }
        if (!move_action.character_npcs?.includes(this.character_npc)) {
            throw this.e('move_action', 'AiTaskMoveToCharacter and ActionMoveNpc mismatch');
        }

        // const turn_action = Action.find(this.turn_action, this.w('turn_action'));
        // if (!(turn_action instanceof ActionMoveNpc)) {
        //     throw this.e('turn_action', 'must be an ActionMoveNpc');
        // }
        // if (!turn_action.character_npcs?.includes(this.character_npc)) {
        //     throw this.e('turn_action', 'AiTaskMoveToCharacter and ActionMoveNpc mismatch');
        // }
    }
}

import {
    float,
    ID,
    int,
    parseAngleXz,
    parseFloatRange,
    parseID,
    parseInt,
} from '../common';
import { Action, ActionMoveNpc, LEVEL_MOVE } from '../action';
import { AiTask, AiTaskArgs } from './task_base';

export type AiTaskMoveToCharacterArgs = AiTaskArgs & {
    /** 进入等级 */
    enter_level?: int;

    /** 维持等级 */
    keep_level?: int;

    /** 移动动作 */
    move_action: ID;

    /** 转向动作 */
    turn_action: ID;

    /** 与目标的理想距离 */
    expected_distance: readonly [float | string, float | string];

    /** 与目标的理想朝向（XZ平面上 角色朝向与向量(目标-角色)的夹角半角） */
    expected_toward: float | string;
};

/**
 * AI任务（移动到角色）
 */
export class AiTaskMoveToCharacter extends AiTask {
    /** 进入等级 */
    public readonly enter_level: int;

    /** 维持等级 */
    public readonly keep_level: int;

    /** 移动动作 */
    public readonly move_action: ID;

    /** 转向动作 */
    public readonly turn_action: ID;

    /** 与目标的理想距离 */
    public readonly expected_distance: readonly [float, float];

    /** 与目标的理想朝向（XZ平面上 角色朝向与向量(目标-角色)的夹角半角） */
    public readonly expected_toward: float;

    public constructor(id: ID, args: AiTaskMoveToCharacterArgs) {
        super(id, args);
        this.enter_level = parseInt(args.enter_level ?? LEVEL_MOVE, this.w('enter_level'), {
            type: 'u16',
        });
        this.keep_level = parseInt(args.keep_level ?? LEVEL_MOVE, this.w('keep_level'), {
            type: 'u16',
        });
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

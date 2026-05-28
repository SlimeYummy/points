import { checkArray, float, ID, int, parseID, parseInt, parseTime, parseVec3 } from '../common';
import { Action, ActionIdle, ActionMoveNpc, LEVEL_MOVE } from '../action';
import { AiTask, AiTaskArgs } from './task_base';

export type AiTaskPatrolArgs = AiTaskArgs & {
    /** 进入等级 */
    enter_level?: int;

    /** 维持等级 */
    keep_level?: int;

    /** 待机动作ID */
    action_idle: ID;

    /** 移动动作ID */
    action_move: ID;

    /** 巡逻路线 */
    route: ReadonlyArray<AiTaskPatrolStepArgs>;
};

export type AiTaskPatrolStepArgs =
    | readonly ['Move', readonly [float | string, float | string, float | string]]
    | readonly ['Idle', float | string];

export type AiTaskPatrolStep =
    | readonly ['Move', readonly [float, float, float]]
    | readonly ['Idle', float];

/**
 * AI任务（巡逻）
 */
export class AiTaskPatrol extends AiTask {
    /** 进入等级 */
    public readonly enter_level: int;

    /** 维持等级 */
    public readonly keep_level: int;

    /** 待机动作ID */
    public readonly action_idle: ID;

    /** 移动动作ID */
    public readonly action_move: ID;

    /** 巡逻路线 */
    public readonly route: ReadonlyArray<AiTaskPatrolStep>;

    public constructor(id: ID, args: AiTaskPatrolArgs) {
        super(id, args);
        this.enter_level = parseInt(args.enter_level ?? LEVEL_MOVE, this.w('enter_level'), {
            type: 'u16',
        });
        this.keep_level = parseInt(args.keep_level ?? LEVEL_MOVE, this.w('keep_level'), {
            type: 'u16',
        });
        this.action_idle = parseID(args.action_idle, 'Action', this.w('action_idle'));
        this.action_move = parseID(args.action_move, 'Action', this.w('action_move'));
        this.route = this.parseRoute(args.route);
    }

    private parseRoute(raw: ReadonlyArray<AiTaskPatrolStepArgs>): ReadonlyArray<AiTaskPatrolStep> {
        const where = this.w('route');
        checkArray(raw, where, { min_len: 1 });

        return raw.map((step, idx) => {
            if (!Array.isArray(step)) {
                throw this.e(`${where}[${idx}]`, 'must be an array');
            }

            if (step[0] === 'Move') {
                return ['Move', parseVec3(step[1] as any, `${where}[${idx}].position`)] as const;
            } else if (step[0] === 'Idle') {
                return [
                    'Idle',
                    parseTime(step[1], `${where}[${idx}].duration`, { min: 0, type: 'f32' }),
                ] as const;
            } else {
                throw this.e(`${where}[${idx}][0]`, 'must be Move|Idle');
            }
        });
    }

    public override verify() {
        super.verify();

        const idle = Action.find(this.action_idle, this.w('action_idle'));
        if (!(idle instanceof ActionIdle)) {
            throw this.e('action_idle', 'must be an ActionIdle');
        }
        if (!idle.character_npcs?.includes(this.character_npc)) {
            throw this.e('action_idle', 'AiTaskPatrol and ActionIdle mismatch');
        }

        const move = Action.find(this.action_move, this.w('action_move'));
        if (!(move instanceof ActionMoveNpc)) {
            throw this.e('action_move', 'must be an ActionMoveNpc');
        }
        if (!move.character_npcs?.includes(this.character_npc)) {
            throw this.e('action_move', 'AiTaskPatrol and ActionMoveNpc mismatch');
        }
    }
}

import { checkArray, float, ID, parseID, parseTime, parseVec3 } from '../common';
import { Action, ActionIdle, ActionMoveNpc } from '../action';
import { AiTask, AiTaskArgs } from './task_base';

export type AiTaskPatrolArgs = AiTaskArgs & {
    /** 待机动作ID */
    action_idle: ID;

    /** 移动动作ID */
    action_move: ID;

    /** 巡逻路线 */
    route: ReadonlyArray<AiTaskPatrolStep>;
};

export type AiTaskPatrolStep =
    | readonly ['Move', readonly [float, float, float]]
    | readonly ['Idle', float | string];

/**
 * AI任务（巡逻）
 */
export class AiTaskPatrol extends AiTask {
    /** 待机动作ID */
    public readonly action_idle: ID;

    /** 移动动作ID */
    public readonly action_move: ID;

    /** 巡逻路线 */
    public readonly route: ReadonlyArray<AiTaskPatrolStep>;

    public constructor(id: ID, args: AiTaskPatrolArgs) {
        super(id, args);
        this.action_idle = parseID(args.action_idle, 'Action', this.w('action_idle'));
        this.action_move = parseID(args.action_move, 'Action', this.w('action_move'));
        this.route = this.parseRoute(args.route);
    }

    private parseRoute(raw: ReadonlyArray<AiTaskPatrolStep>): ReadonlyArray<AiTaskPatrolStep> {
        const where = this.w('route');
        checkArray(raw, where, { min_len: 1 });

        return raw.map((step, idx) => {
            if (!Array.isArray(step)) {
                throw this.error(`${where}[${idx}]`, 'must be an array');
            }

            if (step[0] === 'Move') {
                return ['Move', parseVec3(step[1] as any, `${where}[${idx}].position`)] as const;
            } else if (step[0] === 'Idle') {
                return [
                    'Idle',
                    parseTime(step[1], `${where}[${idx}].duration`, { min: 0 }),
                ] as const;
            } else {
                throw this.error(`${where}[${idx}][0]`, 'must be Move|Idle');
            }
        });
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

        const move = Action.find(this.action_move, this.w('action_move'));
        if (!(move instanceof ActionMoveNpc)) {
            throw this.error('action_move', 'must be an ActionMoveNpc');
        }
        if (!move.character_npcs?.includes(this.character_npc)) {
            throw this.error('action_move', 'AiTaskPatrol and ActionMoveNpc mismatch');
        }
    }
}

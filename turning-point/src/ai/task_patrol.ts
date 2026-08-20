import { checkArray, float, ID, parseBool, parseID, parseTime, parseVec3 } from '../common';
import { Action, ActionIdle, ActionMoveNpc } from '../action';
import { AiIntention, AiTask, AiTaskArgs, parseAiIntention } from './task_base';

export type AiTaskPatrolArgs = AiTaskArgs & {
    /** AI意图 */
    intention?: AiIntention;

    /** AI意图（动作完成后） */
    next_intention?: AiIntention;

    /** 待机动作ID */
    action_idle: ID;

    /** 移动动作ID */
    action_move: ID;

    /** 巡逻路线 */
    route: ReadonlyArray<AiTaskPatrolStepArgs>;

    /** 在目标改变时退出动作 */
    target_exit?: boolean;
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
    /** AI意图 */
    public readonly intention: AiIntention;

    /** AI意图（动作完成后） */
    public readonly next_intention: AiIntention;

    /** 待机动作ID */
    public readonly action_idle: ID;

    /** 移动动作ID */
    public readonly action_move: ID;

    /** 巡逻路线 */
    public readonly route: ReadonlyArray<AiTaskPatrolStep>;

    /** 在目标改变时退出动作 */
    public readonly target_exit: boolean;

    public constructor(id: ID, args: AiTaskPatrolArgs) {
        super(id, args);
        this.intention = parseAiIntention(args.intention ?? 'Move', this.w('intention'));
        this.next_intention = parseAiIntention(
            args.next_intention ?? 'Idle',
            this.w('next_intention'),
        );
        this.action_idle = parseID(args.action_idle, 'Action', this.w('action_idle'));
        this.action_move = parseID(args.action_move, 'Action', this.w('action_move'));
        this.route = this.parseRoute(args.route);
        this.target_exit = parseBool(args.target_exit ?? false, this.w('target_exit'));
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

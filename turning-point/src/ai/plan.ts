import { NpcCharacter } from '../character';
import { float, ID, IDPrefix, parseArray, parseFloat, parseID } from '../common';
import { Resource } from '../resource';
import { AiTask } from './task_base';

export type AiPlanCandidateArgs = {
    /** 行为ID AiTask/AiPlan */
    id: ID;

    /** 概率权重 */
    probability: float | string;
};

export class AiPlanCandidate {
    /** 行为ID AiTask/AiPlan */
    public readonly id: ID;

    /** 概率权重 */
    public readonly probability: float;

    public constructor(args: AiPlanCandidateArgs, where: string) {
        this.id = parseID(args.id, ['AiTask', 'AiPlan'], `${where}.id`);
        this.probability = parseFloat(args.probability, `${where}.probability`, { min: 0 });
    }
}

export type AiPlanArgs = {
    /** 角色ID（仅NpcCharacter） */
    character: ID;

    /** 候选行为列表 */
    candidates: readonly AiPlanCandidateArgs[];
};

/**
 * AI规划
 */
export class AiPlan extends Resource {
    public static override readonly prefix: IDPrefix = 'AiPlan' as const;

    public static override find(id: string, where: string): AiPlan {
        const res = Resource.find(id, where);
        if (!(res instanceof AiPlan)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 角色ID（仅NpcCharacter） */
    public readonly character: ID;

    /** 候选行为列表 */
    public readonly candidates: readonly AiPlanCandidate[];

    public constructor(id: ID, args: AiPlanArgs) {
        super(id);
        this.character = parseID(args.character, 'NpcCharacter', this.w('character'));
        this.candidates = parseArray(
            args.candidates,
            this.w('candidates'),
            (candidate, where) => new AiPlanCandidate(candidate, where),
            { min_len: 0 },
        );
    }

    public override verify() {
        NpcCharacter.find(this.character, this.w('character'));

        for (const [idx, candidate] of this.candidates.entries()) {
            if (candidate.id.startsWith('AiTask')) {
                const task = AiTask.find(candidate.id, this.w(`candidates[${idx}].id`));
                if (task.character !== this.character) {
                    throw this.error(`candidates[${idx}].id`, 'character mismatch');
                }
            } else if (candidate.id.startsWith('AiPlan')) {
                const plan = AiPlan.find(candidate.id, this.w(`candidates[${idx}].id`));
                if (plan.character !== this.character) {
                    throw this.error(`candidates[${idx}].id`, 'character mismatch');
                }
            }
        }
    }
}

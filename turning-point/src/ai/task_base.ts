import { ID, IDPrefix } from '../common';
import { Resource } from '../resource';
import { CharacterNpc } from '../character';

export const AI_INTENTION = ['Idle', 'Move', 'Attack', 'SquareOff'] as const;

export type AiIntention = (typeof AI_INTENTION)[number];

export function isAiIntention(raw: string): raw is AiIntention {
    return AI_INTENTION.includes(raw as AiIntention);
}

export function parseAiIntention(raw: string, where: string): AiIntention {
    if (!AI_INTENTION.includes(raw as AiIntention)) {
        throw new Error(where + ': must be a AiIntention ');
    }
    return raw as AiIntention;
}

export type AiTaskArgs = {
    /** 角色ID（仅CharacterNpc） */
    character_npc: ID;
};

/**
 * 所有AI任务的抽象基类
 */
export abstract class AiTask extends Resource {
    public static override readonly prefix: IDPrefix = 'AiTask';

    public static override find(id: ID, where: string): AiTask {
        const res = Resource.find(id, where);
        if (!(res instanceof AiTask)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 角色ID（仅CharacterNpc） */
    public readonly character_npc: ID;

    public constructor(id: ID, args: AiTaskArgs) {
        super(id);
        this.character_npc = args.character_npc;
    }

    public override verify() {
        CharacterNpc.find(this.character_npc, this.w('character_npc'));
    }
}

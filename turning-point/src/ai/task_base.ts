import { ID, IDPrefix } from '../common';
import { Resource } from '../resource';
import { CharacterNpc } from '../character';

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

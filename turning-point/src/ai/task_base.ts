import { NpcCharacter } from '../character';
import { ID, IDPrefix } from '../common';
import { Resource } from '../resource';

export type AiTaskArgs = {
    /** 角色ID（仅NpcCharacter） */
    character: ID;
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

    /** 角色ID（仅NpcCharacter） */
    public readonly character: ID;

    public constructor(id: ID, args: AiTaskArgs) {
        super(id);
        this.character = args.character;
    }

    public override verify() {
        NpcCharacter.find(this.character, this.w('character'));
    }
}

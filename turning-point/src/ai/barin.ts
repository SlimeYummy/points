import { NpcCharacter } from '../character';
import { float, ID, IDPrefix, int, parseArray, parseFloat, parseID, parseInt, parseTime } from '../common';
import { Sphere, SphereArgs, SphericalCone, SphericalConeArgs } from '../common/shape';
import { Resource } from '../resource';
import { AiPlan } from './plan';
import { AiTask } from './task_base';

export type AiIfArgs = {

}

export class AiIf {
    public constructor(args: AiIfArgs, where: string) {
    }
}

export type AiNodeTaskArgs = {
    /** 任务ID */
    task: ID;

    /** 进入权重 */
    weight?: float | string;

    /** 优先级 */
    priority?: int;

    /** 进入条件 */
    conditions?: AiIfArgs[];
};

export class AiNodeTask {
    public T: 'Task' = 'Task';

    /** 任务ID */
    public task: ID;

    /** 进入权重 */
    public weight: float;

    /** 优先级 */
    public priority: int;

    /** 进入条件 */
    public conditions: ReadonlyArray<AiIf>;

    public constructor(args: AiNodeTaskArgs, where: string) {
        this.task = parseID(args.task, 'AiTask', `${where}.task`);
        this.weight = args.weight == null ? 1 : parseFloat(args.weight, `${where}.weight`);
        this.priority = args.priority == null ? 0 : parseInt(args.priority, `${where}.priority`);
        this.conditions = args.conditions == null ? [] : parseArray(args.conditions, `${where}.conditions`, (condition, where) => new AiIf(condition, where));
    }
}

export type AiNodeBranchArgs = {
    /** 进入条件 */
    conditions?: AiIfArgs[];

    /** 子节点 */
    nodes?: AiNodeArgs[];
};

export class AiNodeBranch {
    public T: 'Branch' = 'Branch';

    /** 进入条件 */
    public conditions: ReadonlyArray<AiIf>;

    /** 子节点 */
    public nodes: ReadonlyArray<AiNode>;

    public constructor(nodes: ReadonlyArray<AiNode>);
    public constructor(args: AiNodeBranchArgs, where: string);
    public constructor(...args: any[]) {
        if (args.length === 1) {
            this.nodes = args[0] as ReadonlyArray<AiNode>;
            this.conditions = [];
        } else {
            const args2 = args[0] as AiNodeBranchArgs;
            const where = args[1] as string;
            this.conditions = args2.conditions == null ? [] : parseArray(args2.conditions, `${where}.conditions`, (condition, where) => new AiIf(condition, where));
            this.nodes = args2.nodes == null ? [] : parseAiNodes(args2.nodes, `${where}.nodes`);
        }
    }
}

export type AiNodeArgs = AiNodeTaskArgs | AiNodeBranchArgs;
export type AiNode = AiNodeTask | AiNodeBranch;

function parseAiNodes(arrayArgs: AiNodeArgs[], where: string): ReadonlyArray<AiNode> {
    return parseArray(arrayArgs, where, (nodeArgs, index) => {
        if ('task' in nodeArgs) {
            return new AiNodeTask(nodeArgs, `${where}.${index}`);
        } else {
            return new AiNodeBranch(nodeArgs, `${where}.${index}`);
        }
    });
}

export type AiBrainArgs = {
    /** 角色ID（仅NpcCharacter） */
    character: ID;

    /** 警戒范围（球形） */
    alert_sphere: SphereArgs;

    /** 警戒范围（锥形） */
    alert_cone: SphericalConeArgs;

    /** 攻击退出延迟 */
    attack_exit_delay: float | string;

    /** 待机行为 */
    idle_nodes: AiNodeArgs[];

    // /** 攻击行为 */
    // attack_nodes: AiNodeArgs[];
};

/**
 * AI执行器
 */
export class AiBrain extends Resource {
    public static override readonly prefix: IDPrefix = 'AiBrain' as const;

    public static override find(id: string, where: string): AiBrain {
        const res = Resource.find(id, where);
        if (!(res instanceof AiBrain)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 角色ID（仅NpcCharacter） */
    public readonly character: ID;

    /** 警戒范围（球形） */
    public readonly alert_sphere: Sphere;

    /** 警戒范围（锥形） */
    public readonly alert_cone: SphericalCone;

    /** 攻击退出延迟 */
    public readonly attack_exit_delay: float;

    /** 待机行为 */
    public readonly idle: AiNodeBranch;

    // /** 攻击行为 */
    // public readonly attack_nodes: ReadonlyArray<AiNode>;

    public constructor(id: ID, args: AiBrainArgs) {
        super(id);
        this.character = parseID(args.character, 'NpcCharacter', this.w('character'));
        this.alert_sphere = new Sphere(args.alert_sphere, this.w('alert_sphere'));
        this.alert_cone = new SphericalCone(args.alert_cone, this.w('alert_cone'));
        this.attack_exit_delay = parseTime(args.attack_exit_delay, this.w('attack_exit_delay'), { min: 0 });
        this.idle = new AiNodeBranch(parseAiNodes(args.idle_nodes, this.w('idle_nodes')));
    }

    public override verify() {
        NpcCharacter.find(this.character, this.w('character'));
        this.verify_nodes(this.idle.nodes, [], this.w('idle.nodes'));
    }

    private verify_nodes(nodes: ReadonlyArray<AiNode>, branch_stack: AiNodeBranch[], where: string) {
        for (const [idx, node] of nodes.entries()) {
            if (node instanceof AiNodeTask) {
                const task = AiTask.find(node.task, `${where}[${idx}].task`);
                if (task.character !== this.character) {
                    throw this.error(`${where}[${idx}].task`, 'character mismatch');
                }

            } else if (node instanceof AiNodeBranch) {
                if (branch_stack.includes(node)) {
                    throw this.error(`${where}[${idx}]`, 'circular reference');
                }

                branch_stack.push(node);
                this.verify_nodes(node.nodes, branch_stack, `${where}[${idx}].nodes`);
                branch_stack.pop();
            }
        }
    }

    public static task(task: ID): AiNodeTaskArgs;
    public static task(task: ID, weight: float | string): AiNodeTaskArgs;
    public static task(task: ID, weight: float | string, conditions: AiIfArgs[]): AiNodeTaskArgs;
    public static task(task: ID, weight: float | string, priority: int): AiNodeTaskArgs;
    public static task(task: ID, weight: float | string, priority: int, conditions: AiIfArgs[]): AiNodeTaskArgs;
    public static task(...args: any[]): AiNodeTaskArgs {
        const task = args[0] as ID;
        if (args.length === 1) {
            return { task };
        } else if (args.length === 2) {
            return { task, weight: args[1] };
        } else if (args.length === 3) {
            if (Array.isArray(args[2])) {
                return { task, weight: args[1], conditions: args[2] };
            } else {
                return { task, weight: args[1], priority: args[2] };
            }
        } else {
            return { task, weight: args[1], priority: args[2], conditions: args[3] };
        }
    }

    public static branch(conditions: AiIfArgs[], nodes: AiNodeArgs[]): AiNodeBranchArgs {
        return { nodes, conditions };
    }
}

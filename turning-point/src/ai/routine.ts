import { ID, IDPrefix, int, parseID, parseInt } from '../common';
import { Resource } from '../resource';
import { CharacterNpc } from '../character';
import { ScriptIf } from '../script';
import { AiTask } from './task_base';

export type AiRoutineItem = AiRoutineItemTask | AiRoutineItemIf | AiRoutineItemElse;

export class AiRoutineItemTask {
    public id: ID;

    public constructor(id: ID, where: string) {
        this.id = parseID(id, 'AiTask', where);
    }

    public toJSON() {
        return { T: 'Task', id: this.id };
    }
}

export class AiRoutineItemIf {
    public readonly script: ScriptIf;
    public jump: int;

    public constructor(script: ScriptIf, jump: int) {
        this.script = script;
        this.jump = jump;
    }

    public toJSON() {
        return {
            T: 'If',
            script: this.script.toJSON(),
            jump: this.jump,
        };
    }
}

export class AiRoutineItemElse {
    public jump: int;

    public constructor(jump: int) {
        this.jump = jump;
    }

    public toJSON() {
        return { T: 'Else', jump: this.jump };
    }
}

export type AiRoutineArgs = {
    /** 角色ID（仅CharacterNpc） */
    character_npc: ID;

    /** 子任务列表（AiTask ID） */
    tasks: ReadonlyArray<ID | IfNode | IfBuilder>;
};

/**
 * AI过程（多任务组）
 */
export class AiRoutine extends Resource {
    public static override readonly prefix: IDPrefix = 'AiRoutine';

    public static override find(id: ID, where: string): AiRoutine {
        const res = Resource.find(id, where);
        if (!(res instanceof AiRoutine)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 角色ID（仅CharacterNpc） */
    public readonly character_npc: ID;

    /** 子任务列表（AiTask ID） */
    public readonly tasks: ReadonlyArray<AiRoutineItem>;

    public constructor(id: ID, args: AiRoutineArgs) {
        super(id);
        this.character_npc = args.character_npc;
        this.tasks = this.parseTasks(args.tasks, this.w('tasks'));
    }

    private parseTasks(
        tasks: ReadonlyArray<ID | IfNode | IfBuilder>,
        where: string,
    ): AiRoutineItem[] {
        const result: AiRoutineItem[] = [];

        const visit = (list: ReadonlyArray<ID | IfNode | IfBuilder>, where: string) => {
            for (const [i, item] of list.entries()) {
                if (typeof item === 'string') {
                    result.push(new AiRoutineItemTask(item, `${where}[${i}]`));
                } else if (typeof item === 'object' && item) {
                    const node = item instanceof IfBuilder ? item.root : (item as IfNode);

                    if (!Array.isArray(node.then) || node.then.length <= 0) {
                        this.e(`${where}[${i}].then`, 'invalid or empty');
                    }

                    const script = new ScriptIf(
                        node.if,
                        this.id,
                        this.character_npc,
                        this.w(`${where}[${i}].if`),
                        {
                            func: 'if',
                            type: node.type || 'bool',
                        },
                    );

                    const itemIf = new AiRoutineItemIf(script, 0);
                    result.push(itemIf);
                    visit(node.then, `${where}[${i}].then`);

                    if (!Array.isArray(node.else) || node.else.length <= 0) {
                        itemIf.jump = result.length;
                    } else {
                        const itemElse = new AiRoutineItemElse(0);
                        result.push(itemElse);
                        itemIf.jump = result.length;
                        visit(node.else, `${where}[${i}].else`);
                        itemElse.jump = result.length;
                    }
                } else {
                    throw this.e(`${where}[${i}]`, 'must be string | { if, then, else }');
                }
            }
        };

        if (!tasks || tasks.length === 0) {
            throw new Error(`${where}: empty tasks`);
        }

        visit(tasks, where);
        return result;
    }

    public override verify() {
        CharacterNpc.find(this.character_npc, this.w('character_npc'));

        for (const [idx, item] of this.tasks.entries()) {
            if (item instanceof AiRoutineItemTask) {
                const task = AiTask.find(item.id, this.w(`tasks[${idx}]`));
                if (task.character_npc !== this.character_npc) {
                    throw this.e(`tasks[${idx}]`, 'AiRoutine and AiTask character_npc mismatch');
                }
                if (task instanceof AiRoutine) {
                    throw this.e(`tasks[${idx}]`, 'AiRoutine cannot contain another AiRoutine');
                }
            }
        }
    }
}

export type IfNode = {
    /** 条件脚本（返回bool） */
    if: string;

    /** 条件为true时执行 */
    then: ReadonlyArray<ID | IfNode>;

    /** 条件为false时执行 */
    else?: ReadonlyArray<ID | IfNode>;

    /** 生成的函数类型 默认bool */
    type?: 'bool' | 'result';
};

/**
 * AI条件构造器
 */
export class IfBuilder {
    public root: IfNode;
    #current: IfNode | null;

    public constructor(if_: string, then_: ReadonlyArray<ID | IfNode>, type_: 'bool' | 'result') {
        this.root = {
            if: if_,
            then: then_,
            type: type_,
        };
        this.#current = this.root;
    }

    public Elsif(cond: string, items: ReadonlyArray<ID | IfNode>): IfBuilder;
    public Elsif(cond: string, ...items: ReadonlyArray<ID | IfNode>): IfBuilder;
    public Elsif(if_: string, ...args: any[]): IfBuilder {
        const then = args.length === 1 && Array.isArray(args[0]) ? args[0] : args;
        return this.elsifImpl(if_, then, 'bool');
    }

    public Elsif_R(cond: string, items: ReadonlyArray<ID | IfNode>): IfBuilder;
    public Elsif_R(cond: string, ...items: ReadonlyArray<ID | IfNode>): IfBuilder;
    public Elsif_R(if_: string, ...args: any[]): IfBuilder {
        const then = args.length === 1 && Array.isArray(args[0]) ? args[0] : args;
        return this.elsifImpl(if_, then, 'result');
    }

    public elsifImpl(
        if_: string,
        then_: ReadonlyArray<ID | IfNode>,
        type_: 'bool' | 'result',
    ): IfBuilder {
        if (!this.#current) {
            throw new Error('IfBuilder already closed');
        }
        const newIf: IfNode = {
            if: if_,
            then: then_,
            type: type_,
        };
        this.#current.else = [newIf];
        this.#current = newIf;
        return this;
    }

    public Else(items: ReadonlyArray<ID | IfNode>): IfBuilder;
    public Else(...items: ReadonlyArray<ID | IfNode>): IfBuilder;
    public Else(...args: any[]): IfBuilder {
        if (!this.#current) {
            throw new Error('IfBuilder already closed');
        }
        this.#current.else = args.length === 1 && Array.isArray(args[0]) ? args[0] : args;
        this.#current = null;
        return this;
    }
}

export function If(if_: string, ...args: any[]): IfBuilder {
    const real_if = if_;
    const real_then = args.length === 1 && Array.isArray(args[0]) ? args[0] : args;
    return new IfBuilder(real_if, real_then, 'bool');
}

export function If_R(if_: string, ...args: any[]): IfBuilder {
    const real_if = if_;
    const real_then = args.length === 1 && Array.isArray(args[0]) ? args[0] : args;
    return new IfBuilder(real_if, real_then, 'result');
}

import { ID, IDPrefix, int, parseID, parseInt } from '../common';
import { Resource } from '../resource';
import { CharacterNpc } from '../character';
import { Script } from '../script';
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
    public readonly script: Script;
    public jump: int;

    public constructor(script: Script, jump: int) {
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

export type AiRoutineIf = {
    /** 条件脚本（返回bool） */
    if: string;

    /** 条件为true时执行 */
    then: ReadonlyArray<ID | AiRoutineIf>;

    /** 条件为false时执行 */
    else?: ReadonlyArray<ID | AiRoutineIf>;
};

export type AiRoutineArgs = {
    /** 角色ID（仅CharacterNpc） */
    character_npc: ID;

    /** 子任务列表（AiTask ID） */
    tasks: ReadonlyArray<ID | AiRoutineIf>;
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

    private parseTasks(tasks: ReadonlyArray<ID | AiRoutineIf>, where: string): AiRoutineItem[] {
        const result: AiRoutineItem[] = [];
        let func_no = 1;

        const visit = (list: ReadonlyArray<ID | AiRoutineIf>, where: string) => {
            for (const [i, item] of list.entries()) {
                if (typeof item === 'string') {
                    result.push(new AiRoutineItemTask(item, `${where}[${i}]`));
                } else if (typeof item === 'object' && item) {
                    if (!Array.isArray(item.then) || item.then.length <= 0) {
                        this.e(`${where}[${i}].then`, 'invalid or empty');
                    }

                    const script = new Script(item.if, this.id, this.w(`${where}[${i}].if`), {
                        func: 'if',
                        func_no: func_no++,
                    });

                    const itemIf = new AiRoutineItemIf(script, 0);
                    result.push(itemIf);
                    visit(item.then, `${where}[${i}].then`);

                    if (!Array.isArray(item.else) || item.else.length <= 0) {
                        itemIf.jump = result.length;
                    } else {
                        const itemElse = new AiRoutineItemElse(0);
                        result.push(itemElse);
                        itemIf.jump = result.length;
                        visit(item.else, `${where}[${i}].else`);
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

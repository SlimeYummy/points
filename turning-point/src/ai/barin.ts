import { float, ID, IDPrefix, parseArray, parseID, parseIDArray, parseTime } from '../common';
import { Resource } from '../resource';
import { CharacterNpc } from '../character';
import { Sphere, SphereArgs, SphericalCone, SphericalConeArgs } from '../common/shape';
import { Script } from '../script';
import { AiRoutine } from './routine';
import { AiTask } from './task_base';

export type AiBrainArgs = {
    /** 角色ID（仅CharacterNpc） */
    character_npc: ID;

    /** 警戒范围（球形） */
    alert_sphere: SphereArgs;

    /** 警戒范围（锥形） */
    alert_cone: SphericalConeArgs;

    /** 仇恨范围（球形） */
    aggro_sphere: SphereArgs;

    /** 丢弃仇恨时间 */
    aggro_lost_time?: float | string;

    /** 是否从脚本中提取任务 */
    tasks_from_script?: boolean;

    /** 可用任务列表 */
    tasks?: ID[];

    /** AI执行脚本 */
    execute?: string;
};

/**
 * AI控制器
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

    /** 角色ID（仅CharacterNpc） */
    public readonly character_npc: ID;

    /** 警戒范围（球形） */
    public readonly alert_sphere: Sphere;

    /** 警戒范围（锥形） */
    public readonly alert_cone: SphericalCone;

    /** 仇恨范围（球形） */
    public readonly aggro_sphere: Sphere;

    /** 丢弃仇恨时间 */
    public readonly aggro_lost_time: float;

    /** 可用任务列表 */
    public readonly tasks: ReadonlyArray<ID>;

    /** AI执行脚本 */
    public readonly execute: Script;

    public constructor(id: ID, args: AiBrainArgs) {
        super(id);
        this.character_npc = parseID(args.character_npc, 'CharacterNpc', this.w('character_npc'));
        this.alert_sphere = new Sphere(args.alert_sphere, this.w('alert_sphere'));
        this.alert_cone = new SphericalCone(args.alert_cone, this.w('alert_cone'));
        this.aggro_sphere = new Sphere(args.aggro_sphere, this.w('aggro_sphere'));
        this.aggro_lost_time = parseTime(args.aggro_lost_time ?? '10s', this.w('aggro_lost_time'), {
            min: 0,
            type: 'f32',
        });
        this.execute = new Script(args.execute, this.id, this.w('execute'), { func: 'execute' });
        this.tasks = this.parseTasks(args.tasks || [], args.tasks_from_script || false);
    }

    private parseTasks(tasks: ID[], tasks_from_script: boolean): ReadonlyArray<ID> {
        const res_tasks = [...tasks];
        if (tasks_from_script && this.execute.code) {
            // Extract AiTask and AiRoutine IDs
            const matches = this.execute.code.matchAll(/id!\("(Ai(?:Task|Routine)\.[^"]+)"\)/g);
            for (const match of matches) {
                const id = match[1];
                if (id && !res_tasks.includes(id)) {
                    res_tasks.push(id);
                }
            }
        }
        return parseIDArray(res_tasks, ['AiTask', 'AiRoutine'], this.w('tasks'));
    }

    public override verify() {
        CharacterNpc.find(this.character_npc, this.w('character_npc'));

        for (const [idx, id] of this.tasks.entries()) {
            if (id.startsWith('AiTask.')) {
                const task = AiTask.find(id, this.w(`tasks[${idx}]`));
                if (task.character_npc !== this.character_npc) {
                    throw this.error(`tasks[${idx}]`, 'character_npc mismatch');
                }
            } else if (id.startsWith('AiRoutine.')) {
                const routine = AiRoutine.find(id, this.w(`tasks[${idx}]`));
                if (routine.character_npc !== this.character_npc) {
                    throw this.error(`tasks[${idx}]`, 'character_npc mismatch');
                }
            }
        }
    }
}

import { ID, int } from '../common';
import { Resource } from '../resource';
import { Var, VarValueArgs, verifyVarValue } from '../variable';
import {
    Action,
    ActionArgs,
    ActionInsert,
    ActionPhase,
    ActionPhaseArgs,
    LEVEL_IDLE,
    parseActionDeriveVarTable,
    parseActionLevel,
    parseActionLevelArray,
    parseActionPhaseArray,
    parseVarActionInserts,
    parseVirtualKey,
    verifyActionDeriveVarTable,
    VirtualKey,
} from './base';

export type ActionGeneralArgs = ActionArgs & {
    /** 进入按键 */
    enter_key?: VirtualKey;

    /** 进入等级 */
    enter_level?: int;

    /** 各阶段详细数值配置 */
    phases: ReadonlyArray<ActionPhaseArgs>;

    /** 各阶段派生等级 */
    derive_levels: ReadonlyArray<int>;

    /** 派生列表（后摇阶段） */
    derives?: Readonly<Partial<Record<VirtualKey, ID | VarValueArgs<ID>>>>;

    /** 可插入的动作 */
    insers?: ReadonlyArray<ActionInsert> | VarValueArgs<ReadonlyArray<ActionInsert>>;
};

/**
 * 最普通的单次攻击动作
 */
export class ActionGeneral extends Action {
    public static override find(id: string, where: string): ActionGeneral {
        const res = Resource.find(id, where);
        if (!(res instanceof ActionGeneral)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 进入按键 */
    public readonly enter_key?: VirtualKey;

    /** 进入等级 */
    public readonly enter_level: int;

    /** 各阶段详细数值配置 */
    public readonly phases: ReadonlyArray<ActionPhase>;

    /** 各阶段派生等级 */
    public readonly derive_levels: ReadonlyArray<int>;

    /** 派生列表 */
    public readonly derives?: Readonly<Partial<Record<VirtualKey, ID | Var<ID>>>>;

    /** 可插入的动作 */
    public readonly insers?: ReadonlyArray<ActionInsert> | Var<ReadonlyArray<ActionInsert>>;

    public constructor(id: ID, args: ActionGeneralArgs) {
        super(id, args);
        this.enter_key =
            args.enter_key == null
                ? undefined
                : parseVirtualKey(args.enter_key, this.w('enter_key'));
        this.enter_level = parseActionLevel(args.enter_level || LEVEL_IDLE, this.w('enter_level'));
        this.phases = parseActionPhaseArray(args.phases, this.w('phases'));
        this.derive_levels = parseActionLevelArray(args.derive_levels, this.w('derive_levels'));
        this.derives = !args.derives
            ? undefined
            : parseActionDeriveVarTable(args.derives, this.w('derives'));
        this.insers = !args.insers
            ? undefined
            : parseVarActionInserts(args.insers, this.w('insers'));
    }

    public override verify(): void {
        super.verify();

        if (this.derives) {
            verifyActionDeriveVarTable(this.derives, { styles: this.styles }, this.w('derives'));
        }
    }
}

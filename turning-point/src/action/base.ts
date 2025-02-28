import { Character, Style } from '../character';
import { checkArray, float, ID, IDPrefix, int, parseFile, parseID, parseIDArray, parseInt, parseIntArray } from '../common';
import { Resource } from '../resource';
import {
    parseVarBool,
    parseVarFloat,
    parseVarID,
    parseVarInt,
    parseVarValueArgs,
    Var,
    VarValueArgs,
    verifyVarValue,
} from '../variable';

export const VIRTUAL_KEYS = [
    // 基础操作
    'Move',
    'View',
    'Dodge',
    'Jump',
    'Guard',
    'Interact',
    'Lock',

    // 基本攻击动作
    'Attack1',
    'Attack2',
    'Attack3',
    'Attack4',
    'Attack5',
    'Attack6',
    'Attack7',
    'Spell',
    'Shot1',
    'Shot2',
    'Aim',
    'Switch',

    // 技能动作
    'Skill1',
    'Skill2',
    'Skill3',
    'Skill4',
    'Skill5',
    'Skill6',
    'Skill7',
    'Skill8',

    // 派生动作
    'Derive1',
    'Derive2',
    'Derive3',

    // 道具使用
    'Item1',
    'Item2',
    'Item3',
    'Item4',
    'Item5',
    'Item6',
    'Item7',
    'Item8',

    // 事件触发反馈动作
    'Idle',
    'Walk',
    'Run',
    'Dash',
    'Break1',
    'Break2',
    'Break3',
] as const;

export type VirtualKey = (typeof VIRTUAL_KEYS)[number];

export const Move = 'Move' as const;
export const View = 'View' as const;
export const Dodge = 'Dodge' as const;
export const Jump = 'Jump' as const;
export const Guard = 'Guard' as const;
export const Interact = 'Interact' as const;
export const Lock = 'Lock' as const;

export const Attack1 = 'Attack1' as const;
export const Attack2 = 'Attack2' as const;
export const Attack3 = 'Attack3' as const;
export const Attack4 = 'Attack4' as const;
export const Attack5 = 'Attack5' as const;
export const Attack6 = 'Attack6' as const;
export const Attack7 = 'Attack7' as const;
export const Spell = 'Spell' as const;
export const Shot1 = 'Shot1' as const;
export const Shot2 = 'Shot2' as const;
export const Aim = 'Aim' as const;
export const Switch = 'Switch' as const;

export const Skill1 = 'Skill1' as const;
export const Skill2 = 'Skill2' as const;
export const Skill3 = 'Skill3' as const;
export const Skill4 = 'Skill4' as const;
export const Skill5 = 'Skill5' as const;
export const Skill6 = 'Skill6' as const;
export const Skill7 = 'Skill7' as const;
export const Skill8 = 'Skill8' as const;

export const Derive1 = 'Derive1' as const;
export const Derive2 = 'Derive2' as const;
export const Derive3 = 'Derive3' as const;

export const Item1 = 'Item1' as const;
export const Item2 = 'Item2' as const;
export const Item3 = 'Item3' as const;
export const Item4 = 'Item4' as const;
export const Item5 = 'Item5' as const;
export const Item6 = 'Item6' as const;
export const Item7 = 'Item7' as const;
export const Item8 = 'Item8' as const;

export const Idle = 'Idle' as const;
export const Walk = 'Walk' as const;
export const Run = 'Run' as const;
export const Dash = 'Dash' as const;
export const Break1 = 'Break1' as const;
export const Break2 = 'Break2' as const;
export const Break3 = 'Break3' as const;

export function isVirtualKey(raw: string): raw is VirtualKey {
    return VIRTUAL_KEYS.includes(raw as VirtualKey);
}

export function parseVirtualKey(raw: string, where: string): VirtualKey {
    if (!VIRTUAL_KEYS.includes(raw as VirtualKey)) {
        throw new Error(where + ': must be a VirtualKey');
    }
    return raw as VirtualKey;
}

/** 空闲态 可派生任何动作的状态 最低的派生/进入等级 */
export const LEVEL_IDLE = 0;

/** 移动态 比空闲态稍高 用于区分Idel/Move的优先级 */
export const LEVEL_MOVE = 50;

/** 攻击派生态 普通攻击动作的后摇阶段 */
export const LEVEL_ATTACK_DERIVE = 100;

/** 技能派生态 普通技能动作的后摇阶段 */
export const LEVEL_SKILL_DERIVE = 200;

/** 大招派生态 大招动作的后摇阶段 */
export const LEVEL_ULTIMATE_DERIVE = 300;

/** 动作态 所有动作前摇/执行中的状态 */
export const LEVEL_ACTION = 400;

/** 不可中断态 最高等级的状态 无法中断 做特殊使用 */
export const LEVEL_UNBREAKABLE = 500;

export function isActionLevel(raw: int): raw is int {
    return raw >= LEVEL_IDLE && raw <= LEVEL_UNBREAKABLE;
}

export function parseActionLevel(
    raw: int,
    where: string,
    opts: {
        min?: int;
        max?: int;
    } = {},
): int {
    return parseInt(raw, where, {
        ...opts,
        min: Math.max(LEVEL_IDLE, opts.min ?? LEVEL_IDLE),
        max: Math.min(LEVEL_UNBREAKABLE, opts.max ?? LEVEL_UNBREAKABLE),
    });
}

export function parseActionLevelArray(
    raw: ReadonlyArray<int>,
    where: string,
    opts: {
        min?: int;
        max?: int;
        len?: int;
        min_len?: int;
        max_len?: int;
        add_first?: int;
    } = {},
): ReadonlyArray<int> {
    return parseIntArray(raw, where, {
        ...opts,
        min: Math.max(LEVEL_IDLE, opts.min ?? LEVEL_IDLE),
        max: Math.min(LEVEL_UNBREAKABLE, opts.max ?? LEVEL_UNBREAKABLE),
    });
}

export const ACTION_INSERTS = ['Dodge', 'PerfectDodge', 'Guard', 'PerfectGuard'] as const;

export type ActionInsert = (typeof ACTION_INSERTS)[number];

export const PerfectDodge = 'PerfectDodge' as const;
export const PerfectGuard = 'PerfectGuard' as const;

export function isActionInsert(raw: string): raw is ActionInsert {
    return ACTION_INSERTS.includes(raw as ActionInsert);
}

export function parseActionInsert(raw: string, where: string): ActionInsert {
    if (!ACTION_INSERTS.includes(raw as ActionInsert)) {
        throw new Error(where + ': must be a ActionInsert');
    }
    return raw as ActionInsert;
}

export function parseActionInserts(
    raw: ReadonlyArray<string>,
    where: string,
): ReadonlyArray<ActionInsert> {
    checkArray(raw, where);
    const res: Array<ActionInsert> = [];
    for (const [idx, item] of raw.entries()) {
        if (!res.includes(item as ActionInsert)) {
            res.push(parseActionInsert(item, `${where}[${idx}]`));
        }
    }
    return res;
}

export function parseActionInsertsArray(
    raw: ReadonlyArray<ReadonlyArray<string>>,
    where: string,
    opts: {
        len?: int;
        min_len?: int;
        max_len?: int;
    } = {},
): ReadonlyArray<ReadonlyArray<ActionInsert>> {
    checkArray(raw, where, opts);
    const res: Array<ReadonlyArray<ActionInsert>> = [];
    for (const [idx, item] of raw.entries()) {
        res.push(parseActionInserts(item, `${where}[${idx}]`));
    }
    return res;
}

export function parseVarActionInserts(
    raw: ReadonlyArray<ActionInsert> | VarValueArgs<ReadonlyArray<ActionInsert>>,
    where: string,
    opts: {
        must_var?: boolean;
        len?: int;
        min_len?: int;
        max_len?: int;
    } = {},
): ReadonlyArray<ActionInsert> | Var<ReadonlyArray<ActionInsert>> {
    return parseVarValueArgs(raw, where, opts, parseActionInserts, parseActionInsertsArray);
}

export type ActionArgs = {
    /** 动作配置文件 */
    config: string;

    /** 是否启用 */
    enabled?: boolean | int | VarValueArgs<boolean | int>;

    /** 所属角色ID Action关联的Style应当属于该Character */
    character: ID;

    /** 可以使用该动作的角色风格 */
    styles: ReadonlyArray<ID>;
};

/**
 * 所有动作的抽象基类
 */
export abstract class Action extends Resource {
    public static override prefix: IDPrefix = 'Action';

    public static override find(id: string, where: string): Action {
        const res = Resource.find(id, where);
        if (!(res instanceof Action)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 动作配置文件 */
    public readonly config: string;

    /** 是否启用 */
    public readonly enabled?: boolean | Var<boolean>;

    /** 所属角色ID Action关联的Style应当属于该Character */
    public readonly character: ID;

    /** 可以使用该动作的角色风格 */
    public readonly styles: ReadonlyArray<ID>;

    public constructor(id: ID, args: ActionArgs) {
        super(id);
        this.config = parseFile(args.config, this.w('config'), { extension: '.json' });
        this.enabled = parseVarBool(args.enabled || true, this.w('enabled'));
        this.character = parseID(args.character, 'Character', this.w('character'));
        this.styles = parseIDArray(args.styles, 'Style', this.w('styles'));
    }

    public override verify() {
        verifyVarValue(this.enabled, { styles: this.styles }, this.w('enabled'));

        Character.find(this.character, this.w('character'));

        if (this.styles) {
            for (const [idx, id] of this.styles.entries()) {
                const usableStyle = Style.find(id, this.w(`styles[${idx}]`));
                if (this.character !== usableStyle.character) {
                    throw this.e(`styles[${idx}]`, 'character mismatch with styles');
                }
            }
        }
    }
}

export type ActionPhaseArgs = {
    /** 派生等级 */
    derive_level?: int | VarValueArgs<int>;

    /** 伤害减免 */
    damage_rdc?: float | VarValueArgs<float>;

    /** 护盾伤害减免 */
    shield_dmg_rdc?: float | VarValueArgs<float>;

    /** 韧性等级 */
    poise_level?: int | VarValueArgs<int>;
};

export class ActionPhase {
    /** 派生等级 */
    public readonly derive_level: int | Var<int>;

    /** 伤害减免 */
    public readonly damage_rdc: float | Var<float>;

    /** 护盾伤害减免 */
    public readonly shield_dmg_rdc: float | Var<float>;

    /** 韧性等级 */
    public readonly poise_level: int | Var<int>;

    public constructor(args: ActionPhaseArgs, where: string) {
        this.derive_level = parseVarInt(args.derive_level || 100, `${where}.derive_level`);
        this.damage_rdc = parseVarFloat(args.damage_rdc || 0, `${where}.damage_rdc`);
        this.shield_dmg_rdc = parseVarFloat(args.shield_dmg_rdc || 0, `${where}.shield_dmg_rdc`);
        this.poise_level = parseVarInt(args.poise_level || 0, `${where}.poise_level`);
    }
}

export function parseActionPhase(raw: ActionPhaseArgs, where: string): ActionPhase {
    return new ActionPhase(raw, where);
}

export function parseActionPhaseArray(
    raw: ReadonlyArray<ActionPhaseArgs>,
    where: string,
    opts: {
        len?: int;
        min_len?: int;
        max_len?: int;
    } = {},
): ReadonlyArray<ActionPhase> {
    checkArray(raw, where, opts);
    const res = [];
    for (const [idx, item] of raw.entries()) {
        res.push(parseActionPhase(item, `${where}[${idx}]`));
    }
    return res;
}

export function parseActionDeriveVarTable(
    raw: Readonly<Partial<Record<VirtualKey, ID | VarValueArgs<ID>>>>,
    where: string,
): Readonly<Partial<Record<VirtualKey, ID | Var<ID>>>> {
    if (typeof raw !== 'object' || raw === null) {
        throw new Error(`${where}: must be a object`);
    }
    const res: Partial<Record<VirtualKey, ID | Var<ID>>> = {};
    for (const [raw_key, raw_level] of Object.entries(raw)) {
        const key = parseVirtualKey(raw_key, `${where}[${raw_key}]`);
        res[key] = parseVarID(raw_level, 'Action', `${where}[${raw_key}]`);
    }
    return res;
}

export function verifyActionDeriveVarTable(
    derives: Readonly<Partial<Record<VirtualKey, ID | Var<ID>>>>,
    consumers: {
        character?: ID;
        styles?: ReadonlyArray<ID>;
    },
    where: string,
): void {
    for (const derive of Object.values(derives)) {
        verifyVarValue(derive, consumers, where, (id: ID, where: string) => {
            Action.find(id, where);
        });
    }
}

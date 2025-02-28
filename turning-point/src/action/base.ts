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

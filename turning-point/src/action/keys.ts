import { checkArray, float, ID, parseArray, parseFloat } from '../common';
import { parseVarID, Var, VarValueArgs, verifyVarValue } from '../variable';
import { Action } from './base';

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

    // Just判定
    'JustAttack1',
    'JustAttack2',
    'JustSpell',
    'JustShot1',
    'JustSwitch',
    'JustDerive1',
    'JustDerive2',
    'JustDerive3',

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

const DERIVE_KEYS: ReadonlyArray<string> = [Derive1, Derive2, Derive3];

export const JustAttack1 = 'JustAttack1' as const;
export const JustAttack2 = 'JustAttack2' as const;
export const JustSpell = 'JustSpell' as const;
export const JustShot1 = 'JustShot1' as const;
export const JustSwitch = 'JustSwitch' as const;
export const JustDerive1 = 'JustDerive1' as const;
export const JustDerive2 = 'JustDerive2' as const;
export const JustDerive3 = 'JustDerive3' as const;

const JUST_KEYS: ReadonlyArray<string> = [
    JustAttack1,
    JustAttack2,
    JustSpell,
    JustShot1,
    JustSwitch,
    JustDerive1,
    JustDerive2,
    JustDerive3,
];

export const Item1 = 'Item1' as const;
export const Item2 = 'Item2' as const;
export const Item3 = 'Item3' as const;
export const Item4 = 'Item4' as const;
export const Item5 = 'Item5' as const;
export const Item6 = 'Item6' as const;
export const Item7 = 'Item7' as const;
export const Item8 = 'Item8' as const;

const ITEM_KEYS: ReadonlyArray<string> = [Item1, Item2, Item3, Item4, Item5, Item6, Item7, Item8];

export const Idle = 'Idle' as const;
export const Walk = 'Walk' as const;
export const Run = 'Run' as const;
export const Dash = 'Dash' as const;
export const Break1 = 'Break1' as const;
export const Break2 = 'Break2' as const;
export const Break3 = 'Break3' as const;

const EVENT_KEYS: ReadonlyArray<string> = [Idle, Walk, Run, Dash, Break1, Break2, Break3];

export function isVirtualKey(raw: string): raw is VirtualKey {
    return VIRTUAL_KEYS.includes(raw as VirtualKey);
}

export function parseVirtualKey(
    raw: string,
    where: string,
    opts: {
        derive?: boolean;
        just?: boolean;
        item?: boolean;
        event?: boolean;
    } = {},
): VirtualKey {
    if (!VIRTUAL_KEYS.includes(raw as VirtualKey)) {
        throw new Error(where + ': must be a VirtualKey');
    }
    if (!opts.derive && DERIVE_KEYS.includes(raw)) {
        throw new Error(where + ': derive key not supported');
    }
    if (!opts.just && JUST_KEYS.includes(raw)) {
        throw new Error(where + ': just key not supported');
    }
    if (!opts.item && ITEM_KEYS.includes(raw)) {
        throw new Error(where + ': item key not supported');
    }
    if (!opts.event && EVENT_KEYS.includes(raw)) {
        throw new Error(where + ': event key not supported');
    }
    return raw as VirtualKey;
}

export type VirtualDirArgs = [
    /** 方向 */
    VirtualDirType,

    /** 角度范围 0-180 */
    float,
];

export const VIRTUAL_DIR_TYPES = ['Forward', 'Backward', 'Left', 'Right'] as const;
type VirtualDirType = (typeof VIRTUAL_DIR_TYPES)[number];

export const Forward: VirtualDirType = 'Forward' as const;
export const Backward: VirtualDirType = 'Backward' as const;
export const Left: VirtualDirType = 'Left' as const;
export const Right: VirtualDirType = 'Right' as const;

export class VirtualDir {
    /** 方向 */
    public readonly dir: VirtualDirType;

    /** 角度的cos值 [-1, 1] */
    public readonly cos: float;

    public constructor(args: VirtualDirArgs, where: string) {
        checkArray(args, where, { len: 2 });
        if (!VIRTUAL_DIR_TYPES.includes(args[0] as VirtualDirType)) {
            throw new Error(where + ': must be a InsertKey');
        }
        this.dir = args[0] as VirtualDirType;
        const angle = parseFloat(args[1], `${where}[1]`, { min: 0, max: 180 });
        this.cos = Math.cos((angle * Math.PI) / 180);
    }
}

export type VirtualKeyDirArgs = VirtualKey | [VirtualKey, VirtualDirArgs];

export class VirtualKeyDir {
    public readonly key: VirtualKey;
    public readonly dir?: VirtualDir;

    public constructor(args: VirtualKeyDirArgs, where: string) {
        if (Array.isArray(args)) {
            checkArray(args, where, { len: 2 });
            this.key = parseVirtualKey(args[0], `${where}[0]`);
            this.dir = new VirtualDir(args[1], `${where}[1]`);
        } else {
            this.key = parseVirtualKey(args, where);
        }
    }

    public toJSON() {
        return [this.key, this.dir || null];
    }
}

export type DeriveRuleArgs =
    | [VirtualKey, ID | VarValueArgs<ID>]
    | [VirtualKey, VirtualDirArgs, ID | VarValueArgs<ID>];

export class DeriveRule {
    public readonly key: VirtualKey;
    public readonly dir?: VirtualDir;
    public readonly action: ID | Var<ID>;

    public constructor(args: DeriveRuleArgs, where: string) {
        checkArray(args, where, { min_len: 2, max_len: 3 });
        if (args.length === 2) {
            this.key = parseVirtualKey(args[0], `${where}[0]`);
            this.action = parseVarID(args[1], 'Action', `${where}[1]`);
        } else {
            this.key = parseVirtualKey(args[0], `${where}[0]`);
            this.dir = new VirtualDir(args[1], `${where}[1]`);
            this.action = parseVarID(args[2], 'Action', `${where}[2]`);
        }
    }
}

export function parseDeriveRuleArray(
    raw: ReadonlyArray<DeriveRuleArgs>,
    where: string,
): ReadonlyArray<DeriveRule> {
    return parseArray(
        raw,
        where,
        (item: DeriveRuleArgs, where: string) => new DeriveRule(item, where),
    );
}

export function verifyDeriveRuleArray(
    derives: ReadonlyArray<DeriveRule>,
    consumers: {
        character?: ID;
        styles?: ReadonlyArray<ID>;
    },
    where: string,
): void {
    for (const derive of Object.values(derives)) {
        verifyVarValue(derive.action, consumers, where, (id: ID, where: string) => {
            Action.find(id, where);
        });
    }
}

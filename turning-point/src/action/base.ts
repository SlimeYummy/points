import {
    float,
    ID,
    IDPrefix,
    int,
    parseArray,
    parseID,
    parseIDArray,
    parseInt,
    parseIntArray,
} from '../common';
import { Resource } from '../resource';
import { Character, Style } from '../character';
import {
    parseVarBool,
    parseVarFloat,
    parseVarInt,
    parseVarValueArgs,
    Var,
    VarValueArgs,
    verifyVarValue,
} from '../variable';

/* 
动作(Action)的配置非常复杂，需要与编辑器配合使用。
一般，涉及时间、判定帧、动画的配置，需要在编辑器中完成。
涉及派生、伤害、数值、buff的配置，放在脚本文件中完成。
下面是一些重要概念的讲解：


## 进入动作
在游戏内，任何时候，角色必定处于一个动作中。
当玩家按下按键，或角色接收到事件(如受击、遇敌等)，角色会进入新的动作。

涉及参数：
- enter_key: 进入按键，新动作的属性，每个动作固定一个，全局生效。
- enter_level: 进入等级，新动作的属性，越大越容易进入新动作。
- derive_level: 派生等级，当前动作的属性，越大越不容易离开当前动作。
- derives: 派生动作列表，当前动作的属性，可以理解为仅在当前动作下生效的enter_key列表。
简化的逻辑如下：
玩家按下按键后，先搜索当前动作的derives，再搜索全局所有动作的enter_key。
若找到匹配按键的动作，则比较该动作的enter_level与当前动作的derive_level。
若enter_level>derive_level，则进入该动作。

事件动作进入 ...


## 动作阶段
动作的数值与属性会随动作阶段的变化而变化。主要阶段包含：
- 准备/前摇(prepare)
- 瞄准/吟唱(spell)
- 蓄力(charge)
- 进行中(active)
- 后摇/恢复(recovery)
动作阶段在编辑器内配置。

涉及参数：
- 脚本中使用PhaseXxx系列为不同阶段赋予不同值。


## 消耗


## 打断


## 攻击判定


## 子弹与轨迹


## 特殊：目押(just)派生
一类在极短时机内的特殊判定，可以是动作内的固定时间，后者是某次攻击判定命中后。
目押时机在编辑器内配置。

涉及参数：
- just_derives: 目押派生动作列表，当前动作的属性，优先于derives与enter_key。


## 特殊：插入(insert)派生
有些动作中，使用闪避等动作，可以不中断该动作的派生，继续派生derives中的后续动作。
仅有以下特殊动作可插入派生：
- 闪避
- 完美闪避
- 防御(短按，长时间按住不行)
- 完美防御

涉及参数：
- insert_actions: 允许插入的动作(仅上文4个可选)
- 插入派生的时机等参数在闪避/防御部分统一配置，不可为每个待插入动作单独配置。






动作按键与派生

enter_key:
    动作进入按键
    仅能是PrimaryKey或BuiltinKey。

enter_level:
    动作进入等级
    与*_derive_level配合使用。当enter_level>=*_derive_level时，角色可进入当前动作。

base_derive_level:
    前摇&动作中的派生等级
    与enter_level配合使用，默认LevelDoing，即大部分动作不可派生。

derive_level:
    后摇派生等级
    与enter_level配合使用。

derive_start:
    后摇派生开始时间。
    该值表示动作开始后的时间偏移。

derive_duration:
    后摇派生持续时间。
    默认持续到动作结束，但有时我们希望角色回归自由态后，派生窗口能再持续一段时间。

derives:
    特殊派生动作列表
    此处的派生不参考enter_level/*_derive_level，仅能是DeriveKey。

insertion_enabled:
    启用派生插入

insertion_actions:
    允许派生插入的动作类型
    通常，在防御/闪避后，即使post_derive_duration尚未结束，derives中的派生也会立即失效。
    该字段允许额外插入防御/闪避等动作，不打断派生。

insertion_derive_duration:
    插入派生后再派生时间
    默认为空，表示插入动作后，可派生时间维持post_derive_duration不延长。
    若为秒数，则在插入动作后，刷新可派生时间至该值。

enable_just:
    启用目押判定

just_start:
    Just(目押)判定开始时间。
    若just_hit为空，该值表示动作开始后的时间偏移。
    若just_hit不为空，该值表示对应判定命中后的时间偏移。

just_duration:
    Just(目押)判定持续时间。

just_hit:
    触发just(目押)判定的hit，参考just_time。



消耗与中断

anitbreak_level:
    抗打断等级
    通常，在攻击判定发生的核心阶段，动作抗打断等级需要提升。需配合下面的时间使用。

antibreak_time:
    抗打断开始时间

antibreak_duration:
    抗打断持续时间

base_anitbreak_level:
    基础抗打断等级
    非核心阶段的抗打断等级
*/

export type ActionArgs = {
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
    public static override readonly prefix: IDPrefix = 'Action';

    public static override find(id: string, where: string): Action {
        const res = Resource.find(id, where);
        if (!(res instanceof Action)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 是否启用 */
    public readonly enabled?: boolean | Var<boolean>;

    /** 所属角色ID Action关联的Style应当属于该Character */
    public readonly character: ID;

    /** 可以使用该动作的角色风格 */
    public readonly styles: ReadonlyArray<ID>;

    public constructor(id: ID, args: ActionArgs) {
        super(id);
        this.enabled = parseVarBool(args.enabled || true, this.w('enabled'));
        this.character = parseID(args.character, 'Character', this.w('character'));
        this.styles = parseIDArray(args.styles, 'Style', this.w('styles'));
    }

    public override verify() {
        verifyVarValue(this.enabled, { styles: this.styles }, this.w('enabled'));

        Character.find(this.character, this.w('character'));

        if (this.styles) {
            for (const [idx, id] of this.styles.entries()) {
                const usable_style = Style.find(id, this.w(`styles[${idx}]`));
                if (this.character !== usable_style.character) {
                    throw this.e(`styles[${idx}]`, 'character mismatch with styles');
                }
                if (!usable_style.actions.includes(this.id)) {
                    throw this.e(`styles[${idx}]`, 'Style and Action mismatch');
                }
            }
        }
    }
}

export const DERIVE_CONTINUE = ['Dodge', 'PerfectDodge', 'Guard', 'PerfectGuard'] as const;

export type DeriveContinue = (typeof DERIVE_CONTINUE)[number];

export const PerfectDodge = 'PerfectDodge' as const;
export const PerfectGuard = 'PerfectGuard' as const;

export function isDeriveContinue(raw: string): raw is DeriveContinue {
    return DERIVE_CONTINUE.includes(raw as DeriveContinue);
}

export function parseDeriveContinue(raw: string, where: string): DeriveContinue {
    if (!DERIVE_CONTINUE.includes(raw as DeriveContinue)) {
        throw new Error(where + ': must be a DeriveContinue');
    }
    return raw as DeriveContinue;
}

export function parseDeriveContinueSet(
    raw: ReadonlyArray<string>,
    where: string,
): ReadonlyArray<DeriveContinue> {
    return parseArray(raw, where, parseDeriveContinue);
}

export function parseDeriveContinueSetArray(
    raw: ReadonlyArray<ReadonlyArray<string>>,
    where: string,
    opts: {
        len?: int;
        min_len?: int;
        max_len?: int;
    } = {},
): ReadonlyArray<ReadonlyArray<DeriveContinue>> {
    return parseArray(raw, where, parseDeriveContinueSet, opts);
}

export function parseVarDeriveContinueSet(
    raw: ReadonlyArray<DeriveContinue> | VarValueArgs<ReadonlyArray<DeriveContinue>>,
    where: string,
    opts: {
        must_var?: boolean;
        len?: int;
        min_len?: int;
        max_len?: int;
    } = {},
): ReadonlyArray<DeriveContinue> | Var<ReadonlyArray<DeriveContinue>> {
    return parseVarValueArgs(raw, where, opts, parseDeriveContinueSet, parseDeriveContinueSetArray);
}

/** 空闲态 可派生任何动作的状态 最低的派生/进入等级 */
export const LEVEL_IDLE = 0;

/** 移动态 比空闲态稍高 用于区分Idel/Move的优先级 */
export const LEVEL_MOVE = 50;

/** 攻击派生态 普通攻击动作的后摇阶段 */
export const LEVEL_ATTACK = 100;

/** 技能派生态 普通技能动作的后摇阶段 */
export const LEVEL_SKILL = 200;

/** 派生派生态 用于标记动作的高优先级特殊派生 */
export const LEVEL_DERIVE = 300;

/** 大招派生态 大招动作的后摇阶段 */
export const LEVEL_ULTIMATE = 400;

/** 动作态 所有动作前摇/执行中的状态 */
export const LEVEL_ACTION = 500;

/** 不可中断态 最高等级的状态 无法中断 做特殊使用 */
export const LEVEL_UNBREAKABLE = 600;

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
    } = {},
): ReadonlyArray<int> {
    return parseIntArray(raw, where, {
        ...opts,
        min: Math.max(LEVEL_IDLE, opts.min ?? LEVEL_IDLE),
        max: Math.min(LEVEL_UNBREAKABLE, opts.max ?? LEVEL_UNBREAKABLE),
    });
}

export function parseVarActionLevel(
    raw: int | VarValueArgs<int>,
    where: string,
    opts: {
        min?: int;
        max?: int;
    } = {},
): int | Var<int> {
    return parseVarInt(raw, where, {
        ...opts,
        min: Math.max(LEVEL_IDLE, opts.min ?? LEVEL_IDLE),
        max: Math.min(LEVEL_UNBREAKABLE, opts.max ?? LEVEL_UNBREAKABLE),
    });
}

export function parseVarActionLevelArray(
    raw: ReadonlyArray<int | VarValueArgs<int>>,
    where: string,
    opts: {
        min?: int;
        max?: int;
        len?: int;
        min_len?: int;
        max_len?: int;
    } = {},
): ReadonlyArray<int | Var<int>> {
    return parseArray(raw, where, (item, where) => parseVarActionLevel(item, where, opts), opts);
}

export type ActionAttributesArgs = {
    /** 伤害减免 */
    damage_rdc?: float | string | VarValueArgs<float | string>;

    /** 护盾伤害减免 */
    shield_dmg_rdc?: float | string | VarValueArgs<float | string>;

    /** 韧性等级 */
    poise_level?: int | VarValueArgs<int>;
};

export class ActionAttributes {
    /** 伤害减免 */
    public readonly damage_rdc: float | Var<float>;

    /** 护盾伤害减免 */
    public readonly shield_dmg_rdc: float | Var<float>;

    /** 韧性等级 */
    public readonly poise_level: int | Var<int>;

    public constructor();
    public constructor(args: ActionAttributesArgs, where: string);
    public constructor() {
        if (arguments.length === 0) {
            this.damage_rdc = 0;
            this.shield_dmg_rdc = 0;
            this.poise_level = 0;
        } else {
            const args = arguments[0];
            const where = arguments[1];
            this.damage_rdc = parseVarFloat(args.damage_rdc || 0, `${where}.damage_rdc`);
            this.shield_dmg_rdc = parseVarFloat(
                args.shield_dmg_rdc || 0,
                `${where}.shield_dmg_rdc`,
            );
            this.poise_level = parseVarInt(args.poise_level || 0, `${where}.poise_level`, {
                min: 0,
                max: 4,
            });
        }
    }
}

export function parseActionAttributes(raw: ActionAttributesArgs, where: string): ActionAttributes {
    return new ActionAttributes(raw, where);
}

// export function parseActionAttributesArray(
//     raw: ReadonlyArray<ActionAttributesArgs>,
//     where: string,
//     opts: {
//         len?: int;
//         min_len?: int;
//         max_len?: int;
//     } = {},
// ): ReadonlyArray<ActionAttributes> {
//     return parseArray(raw, where, (item, where) => parseActionAttributes(item, where), opts);
// }

// function

export type ActionHitbox = {
    name: string;
    window: readonly [int | string, int | string];
    joint: string;
    shape: any;
};

import {
    Capsule,
    checkType,
    FilePath,
    float,
    ID,
    IDPrefix,
    int,
    MAX_NAME_LEN,
    parseFile,
    parseFloat,
    parseID,
    parseIDArray,
    parseIntRange,
    parseString,
    parseStringArray,
    parseVec2,
    TaperedCapsule,
} from './common';
import { Resource } from './resource';
import { Action } from './action';
import {
    parseAttributeTable,
    PRIMARY_ATTRIBUTES,
    PrimaryAttribute,
    SECONDARY_ATTRIBUTES,
    SecondaryAttribute,
} from './attribute';
import { Equipment } from './equipment';
import { parseJevelSlotsArray } from './jewel';
import * as native from './native';
import { Perk } from './perk';
import { AiBrain } from './ai';

export type CharacterArgs = {
    /** 角色名字 */
    name: string;

    /** 最大等级 */
    level: readonly [int, int];

    /** 风格ID列表 */
    styles: ReadonlyArray<ID>;

    /** 装备ID列表 */
    equipments: ReadonlyArray<ID>;

    /** 用于移动的包围胶囊体 */
    bounding: Capsule | TaperedCapsule;

    /** 骨骼动画模型文件 一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.ls-ozz 逻辑骨骼
     * - xxx.vs-ozz 视图骨骼
     * - xxx.cp-rkyv/xxx.cp-json 角色物理
     */
    skeleton_files: FilePath;

    /** 模型在XZ平面上的朝向（正面方向） */
    skeleton_toward: readonly [float, float];
};

/**
 * 角色，即游戏中的一个玩家控制的角色。
 * 注意区分Character与Style，一个Character对应多个Style。
 * Character里包含该角色所有Style通用的数据。
 */
export class Character extends Resource {
    public static override readonly prefix: IDPrefix = 'Character';

    public static override find(id: ID, where: string): Character {
        const res = Resource.find(id, where);
        if (!(res instanceof Character)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 角色名字 */
    public readonly name: string;

    /** 最大等级 */
    public readonly level: readonly [int, int];

    /** 风格ID列表 */
    public readonly styles: ReadonlyArray<ID>;

    /** 装备ID列表 */
    public readonly equipments: ReadonlyArray<ID>;

    /** 用于移动的包围胶囊体 */
    public readonly bounding: Capsule | TaperedCapsule;

    /** 骨骼动画模型文件  一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.ls-ozz 逻辑骨骼
     * - xxx.vs-ozz 视图骨骼
     * - xxx.cp-rkyv/xxx.cp-json
     */
    public readonly skeleton_files: FilePath;

    /** 模型在XZ平面上的朝向（正面方向） */
    public readonly skeleton_toward: readonly [float, float];

    public constructor(id: ID, args: CharacterArgs) {
        super(id);
        this.name = parseString(args.name, this.w('name'), { max_len: MAX_NAME_LEN });
        this.level = parseIntRange(args.level, this.w('level'), { min: 0 });
        this.styles = parseIDArray(args.styles, 'Style', this.w('styles'));
        this.equipments = parseIDArray(args.equipments, 'Equipment', this.w('equipments'));
        this.bounding = checkType(args.bounding, [Capsule, TaperedCapsule], this.w('bounding'));
        this.skeleton_files = parseFile(args.skeleton_files, this.w('skeleton_files'), {
            extension: '.*',
        });
        this.skeleton_toward = parseVec2(args.skeleton_toward, this.w('skeleton_toward'), {
            normalized: true,
        });

        this.checkSkeletonFiles();
    }

    private checkSkeletonFiles() {
        if (!native.existCharacterPhysics(this.skeleton_files)) {
            throw this.e('skeleton_files', `file not found (${this.skeleton_files})`);
        }

        native.loadSkeletonMeta(
            this.skeleton_files,
            false,
            `${this.w('skeleton_files')}: file not found (${this.skeleton_files})`,
        );
    }

    public override verify() {
        for (const [idx, style_id] of this.styles.entries()) {
            const style = Style.find(style_id, this.w(`styles[${idx}]`));
            if (style.character !== this.id) {
                throw this.e(`styles[${idx}]`, 'Character and Style mismatch');
            }
        }

        for (const [idx, equip_id] of this.equipments.entries()) {
            const equip = Equipment.find(equip_id, this.w(`equipments[${idx}]`));
            if (equip.character !== this.id) {
                throw this.e(`equipments[${idx}]`, 'Character and Equipment mismatch');
            }
        }
    }
}

export type StyleArgs = {
    /** 风格名字 */
    name: string;

    /** 所属角色ID */
    character: ID;

    /** 标签 */
    tags?: ReadonlyArray<string>;

    /** 每一级的属性 */
    attributes: Readonly<
        Partial<Record<PrimaryAttribute | SecondaryAttribute, ReadonlyArray<float | string>>>
    >;

    /** 每一级的插槽列 */
    slots: ReadonlyArray<string | readonly [int, int, int]>;

    /** 不随等级变动的属性 */
    fixed_attributes: FixedAttributesArgs;

    /** 拥有的Perk列表 */
    perks: ReadonlyArray<ID>;

    /** 可以使用的Perk列表 */
    usable_perks?: ReadonlyArray<ID>;

    /** 可用的动作列表 */
    actions: ReadonlyArray<ID>;

    /** 角色模型（渲染） */
    view_model: FilePath;
};

/**
 * 角色风格，玩家可控角色的不同变体。
 */
export class Style extends Resource {
    public static override readonly prefix: IDPrefix = 'Style';

    public static override find(id: string, where: string): Style {
        const res = Resource.find(id, where);
        if (!(res instanceof Style)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 风格名字 */
    public readonly name: string;

    /** 所属角色ID */
    public readonly character: ID;

    /** 标签 */
    public readonly tags: ReadonlyArray<string>;

    /** 每级的属性列表 */
    public readonly attributes: Readonly<
        Partial<Record<PrimaryAttribute | SecondaryAttribute, ReadonlyArray<float>>>
    >;

    /** 每级的插槽列 */
    public readonly slots: ReadonlyArray<readonly [int, int, int]>;

    /** 不随等级变动的属性 */
    public readonly fixed_attributes: FixedAttributes;

    /** 拥有的Perk列表 即该风格可以点亮的Perk */
    public readonly perks: ReadonlyArray<ID>;

    /** 可以使用的Perk列表 包含了其他由Style点亮 但该Style也可使用的Perk */
    public readonly usable_perks?: ReadonlyArray<ID>;

    /** 可用的动作列表 */
    public readonly actions: ReadonlyArray<ID>;

    /** 角色模型（渲染） */
    public readonly view_model: FilePath;

    public constructor(id: ID, args: StyleArgs) {
        super(id);
        this.name = parseString(args.name, this.w('name'), { max_len: MAX_NAME_LEN });
        this.character = parseID(args.character, 'Character', this.w('character'));
        this.tags = parseStringArray(args.tags || [], this.w('tags'), {
            // includes: ['Player'],
            deduplicate: true,
        });
        this.attributes = parseAttributeTable<PrimaryAttribute | SecondaryAttribute>(
            args.attributes,
            [PRIMARY_ATTRIBUTES, SECONDARY_ATTRIBUTES],
            this.w('attributes'),
        );
        this.slots = parseJevelSlotsArray(args.slots, this.w('slots'));
        this.fixed_attributes = new FixedAttributes(
            args.fixed_attributes,
            this.w('fixed_attributes'),
        );
        this.perks = parseIDArray(args.perks, 'Perk', this.w('perks'));
        this.usable_perks = this.parseUsablePerks(args.usable_perks, args.perks);
        this.actions = parseIDArray(args.actions, 'Action', this.w('actions'));
        this.view_model = parseFile(args.view_model, this.w('view_model'), {
            extension: ['.vrm', '.prefab', '.unity'],
        });
    }

    private parseUsablePerks(
        usable_perks: ReadonlyArray<ID> | undefined,
        perks: ReadonlyArray<ID>,
    ): ReadonlyArray<ID> {
        const all_perks = usable_perks ? [...usable_perks] : [];
        for (const perk of perks) {
            if (!all_perks.includes(perk)) {
                all_perks.push(perk);
            }
        }
        return parseIDArray(all_perks, 'Perk', this.w('usable_perks'));
    }

    public override verify() {
        const character = Character.find(this.character, this.w('character'));
        if (!character.styles.includes(this.id)) {
            throw this.e('character', 'Character and Style mismatch');
        }

        const level_count = character.level[1] - character.level[0] + 1;
        for (const vals of Object.values(this.attributes)) {
            if (vals.length !== level_count) {
                throw this.e('attributes', `len must = ${vals.length}`);
            }
        }

        for (const [idx, perk_id] of this.perks.entries()) {
            const perk = Perk.find(perk_id, this.w(`perks[${idx}]`));
            if (perk.style !== this.id) {
                throw this.e(`perks[${idx}]`, 'Style and Perk mismatch');
            }
        }

        if (this.usable_perks) {
            for (const [idx, perk_id] of this.usable_perks.entries()) {
                const perk = Perk.find(perk_id, this.w(`usable_perks[${idx}]`));
                if (!perk.usable_styles.includes(this.id)) {
                    throw this.e(`usable_perks[${idx}]`, 'Style and Perk mismatch');
                }
            }
        }

        for (const [idx, entry_id] of this.actions.entries()) {
            const action = Action.find(entry_id, this.w(`actions[${idx}]`));
            if (!action.styles?.includes(this.id)) {
                throw this.e(`actions[${idx}]`, 'Style and Action mismatch');
            }
        }
    }
}

export type NpcCharacterArgs = {
    /** 角色名字 */
    name: string;

    /** 标签 */
    tags?: ReadonlyArray<string>;

    /** 最大等级 */
    level: readonly [int, int];

    /** 每一级的属性 */
    attributes: Readonly<
        Partial<Record<PrimaryAttribute | SecondaryAttribute, ReadonlyArray<float | string>>>
    >;

    /** 不随等级变动的属性 */
    fixed_attributes: FixedAttributesArgs;

    /** 可用的动作列表 */
    actions: ReadonlyArray<ID>;

    /** AI执行器列表 */
    ai_executors: ReadonlyArray<ID>;

    /** 用于移动的包围胶囊体 */
    bounding: Capsule | TaperedCapsule;

    /** 骨骼动画模型文件  一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.ls-ozz 逻辑骨骼
     * - xxx.vs-ozz 视图骨骼
     * - xxx.cp-rkyv/xxx.cp-json 角色物理
     */
    skeleton_files: FilePath;

    /** 模型在XZ平面上的朝向（正面方向） */
    skeleton_toward: readonly [float, float];

    /** 角色模型（渲染） */
    view_model: FilePath;
};

/**
 * NPC角色，包括敌人、BOSS、队友等非玩家控制角色。
 */
export class NpcCharacter extends Resource {
    public static override readonly prefix: IDPrefix = 'NpcCharacter';

    public static override find(id: string, where: string): NpcCharacter {
        const res = Resource.find(id, where);
        if (!(res instanceof NpcCharacter)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 角色名字 */
    public readonly name: string;

    /** 标签 */
    public readonly tags: ReadonlyArray<string>;

    /** 最大等级 */
    public readonly level: readonly [int, int];

    /** 每级的属性列表 */
    public readonly attributes: Readonly<
        Partial<Record<PrimaryAttribute | SecondaryAttribute, ReadonlyArray<float>>>
    >;

    /** 不随等级变动的属性 */
    public readonly fixed_attributes: FixedAttributes;

    /** 可用的动作列表 */
    public readonly actions: ReadonlyArray<ID>;

    /** AI执行器列表 */
    public readonly ai_executors: ReadonlyArray<ID>;

    /** 用于移动的包围胶囊体 */
    public readonly bounding: Capsule | TaperedCapsule;

    /** 骨骼动画模型文件  一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.ls-ozz 逻辑骨骼
     * - xxx.vs-ozz 视图骨骼
     * - xxx.cp-rkyv/xxx.cp-json 角色物理
     */
    public readonly skeleton_files: FilePath;

    /** 模型在XZ平面上的朝向（正面方向） */
    public readonly skeleton_toward: readonly [float, float];

    /** 角色模型（渲染） */
    public readonly view_model: FilePath;

    public constructor(id: ID, args: NpcCharacterArgs) {
        super(id);
        this.name = parseString(args.name, this.w('name'), { max_len: MAX_NAME_LEN });
        this.tags = parseStringArray(args.tags || [], this.w('tags'), { deduplicate: true });
        this.level = parseIntRange(args.level, this.w('level'), { min: 0 });
        this.attributes = parseAttributeTable<PrimaryAttribute | SecondaryAttribute>(
            args.attributes,
            [PRIMARY_ATTRIBUTES, SECONDARY_ATTRIBUTES],
            this.w('attributes'),
        );
        this.fixed_attributes = new FixedAttributes(
            args.fixed_attributes,
            this.w('fixed_attributes'),
        );
        this.actions = parseIDArray(args.actions, 'Action', this.w('actions'));
        this.ai_executors = parseIDArray(args.ai_executors, 'AiBrain', this.w('ai_executors'));
        this.bounding = checkType(args.bounding, [Capsule, TaperedCapsule], this.w('bounding'));
        this.skeleton_files = parseFile(args.skeleton_files, this.w('skeleton_files'), {
            extension: '.*',
        });
        this.skeleton_toward = parseVec2(args.skeleton_toward, this.w('skeleton_toward'), {
            normalized: true,
        });
        this.view_model = parseFile(args.view_model, this.w('view_model'), {
            extension: ['.vrm', '.prefab', '.unity'],
        });
    }

    public verify(): void {
        const level_count = this.level[1] - this.level[0] + 1;
        for (const vals of Object.values(this.attributes)) {
            if (vals.length !== level_count) {
                throw this.e('attributes', `len must = ${vals.length}`);
            }
        }

        for (const [idx, entry_id] of this.actions.entries()) {
            const action = Action.find(entry_id, this.w(`actions[${idx}]`));
            if (!action.npc_characters?.includes(this.id)) {
                throw this.e(`actions[${idx}]`, 'NpcCharacter and Action mismatch');
            }
        }

        for (const [idx, entry_id] of this.ai_executors.entries()) {
            const ai_executor = AiBrain.find(entry_id, this.w(`ai_executors[${idx}]`));
            if (ai_executor.character !== this.id) {
                throw this.e(`ai_executors[${idx}]`, 'NpcCharacter and AiBrain mismatch');
            }
        }
    }
}

export type FixedAttributesArgs = {
    /** 常规状态伤害减免 P1 */
    damage_reduce_param_1: float | string;

    /** 常规状态伤害减免 P2 */
    damage_reduce_param_2: float | string;

    /** 防御状态下伤害减免率 */
    guard_damage_ratio_1: float | string;

    /** 常规状态架势伤害减免 P1 */
    deposture_reduce_param_1: float | string;

    /** 常规状态架势伤害减免 P2 */
    deposture_reduce_param_2: float | string;

    /** 防御状态下架势伤害减免率 */
    guard_deposture_ratio_1: float | string;

    /** 对虚弱状态下的敌人增伤 */
    weak_damage_up: float | string;
};

export class FixedAttributes {
    // 常规状态伤害减免的公式
    // P1 + (1 - P1) * defense / (P2 + defense)

    /** 常规状态伤害减免 P1 */
    public readonly damage_reduce_param_1: float;

    /** 常规状态伤害减免 P2 */
    public readonly damage_reduce_param_2: float;

    /** 防御状态下伤害减免率 */
    public readonly guard_damage_ratio_1: float;

    // 常规状态架势伤害减免的公式
    // P1 + (1 - P1) * defense / (P2 + defense)

    /** 常规状态架势伤害减免 P1 */
    public readonly deposture_reduce_param_1: float;

    /** 常规状态架势伤害减免 P2 */
    public readonly deposture_reduce_param_2: float;

    /** 防御状态下架势伤害减免率 */
    public readonly guard_deposture_ratio_1: float;

    /** 对虚弱状态下的敌人增伤 */
    public readonly weak_damage_up: float;

    public constructor(args: FixedAttributesArgs, where: string) {
        this.damage_reduce_param_1 = parseFloat(
            args.damage_reduce_param_1,
            `${where}.damage_reduce_param_1`,
            { min: 0, max: 1 },
        );
        this.damage_reduce_param_2 = parseFloat(
            args.damage_reduce_param_2,
            `${where}.damage_reduce_param_2`,
            { min: 0 },
        );
        this.guard_damage_ratio_1 = parseFloat(
            args.guard_damage_ratio_1,
            `${where}.guard_damage_ratio_1`,
            { min: 0, max: 1 },
        );
        this.deposture_reduce_param_1 = parseFloat(
            args.deposture_reduce_param_1,
            `${where}.deposture_reduce_param_1`,
            { min: 0, max: 1 },
        );
        this.deposture_reduce_param_2 = parseFloat(
            args.deposture_reduce_param_2,
            `${where}.deposture_reduce_param_2`,
            { min: 0 },
        );
        this.guard_deposture_ratio_1 = parseFloat(
            args.guard_deposture_ratio_1,
            `${where}.guard_deposture_ratio_1`,
            { min: 0, max: 1 },
        );
        this.weak_damage_up = parseFloat(args.weak_damage_up, `${where}.weak_damage_up`, {
            min: 0,
        });
    }
}

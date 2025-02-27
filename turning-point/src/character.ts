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
} from './common';
import { Resource } from './resource';
import {
    parseAttributeTable,
    PRIMARY_ATTRIBUTES,
    PrimaryAttribute,
    SECONDARY_ATTRIBUTES,
    SecondaryAttribute,
} from './attribute';
import { Equipment } from './equipment';
import { parseJevelSlotsArray } from './jewel';

export type CharacterArgs = {
    /** 角色名字 */
    name: string;

    /** 最大等级 */
    level: [int, int];

    /** 风格ID列表 */
    styles: ReadonlyArray<ID>;

    /** 装备ID列表 */
    equipments: ReadonlyArray<ID>;

    /** 用于移动的包围胶囊体 */
    bounding_capsule: Capsule;

    /** 用于骨骼动画的模型文件(ozz) */
    skeleton: FilePath;

    /** 绑定在骨骼动画上的受击包围盒配置文件 */
    target_box: FilePath;
};

/**
 * 角色，即游戏中的一个角色，如LK/LL/WQ/YJ等。
 * 注意区分Character与Style，一个Character对应多个Style。
 * Character里包含该角色所有Style通用的数据。
 */
export class Character extends Resource {
    public static override prefix: IDPrefix = 'Character';

    public static override find(id: string, where: string): Character {
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
    public readonly bounding_capsule: Capsule;

    /** 用于骨骼动画的模型文件(ozz) */
    public readonly skeleton: FilePath;

    /** 绑定在骨骼动画上的受击包围盒配置文件 */
    public readonly target_box: FilePath;

    public constructor(id: ID, args: CharacterArgs) {
        super(id);
        this.name = parseString(args.name, this.w('name'), { max_len: 32 });
        this.level = parseIntRange(args.level, this.w('level'), { min: 0 });
        this.styles = parseIDArray(args.styles, 'Style', this.w('styles'));
        this.equipments = parseIDArray(args.equipments, 'Equipment', this.w('equipments'));
        this.bounding_capsule = checkType(
            args.bounding_capsule,
            Capsule,
            this.w('bounding_capsule'),
        );
        this.skeleton = parseFile(args.skeleton, this.w('skeleton'), { extension: '.ozz' });
        this.target_box = parseFile(args.target_box, this.w('target_box'));
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

export class Style extends Resource {
    public static override prefix: IDPrefix = 'Style';

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
        this.attributes = parseAttributeTable<PrimaryAttribute | SecondaryAttribute>(
            args.attributes,
            [PRIMARY_ATTRIBUTES, SECONDARY_ATTRIBUTES],
            this.w('attributes'),
        );
        this.slots = parseJevelSlotsArray(args.slots, this.w('slots'));
        this.fixed_attributes = parseFixedAttributes(
            args.fixed_attributes,
            this.w('fixed_attributes'),
        );
        this.perks = parseIDArray(args.perks, 'Perk', this.w('perks'));
        this.usable_perks = !args.usable_perks
            ? undefined
            : parseIDArray(args.usable_perks, 'Perk', this.w('usable_perks'));
        this.actions = parseIDArray(args.actions, 'Action', this.w('actions'));
        this.view_model = parseFile(args.view_model, this.w('view_model'), { extension: '.vrm' });
    }

    public override verify() {
        const character = Character.find(this.character, this.w('character'));
        if (!character.styles.includes(this.id)) {
            throw this.e('character', 'Character and Style mismatch');
        }
        const levels = character.level[1] - character.level[0] + 1;

        for (const [attr, attributes] of Object.entries(this.attributes)) {
            if (attributes.length !== levels) {
                throw this.e(`attributes[${attr}]`, `len must = ${levels}`);
            }
        }

        if (this.slots.length !== levels) {
            throw this.e('slots', `len must = ${levels}`);
        }

        // for (const [idx, prek] of this.perks.entries()) {
        // }

        // for (const [idx, prek] of this.usable_perks.entries()) {
        // }

        // for (const [idx, prek] of this.actions.entries()) {
        // }
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

export function parseFixedAttributes(args: FixedAttributesArgs, where: string): FixedAttributes {
    return new FixedAttributes(args, where);
}

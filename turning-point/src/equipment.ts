import {
    checkArray,
    checkRecord,
    float,
    ID,
    IDPrefix,
    int,
    MAX_NAME_LEN,
    parseArray,
    parseID,
    parseInt,
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
import { Character } from './character';
import { parseEntryTable, verifyEntryTable } from './entry';
import { parseJevelSlotsArray } from './jewel';

const EQUIPMENT_SLOTS = ['Slot1', 'Slot2', 'Slot3'] as const;

export const Slot1 = 'Slot1';
export const Slot2 = 'Slot2';
export const Slot3 = 'Slot3';

export type EquipmentSlot = (typeof EQUIPMENT_SLOTS)[number];

export function isEquipmentSlot(raw: string): raw is EquipmentSlot {
    return EQUIPMENT_SLOTS.includes(raw as EquipmentSlot);
}

export function parseEquipmentSlot(raw: string, where: string): EquipmentSlot {
    if (!EQUIPMENT_SLOTS.includes(raw as EquipmentSlot)) {
        throw new Error(`${where}: must be an EquipmentSlot`);
    }
    return raw as EquipmentSlot;
}

export type EquipmentArgs = {
    /** 展示用的名字 */
    name: string;

    /** 所属角色ID */
    character: ID;

    /** 该装备能用于哪个装备槽 */
    slot: EquipmentSlot;

    /** 装备树中的父节点 (父装备ID => 等级) */
    parents?: Readonly<Record<ID, int>>;

    /** 等级范围 */
    level: readonly [int, int];

    /** 每一级的武器强化素材 */
    materials?: ReadonlyArray<ReadonlyArray<[ID, int]>>;

    /** 每级的属性列表 */
    attributes: Readonly<
        Partial<Record<PrimaryAttribute | SecondaryAttribute, ReadonlyArray<float | string>>>
    >;

    /** 每级的插槽列 */
    slots?: ReadonlyArray<string | readonly [int, int, int]>;

    /** 每级的词条 */
    entries?: Readonly<Record<ID, ReadonlyArray<readonly [int, int]>>>;
};

/**
 * 武器&装备
 *
 * 每角色3个装备槽 原则上分主武器/副武器/防具等部位 也可以根据角色调整
 * 不同部位差异体现在数值上 武器加攻击属性 防具加防御属性
 *
 * 装备采用类似怪猎的派生树机制 消耗素材制作
 * 装备生产出来后 派生树上的对应节点被激活 即使后续装备升级 被激活装备也将一直可用
 *
 * 装备区分等级 原则上同等级装备性能接近 方便将等级作为衡量强弱的标准
 */
export class Equipment extends Resource {
    public static override readonly prefix: IDPrefix = 'Equipment';

    public static override find(id: string, where: string): Equipment {
        const res = Resource.find(id, where);
        if (!(res instanceof Equipment)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 展示用的名字 */
    public readonly name: string;

    /** 所属角色ID */
    public readonly character: ID;

    /** 该装备能用于哪个装备槽 */
    public readonly slot: EquipmentSlot;

    /** 等级范围 */
    public readonly level: readonly [int, int];

    /** 装备树中的父节点 (父装备ID => 等级) */
    public readonly parents?: Readonly<Record<ID, int>>;

    // /** 每一级的武器强化素材 */
    // public readonly materials?: ReadonlyArray<ReadonlyArray<[ID, int]>>;

    /** 每级的属性列表 */
    public readonly attributes: Readonly<
        Partial<Record<PrimaryAttribute | SecondaryAttribute, ReadonlyArray<float>>>
    >;

    /** 每级的插槽列 */
    public readonly slots?: ReadonlyArray<readonly [int, int, int]>;

    /** 每级的词条 */
    public readonly entries?: Readonly<Record<ID, ReadonlyArray<readonly [int, int]>>>;

    public constructor(id: ID, args: EquipmentArgs) {
        super(id);
        this.name = parseString(args.name, this.w('name'), { max_len: MAX_NAME_LEN });
        this.character = parseID(args.character, 'Character', this.w('character'));
        this.slot = parseEquipmentSlot(args.slot, this.w('slot'));
        this.level = parseIntRange(args.level, this.w('level'), { min: 0 });
        const levels = this.level[1] - this.level[0] + 1;
        this.parents = this.parseParents(args.parents);
        // this.materials = this.parseMaterials(args.materials, levels);
        this.attributes = parseAttributeTable(
            args.attributes,
            [PRIMARY_ATTRIBUTES, SECONDARY_ATTRIBUTES],
            this.w('attributes'),
            { len: levels },
        );
        this.slots = !args.slots
            ? undefined
            : parseJevelSlotsArray(args.slots, this.w('slots'), { len: levels });
        this.entries = !args.entries
            ? undefined
            : parseEntryTable(args.entries, this.w('entries'), { len: levels });
    }

    private parseParents(
        parents: Readonly<Record<ID, int>> | undefined,
    ): Readonly<Record<ID, int>> | undefined {
        if (parents == null) {
            return undefined;
        }
        checkRecord(parents, this.where('parents'));

        const res: Record<ID, int> = {};
        for (const [pid, level] of Object.entries(parents)) {
            const res_pid = parseID(pid, 'Equipment', this.w(`parents[${pid}]`));
            res[res_pid] = parseInt(level, this.w(`parents[${pid}]`), { min: 0 });
        }
        return res;
    }

    private parseMaterials(
        materials: ReadonlyArray<ReadonlyArray<[ID, int]>> | undefined,
        len: int,
    ): ReadonlyArray<ReadonlyArray<[ID, int]>> | undefined {
        if (materials == null) {
            return undefined;
        }
        return parseArray(
            materials,
            this.w('materials'),
            (vals, where) => {
                return parseArray(vals, where, (tuple, where) => {
                    const [id, cnt] = tuple;
                    checkArray(tuple, where, { len: 2 });
                    return [
                        parseID(id, 'Material', `${where}[0]`),
                        parseInt(cnt, `${where}[1]`, { min: 0 }),
                    ];
                });
            },
            { len },
        );
    }

    public override verify() {
        const character = Character.find(this.character, this.w('character'));
        if (!character.equipments.includes(this.id)) {
            throw this.e('character', 'Character and Equipment mismatch');
        }

        if (this.parents) {
            for (const [pid, level] of Object.entries(this.parents)) {
                const parent = Equipment.find(pid, this.w(`parents[${pid}]`));
                if (this.character !== parent.character) {
                    throw this.e(`parents[${pid}]`, 'Character mismatch with parent');
                }
                if (this.slot !== parent.slot) {
                    throw this.e(`parents[${pid}]`, 'slot missmatch with parent');
                }
                if (level < parent.level[0] || level > parent.level[1]) {
                    throw this.e(`parents[${pid}]`, "out of parent's level range");
                }
            }
        }

        // if (this.materials) {
        //     for (const [i, vals] of this.materials.entries()) {
        //         for (const [j, val] of vals.entries()) {
        //             Resource.find(val[0], this.w(`materials[${i}][${j}]`));
        //         }
        //     }
        // }

        if (this.entries) {
            verifyEntryTable(this.entries, this.w('entries'));
        }
    }
}

import {
    checkArray,
    checkRecord,
    float,
    ID,
    IDPrefix,
    int,
    MAX_ENTRY_PLUS,
    MAX_NAME_LEN,
    parseID,
    parseInt,
    parseString,
} from './common';
import { Resource } from './resource';
import {
    parseAttributePlusTable,
    PRIMARY_ATTRIBUTES,
    PrimaryAttribute,
    SECONDARY_ATTRIBUTES,
    SECONDARY_PLUS_ATTRIBUTES,
    SecondaryAttribute,
    SecondaryPlusAttribute,
} from './attribute';
import { parseVarIndexPlusTable, verifyVarIndexTable } from './variable';

export type EntryArgs = {
    /** 展示用的名字 */
    name: string;

    /** 词条的叠加上限 攻击7 生命3 等等 */
    max_piece: int;

    /**
     * 同一词条叠加带来的提升 List长度必须等于max_piece
     * 对于(+/$)带来的提升 累计MAX_ENTRY_PLUS个(+/$)提升一次 共max_piece次
     */
    attributes?: Readonly<
        Partial<
            Record<
                PrimaryAttribute | SecondaryAttribute | SecondaryPlusAttribute,
                ReadonlyArray<float | string>
            >
        >
    >;

    /** 每一级的变量 */
    var_indexes?: Readonly<Record<ID, ReadonlyArray<int | boolean>>>;
};

/**
 * 词条 装备/饰品/宝石上需要凑够数量发动效果的技能
 */
export class Entry extends Resource {
    public static override prefix: IDPrefix = 'Entry';

    public static override find(id: string, where: string): Entry {
        const res = Resource.find(id, where);
        if (!(res instanceof Entry)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 展示用的名字 */
    public readonly name: string;

    /** 词条的叠加上限 攻击7 生命3 等等 */
    public readonly max_piece: int;

    /**
     * 同一词条叠加带来的提升 List长度必须等于max_piece
     */
    public readonly attributes?: Readonly<
        Partial<Record<PrimaryAttribute | SecondaryAttribute, ReadonlyArray<float>>>
    >;

    /**
     * 同一词条(+/$)叠加带来的提升 List长度必须等于max_piece
     * 累计MAX_ENTRY_PLUS个(+/$)提升一次 共max_piece次
     */
    public readonly attributes_plus?: Readonly<
        Partial<Record<SecondaryAttribute, ReadonlyArray<float>>>
    >;

    /** 每一级的变量(等级) */
    public readonly var_indexes?: Readonly<Record<ID, ReadonlyArray<int>>>;

    /** 每一级的变量(等级) */
    public readonly var_indexes_plus?: Readonly<Record<ID, ReadonlyArray<int>>>;

    // /** 脚本 */
    // script?: string;

    // /**
    //  * 脚本参数
    //  * 接受形如以下的参数:
    //  * - xxx
    //  * - xxx + Plus
    //  * - "xxx+"
    //  * 其中「+ Plus」表示「+」值堆叠带来的提升
    //  */
    // script_args?: EntryScriptArgs;

    public constructor(id: ID, args: EntryArgs) {
        super(id);
        this.name = parseString(args.name, this.w('name'), { max_len: MAX_NAME_LEN });
        this.max_piece = parseInt(args.max_piece, this.w('max_piece'), { min: 1 });
        [this.attributes, this.attributes_plus] = !args.attributes
            ? [undefined, undefined]
            : parseAttributePlusTable(
                  args.attributes,
                  [PRIMARY_ATTRIBUTES, SECONDARY_ATTRIBUTES, SECONDARY_PLUS_ATTRIBUTES],
                  this.w('attributes'),
                  { len: this.max_piece, add_first: 0 },
              );

        [this.var_indexes, this.var_indexes_plus] = !args.var_indexes
            ? [undefined, undefined]
            : parseVarIndexPlusTable(args.var_indexes, this.w('var_indexes'), {
                  len: this.max_piece,
              });
    }

    public override verify() {
        if (this.var_indexes) {
            verifyVarIndexTable(this.var_indexes, {}, this.w('var_indexes'));
        }
        if (this.var_indexes_plus) {
            verifyVarIndexTable(this.var_indexes_plus, {}, this.w('var_indexes_plus'));
        }
    }
}

export function parseEntryTable(
    entries: Readonly<Record<ID, ReadonlyArray<readonly [int, int]>>>,
    where: string,
    opts: { len?: int } = {},
): Readonly<Record<ID, ReadonlyArray<readonly [int, int]>>> {
    checkRecord(entries, where);

    const res: Record<ID, Array<[int, int]>> = {};
    for (const [id, vals] of Object.entries(entries)) {
        const resId = parseID(id, 'Entry', `${where}[${id}]`);
        checkArray(vals, `${where}[${id}]`, opts);
        res[resId] = vals.map((tuple, idx) => {
            const [piece, plus] = tuple;
            checkArray(tuple, `${where}[${id}][${idx}]`, { len: 2 });
            if (plus > piece * MAX_ENTRY_PLUS) {
                throw new Error(`${where}[${id}][${idx}]: [1] must <= [0] * {MAX_ENTRY_PLUS}`);
            }
            return [
                parseInt(piece, `${where}[${id}][${idx}][0]`, { min: 0 }),
                parseInt(plus, `${where}[${id}][${idx}][1]`, { min: 0 }),
            ];
        });
    }
    return res;
}

export function verifyEntryTable(
    entries: Readonly<Record<ID, ReadonlyArray<readonly [int, int]>>>,
    where: string,
) {
    for (const [id, vals] of Object.entries(entries)) {
        const entry = Entry.find(id, `${where}[${id}]`);
        for (const [idx, [piece, plus]] of vals.entries()) {
            if (piece > entry.max_piece) {
                throw new Error(`${where}[${id}][${idx}]: [0] must <= entry.max_piece`);
            }
            if (plus > piece * MAX_ENTRY_PLUS) {
                throw new Error(`${where}[${id}][${idx}]: [1] must <= [0] * {MAX_ENTRY_PLUS}`);
            }
        }
    }
}

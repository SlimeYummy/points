import {
    checkRecord,
    float,
    ID,
    IDPrefix,
    int,
    MAX_NAME_LEN,
    parseID,
    parseIDArray,
    parseInt,
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
import { Character, Style } from './character';
import { parseEntryTable, verifyEntryTable } from './entry';
import { parseJevelSlotsArray } from './jewel';
import { parseVarIndexTable, verifyVarIndexTable } from './variable';

export type PerkArgs = {
    /** 天赋点名字 */
    name: string;

    /** 所属角色ID Perk关联的Style应当属于该Character */
    character: ID;

    /** 所属的风格 拥有该风格时才能点亮天赋 */
    style: ID;

    /** 可以启用该天赋的角色风格 */
    usable_styles?: ReadonlyArray<ID>;

    /** 天赋树中的父节点 {天赋ID: 等级} */
    parents?: Readonly<Record<ID, int>>;

    /** 最高等级 */
    max_level: int;

    /** 每一级的属性列表 */
    attributes?: Readonly<
        Partial<Record<PrimaryAttribute | SecondaryAttribute, ReadonlyArray<float | string>>>
    >;

    /** 每一级的插槽列 */
    slots?: ReadonlyArray<string | readonly [int, int, int]>;

    /** 每一级的词条 */
    entries?: Readonly<Record<ID, ReadonlyArray<readonly [int, int]>>>;

    /** 每一级的变量 */
    var_indexes?: Readonly<Record<ID, ReadonlyArray<int | boolean>>>;
};

/**
 * 天赋点 即天赋树上的天赋加点
 */
export class Perk extends Resource {
    public static override readonly prefix: IDPrefix = 'Perk';

    public static override find(id: string, where: string): Perk {
        const res = Resource.find(id, where);
        if (!(res instanceof Perk)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** 天赋点名字 */
    public readonly name: string;

    /** 所属角色ID Perk关联的Style应当属于该Character */
    public readonly character: ID;

    /** 所属的风格 拥有该风格时才能点亮天赋 */
    public readonly style: ID;

    /** 可以启用该天赋的角色风格 */
    public readonly usable_styles: ReadonlyArray<ID>;

    /** 天赋树中的父节点 {天赋ID: 等级} */
    public readonly parents?: Readonly<Record<ID, int>>;

    /** 最高等级 */
    public readonly max_level: int;

    /** 每一级的属性列表 */
    public readonly attributes?: Readonly<
        Partial<Partial<Record<PrimaryAttribute | SecondaryAttribute, ReadonlyArray<float>>>>
    >;

    /** 每一级的插槽列 */
    public readonly slots?: ReadonlyArray<readonly [int, int, int]>;

    /** 每一级的词条 */
    public readonly entries?: Readonly<Record<ID, ReadonlyArray<readonly [int, int]>>>;

    /** 每一级的变量(等级) */
    public readonly var_indexes?: Readonly<Record<ID, ReadonlyArray<int>>>;

    // /** 脚本 */
    // public readonly script?: string;

    // /** 脚本参数 */
    // public readonly scriptArgs?: Record<string, number | string>;

    public constructor(id: ID, args: PerkArgs) {
        super(id);
        this.name = parseString(args.name, this.w('name'), { max_len: MAX_NAME_LEN });
        this.character = parseID(args.character, 'Character', this.w('character'));
        this.style = parseID(args.style, 'Style', this.w('style'));
        this.usable_styles = this.parseUsableStyles(args.usable_styles, this.style);
        this.parents = this.parseParents(args.parents);
        this.max_level = parseInt(args.max_level, this.w('max_level'), { min: 0 });
        this.attributes = !args.attributes
            ? undefined
            : parseAttributeTable(
                  args.attributes,
                  [PRIMARY_ATTRIBUTES, SECONDARY_ATTRIBUTES],
                  this.w('attributes'),
                  { len: this.max_level },
              );
        this.slots = !args.slots
            ? undefined
            : parseJevelSlotsArray(args.slots, this.w('slots'), { len: this.max_level });
        this.entries = !args.entries
            ? undefined
            : parseEntryTable(args.entries, this.w('entries'), { len: this.max_level });
        this.var_indexes = !args.var_indexes
            ? undefined
            : parseVarIndexTable(args.var_indexes, this.w('var_indexes'), { len: this.max_level });
    }

    private parseUsableStyles(
        usable_styles: ReadonlyArray<ID> | undefined,
        style: ID,
    ): ReadonlyArray<ID> {
        if (!usable_styles) {
            return [style];
        }
        const styles = !usable_styles.includes(style)
            ? [style, ...usable_styles]
            : [...usable_styles];
        return parseIDArray(styles, 'Style', this.w('usable_styles'));
    }

    private parseParents(
        parents: Readonly<Record<ID, int>> | undefined,
    ): Readonly<Record<ID, int>> | undefined {
        if (!parents) {
            return undefined;
        }
        checkRecord(parents, this.w('parents'));

        const res: Record<ID, int> = {};
        for (const [pid, level] of Object.entries(parents)) {
            const res_pid = parseID(pid, 'Perk', this.w(`parents[${pid}]`));
            res[res_pid] = parseInt(level, this.w(`parents[${pid}]`), { min: 0 });
        }
        return res;
    }

    public override verify() {
        Character.find(this.character, this.w('character'));
        const style = Style.find(this.style, this.w('style'));
        if (this.character !== style.character) {
            throw this.e('style', 'Character mismatch with Style');
        }
        if (!style.perks.includes(this.id)) {
            throw this.e('style', 'Style and Perk mismatch');
        }

        if (this.usable_styles) {
            for (const [idx, id] of this.usable_styles.entries()) {
                const usable_style = Style.find(id, this.w(`usable_styles[${idx}]`));
                if (this.character !== usable_style.character) {
                    throw this.e(
                        `usable_styles[${idx}]`,
                        'Character mismatch with Style (usable_styles)',
                    );
                }
                if (!usable_style.usable_perks?.includes(this.id)) {
                    throw this.e(
                        `usable_styles[${idx}]`,
                        'Style and Perk mismatch (usable_styles)',
                    );
                }
            }
        }

        if (this.parents) {
            for (const [pid, level] of Object.entries(this.parents)) {
                const parent = Perk.find(pid, this.w(`parents[${pid}]`));
                if (this.character !== parent.character) {
                    throw this.e(`parents[${pid}]`, 'Character mismatch with parent');
                }
                if (level > parent.max_level) {
                    throw this.e(`parents[${pid}]`, "> parent's max_level");
                }
            }
        }

        if (this.entries) {
            verifyEntryTable(this.entries, this.w('entries'));
        }
        if (this.var_indexes) {
            verifyVarIndexTable(
                this.var_indexes,
                { styles: this.usable_styles },
                this.w('var_indexes'),
            );
        }
    }
}

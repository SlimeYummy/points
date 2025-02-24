import {
    float,
    ID,
    IDPrefix,
    int,
    parseBool,
    parseBoolArray,
    parseFloat,
    parseFloatArray,
    parseID,
    parseIDArray,
    parseInt,
    parseIntArray,
} from './common';
import { Character, Style } from './character';

type VarMeta = {
    readonly id: ID;
    readonly max_level: number;
    readonly no_limit: boolean;
    readonly characters: ReadonlyArray<ID>;
    readonly styles: ReadonlyArray<ID>;
};

export class Var<T> {
    static #metas: Map<ID, Readonly<VarMeta>> = new Map();

    public static define(vars: Readonly<Record<ID, readonly [int, '*' | ID | Array<ID>]>>) {
        for (const [var_id, [max_level, res_ids]] of Object.entries(vars)) {
            if (this.#metas.has(var_id)) {
                throw new Error(`Var.define(${var_id}): define multiple times`);
            }
            const meta = {
                id: parseID(var_id, '#', `Var.define(${var_id}: ...)`),
                max_level: parseInt(max_level, `Var.define(${var_id}: [0])`, { min: 1 }),
                no_limit: false,
                characters: [] as string[],
                styles: [] as string[],
            };

            if (res_ids === '*') {
                meta.no_limit = true;
            } else if (typeof res_ids === 'string') {
                const res_id = res_ids;
                if (res_id.startsWith('Character.')) {
                    meta.characters.push(
                        parseID(res_id, 'Character', `Var.define(${var_id}: [1])`),
                    );
                } else if (res_id.startsWith('Style.')) {
                    meta.styles.push(parseID(res_id, 'Style', `Var.define(${var_id}: [1])`));
                }
            } else if (Array.isArray(res_ids)) {
                for (const [idx, res_id] of res_ids.entries()) {
                    if (res_id.startsWith('Character.')) {
                        meta.characters.push(
                            parseID(res_id, 'Character', `Var.define(${var_id}: [1][${idx}])`),
                        );
                    } else if (res_id.startsWith('Style.')) {
                        meta.styles.push(
                            parseID(res_id, 'Style', `Var.define(${var_id}: [1][${idx}])`),
                        );
                    } else {
                        throw new Error(
                            `Var.define(${var_id}: [1][${idx}]): invalid or unsupported ID`,
                        );
                    }
                }
            } else {
                throw new Error(`Var.define(${var_id}: [1]): must be ID|Array<ID>|*`);
            }

            this.#metas.set(var_id, meta);
        }
    }

    public static find(id: ID, where: string): Readonly<VarMeta> {
        const meta = this.#metas.get(id);
        if (!meta) {
            throw new Error(`${where}: Var "${id}" not found`);
        }
        return meta;
    }

    public readonly id: ID;
    public readonly values: ReadonlyArray<T>;

    private constructor(id: ID, values: ReadonlyArray<T>) {
        this.id = id;
        this.values = values;
    }

    public static new_unchecked<T>(id: ID, values: ReadonlyArray<T>): Var<T> {
        return new Var(id, values);
    }
}

export type VarValueArgs<T> = [ID, Array<T>];

export function parseVarValueArgs<R, T>(
    raw: R | VarValueArgs<R>,
    where: string,
    opts: {
        must_var?: boolean;
        len?: int;
        min_len?: int;
        max_len?: int;
        [key: string]: any;
    } = {},
    parse: (raw: R, where: string, opts: Record<string, any>) => T,
    parseArray: (
        raw: ReadonlyArray<R>,
        where: string,
        opts: Record<string, any>,
    ) => ReadonlyArray<T>,
): T | Var<T> {
    if (Array.isArray(raw) && raw[0].startsWith('#.')) {
        return Var.new_unchecked(
            parseID(raw[0], '#', `${where}[0]`),
            parseArray(raw[1], `${where}.values`, {
                ...opts,
                len: opts.len ? Math.max(opts.len, 2) : undefined,
                min_len: opts.min_len ? Math.max(opts.min_len, 2) : undefined,
                max_len: opts.max_len ? Math.max(opts.max_len, 2) : undefined,
            }),
        );
    } else {
        if (opts.must_var) {
            throw new Error(`${where}: must be a VarValueArgs<R>`);
        }
        return parse(raw as R, where, opts);
    }
}

export function parseVarBool(
    raw: boolean | int | VarValueArgs<boolean | int>,
    where: string,
    opts: {
        must_var?: boolean;
        len?: int;
        min_len?: int;
        max_len?: int;
    } = {},
): boolean | Var<boolean> {
    return parseVarValueArgs(raw, where, opts, parseBool, parseBoolArray);
}

export function parseVarInt(
    raw: int | boolean | VarValueArgs<int | boolean>,
    where: string,
    opts: {
        must_var?: boolean;
        len?: int;
        min_len?: int;
        max_len?: int;
        min?: int;
        max?: int;
        allow_bool?: boolean;
    } = {},
): int | Var<int> {
    return parseVarValueArgs(raw, where, opts, parseInt, parseIntArray);
}

export function parseVarFloat(
    raw: float | string | VarValueArgs<float | string>,
    where: string,
    opts: {
        must_var?: boolean;
        len?: int;
        min_len?: int;
        max_len?: int;
        min?: float;
        max?: float;
    } = {},
): float | Var<float> {
    return parseVarValueArgs(raw, where, opts, parseFloat, parseFloatArray);
}

export function parseVarID(
    raw: ID | VarValueArgs<ID>,
    prefix: IDPrefix,
    where: string,
    opts: {
        len?: number;
        min_len?: number;
        max_len?: number;
    } = {},
) {
    function helper1(raw: ID, where: string) {
        return parseID(raw, prefix, where);
    }
    function helper2(raw: ReadonlyArray<string>, where: string, opts: Record<string, any>) {
        return parseIDArray(raw, prefix, where, { ...opts, allow_conflict: true });
    }
    return parseVarValueArgs(raw, where, opts, helper1, helper2);
}

export function verifyVarValue<T>(
    va: T | Var<T>,
    consumers: {
        character?: ID;
        styles?: ReadonlyArray<ID>;
    },
    where: string,
    callback?: (value: T, where: string) => void,
) {
    if (!(va instanceof Var)) {
        return;
    }

    const meta = Var.find(va.id, where);
    
    if (!meta.no_limit) {
        if (consumers.character) {
            const character = Character.find(consumers.character, where);
            const ok =
                meta.characters.includes(consumers.character) ||
                character.styles.every((style) => meta.styles.includes(style));
            if (!ok) {
                throw new Error(
                    `${where}: ${consumers.character} not defined in ${va.id}`,
                );
            }
        }
    
        if (consumers.styles) {
            for (const style_id of consumers.styles) {
                const style = Style.find(style_id, where);
                const ok = meta.styles.includes(style_id) || meta.characters.includes(style.character);
                if (!ok) {
                    throw new Error(`${where}: ${style_id} not defined in ${va.id}`);
                }
            }
        }
    }

    if (meta.max_level + 1 !== va.values.length) {
        throw new Error(`${where}: ${va.id} must have ${meta.max_level + 1} values`);
    }
    if (callback) {
        for (const [idx, value] of va.values.entries()) {
            callback(value, `${where}][${idx}]`);
        }
    }
}

export function parseVarIndexTable(
    raw: Readonly<Record<ID, ReadonlyArray<int | boolean>>>,
    where: string,
    opts: { len?: int } = {},
): Readonly<Record<ID, ReadonlyArray<int>> | undefined> {
    if (typeof raw !== 'object' || raw === null) {
        throw new Error(`${where}: must be a object`);
    }

    const res_indexes: Record<ID, ReadonlyArray<int>> = {};
    let any_index = false;
    for (const [id, values] of Object.entries(raw)) {
        const res_id = parseID(id, '#', `${where}[${id}]`);
        res_indexes[res_id] = parseIntArray(values, `${where}[${id}]`, { ...opts, min: 0 });
        any_index = true;
    }
    return any_index ? res_indexes : undefined;
}

export function parseVarIndexPlusTable(
    raw: Readonly<Record<ID, ReadonlyArray<int | boolean>>>,
    where: string,
    opts: { len?: int } = {},
): [
    Readonly<Record<ID, ReadonlyArray<int>> | undefined>,
    Readonly<Record<ID, ReadonlyArray<int>> | undefined>,
] {
    if (typeof raw !== 'object' || raw === null) {
        throw new Error(`${where}: must be a object`);
    }

    const res_indexes: Record<ID, ReadonlyArray<int>> = {};
    let any_index = false;
    const res_plus_indexes: Record<ID, ReadonlyArray<int>> = {};
    let any_plus_index = false;

    for (const [id, values] of Object.entries(raw)) {
        if (id.startsWith('$')) {
            const res_id = parseID(id.slice(1), '#', `${where}[${id}]`);
            res_plus_indexes[res_id] = parseIntArray(values, `${where}[${id}]`, {
                ...opts,
                min: 0,
            });
            any_plus_index = true;
        } else {
            const res_id = parseID(id, '#', `${where}[${id}]`);
            res_indexes[res_id] = parseIntArray(values, `${where}[${id}]`, { ...opts, min: 0 });
            any_index = true;
        }
    }

    return [any_index ? res_indexes : undefined, any_plus_index ? res_plus_indexes : undefined];
}

export function verifyVarIndexTable(
    floats: Readonly<Record<ID, ReadonlyArray<int>>>,
    suppliers: {
        character?: ID;
        styles?: ReadonlyArray<ID>;
    },
    where: string,
) {
    for (const id of Object.keys(floats)) {
        const meta = Var.find(id, where);
        if (meta.no_limit) {
            return;
        }

        if (suppliers.character) {
            const character = Character.find(suppliers.character, where);
            const ok =
                meta.characters.includes(suppliers.character) ||
                character.styles.every((style) => meta.styles.includes(style));
            if (!ok) {
                throw new Error(
                    `${where}: ${suppliers.character} not defined in ${id}`,
                );
            }
        }

        if (suppliers.styles) {
            for (const style_id of suppliers.styles) {
                const style = Style.find(style_id, where);
                const ok =
                    meta.styles.includes(style_id) || meta.characters.includes(style.character);
                if (!ok) {
                    throw new Error(`${where}: ${style_id} not defined in ${id}`);
                }
            }
        }
    }
}

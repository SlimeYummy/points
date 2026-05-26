import fs from 'node:fs';
import path from 'node:path';
import { ID, IDPrefix, RE_TMPL_ID_EXTRA } from './common';

export abstract class Resource {
    static #resources = new Map<string, Resource>();
    static #keys = new Set<string>();
    static #done: boolean = false;

    public static readonly prefix?: IDPrefix = undefined;

    public readonly T: string;
    public readonly id: string;

    public abstract verify(): void;

    public static newId(id: string, prefix: string) {
        const match = RE_TMPL_ID_EXTRA.exec(id.slice(prefix.length));
        if (!match) {
            throw new Error(`<${id}>.id: must match ID pattern`);
        }
        for (let idx = 1; idx <= 3; idx += 1) {
            if (match[idx]) {
                Resource.#keys.add(match[idx]!);
            }
        }
    }

    constructor(id: string) {
        if (typeof id !== 'string') {
            throw new Error(`<${id}>.id: must be a string`);
        }
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const prefix: string = (this.constructor as any).prefix;
        if (!prefix) {
            throw new Error('Bad resource prefix');
        }
        if (!id.startsWith(prefix)) {
            throw new Error(`<${id}>.id: must start with "${prefix}"`);
        }

        Resource.newId(id, prefix);

        if (Resource.#resources.has(id)) {
            throw new Error(`<${id}>.id: id cannot repeat`);
        }
        Resource.#resources.set(id, this);
        this.T = this.constructor.name;
        this.id = id;
    }

    protected w(field: string): string {
        return `<${this.id}>.${field}`;
    }

    protected where(field: string): string {
        return `<${this.id}>.${field}`;
    }

    protected e(field: string, message: string): Error {
        return new Error(`<${this.id}>.${field}: ${message}`);
    }

    protected error(field: string, message: string): Error {
        return new Error(`<${this.id}>.${field}: ${message}`);
    }

    public static find(id: string, where: string): Resource {
        if (typeof id !== 'string') {
            throw new Error(`${where}: ResID must be a string`);
        }
        const res = Resource.#resources.get(id);
        if (!res) {
            throw new Error(`${where}: Resource "${id}" not found`);
        }
        return res;
    }

    public static write(folder: string, extra_symbols?: ReadonlyArray<string>) {
        fs.rmSync(folder, { force: true, recursive: true });
        fs.mkdirSync(folder, { recursive: true });

        const keys = Array.from(Resource.#keys);
        keys.sort();
        if (Array.isArray(extra_symbols)) {
            keys.unshift(...extra_symbols);
        }

        const indexes: Record<ID, [number, number]> = {};
        let offset = 3;
        const resources = ['[\r\n'];
        for (const res of Resource.#resources.values()) {
            res.verify();
            const json = JSON.stringify(res);
            indexes[res.id] = [offset, json.length];
            resources.push(json, ',\r\n');
            offset += json.length + 3;
        }
        resources[resources.length - 1] = '\r\n]';

        fs.writeFileSync(path.join(folder, 'key.json'), JSON.stringify(keys));
        fs.writeFileSync(path.join(folder, 'index.json'), JSON.stringify(indexes));
        fs.writeFileSync(path.join(folder, 'data.json'), resources.join(''));

        console.log(`Templates: ${folder}`);
        Resource.#done = true;
    }

    public static done(): boolean {
        return Resource.#done;
    }
}

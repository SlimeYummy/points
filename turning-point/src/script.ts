import cp from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { Resource } from './resource';
import { ID, parseID } from './common/base';

const ID_SPLIT_RE = /[\.\^]/g;

export class Script {
    static #scripts = new Array<Script>();
    static #script_names = new Map<string, number>();

    static readonly funcs: ReadonlyMap<string, Function> = new Map([
        ['AiBrain::execute', Script.#genAiBrainExecute],
    ]);

    public readonly code: string;
    public readonly owner: ID;
    public readonly funcLong: string;
    public readonly funcShort: string;

    constructor(
        code: string | null | undefined,
        owner: ID,
        where: string,
        opts: {
            func: string;
        },
    ) {
        if (code == null) {
            this.code = '';
            this.owner = owner;
            this.funcLong = '';
            this.funcShort = '';
            return;
        }

        if (typeof code !== 'string') {
            throw new Error(`${where}: must be a string`);
        }
        this.code = code;

        this.owner = parseID(owner, ['Character', 'AiBrain'], where);
        const prefix = this.owner.split('.')[0];

        this.funcShort = opts.func;
        this.funcLong = `${prefix}::${opts.func}`;
        if (!Script.funcs.has(this.funcLong)) {
            throw new Error(`${where}: invalid func`);
        }

        Script.#scripts.push(this);
        Script.#script_names.set(this.genFuncName(), Script.#scripts.length - 1);
    }

    public verify(where: string) {
        if (this.code) {
            Resource.find(this.owner, where);
        }
    }

    public toJSON() {
        return !!this.code;
    }

    public genFuncName(): string {
        return `${this.owner.replace(ID_SPLIT_RE, '_')}__${this.funcShort}`;
    }

    public genFuncCode(): string {
        if (!this.code) {
            return '';
        }
        const generator = Script.funcs.get(this.funcLong);
        if (!generator) {
            throw new Error(`Invalid func: ${this.genFuncName()}`);
        }
        return generator(this.owner, this.funcShort, this.genFuncName(), this.code);
    }

    static #genAiBrainExecute(owner: ID, func: string, funcName: string, code: string): string {
        return `
// ${owner} - ${func}
#[unsafe(no_mangle)]
pub extern "C" fn ${funcName}(
    global_ptr: *const WsGameGlobal,
    chara_value_ptr: *const WsCharaValue,
    ai_tasks_ptr: *mut WsAiTask,
    ai_tasks_len: u32
) -> u64 {
    fn ai_brain_execute(global: &WsGameGlobal, value: &WsCharaValue, out: &mut HostBuffer<WsAiTask>) -> Result<()> {
        ${code}
        Ok(())
    }
    wrap_ai_brain_execute(global_ptr, chara_value_ptr, ai_tasks_ptr, ai_tasks_len, ai_brain_execute)
}`;
    }

    public static write(project: string, folder?: string) {
        fs.mkdirSync(path.join(project, 'src'), { recursive: true });

        const codes = [
            `#![allow(unused)]

use critical_point_wasm_types::*;
use glam::*;
use glam_ext::*;`,
            ...Script.#scripts
                .filter((script) => !!script.code)
                .map((script) => script.genFuncCode()),
        ];
        const content = codes.join('\r\n\r\n');

        fs.writeFileSync(path.join(project, 'src', 'auto_gen.rs'), content, 'utf-8');

        const configPath = path.join(project, '.cargo', 'config.toml');
        if (folder && fs.existsSync(configPath)) {
            let config = fs.readFileSync(configPath, 'utf8');
            const keysPath = path.resolve(folder, 'key.json').split(path.sep).join('/');
            config = config.replace(/(TMPL_KEYS_PATH\s*=\s*")[^"]*(")/, `$1${keysPath}$2`);
            fs.writeFileSync(configPath, config, 'utf8');
        }

        console.log(`Scripts: ${project}`);
    }
}

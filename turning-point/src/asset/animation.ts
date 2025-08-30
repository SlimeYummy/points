import artTemplate from 'art-template';
import cp from 'node:child_process';
import fs from 'node:fs';
import process from 'node:process';
import tmp from 'tmp';

tmp.setGracefulCleanup();

function makePrebuiltPath(binary: string) {
    if (process.platform === 'win32' && process.arch === 'x64') {
        return `${__dirname}/../../prebuilt/win32-x64/${binary}.exe`;
    } else if (process.platform === 'linux' && process.arch === 'x64') {
        return `${__dirname}/../../prebuilt/linux-x64/${binary}`;
    } else {
        throw new Error('Unsupported platform');
    }
}

export const GLTF_2_OZZ = makePrebuiltPath('gltf2ozz');

export type MappingPair = {
    logicFile: string;
    viewFile: string;
};

export function gltf2ozz(
    gltfFile: string,
    configFile: string,
    MappingPair: MappingPair,
    dstPattern: string,
) {
    const tmpName = tmp
        .tmpNameSync({
            template: 'gltf2ozz-XXXXXX.ozz',
            tries: 3,
        })
        .replace(/\\/g, '/');

    let config = fs.readFileSync(configFile, 'utf-8');
    config = config.replace(/\{\{(?:CLIPPED_FILE|PREFIX)\}\}/g, (match) => {
        if (match === '{{CLIPPED_FILE}}') {
            // return '../test-tmp/test-asset/girl_clipped.logic-skel.ozz';
            return tmpName;
        } else if (match === '{{PREFIX}}') {
            return dstPattern;
        } else {
            return match;
        }
    });

    const logicCfg = loadConfig(configFile, {
        skeleton_suffix: 'ls',
        animation_suffix: 'la',
        prefix: dstPattern,
        clipped_file: tmpName,
        motion: true,
    });
    execute(gltfFile, logicCfg, MappingPair.logicFile, true);

    const viewCfg = loadConfig(configFile, {
        skeleton_suffix: 'vs',
        animation_suffix: 'va',
        prefix: dstPattern,
        clipped_file: tmpName,
        motion: false,
    });
    execute(gltfFile, viewCfg, MappingPair.viewFile, false);
}

const compiledTmpls: Record<string, (data: any) => string> = {};

function loadConfig(tmplPath: string, data: any) {
    let compiled = compiledTmpls[tmplPath];
    if (!compiled) {
        const file = fs.readFileSync(tmplPath, 'utf-8');
        compiled = artTemplate.compile(file, {
            bail: true,
            escape: false,
            minimize: false,
            cache: false,
        });
        compiledTmpls[tmplPath] = compiled;
    }
    return compiled(data);
}

function execute(gltfFile: string, config: string, mappingFile: string, isLogic: boolean) {
    try {
        cp.execFileSync(
            GLTF_2_OZZ,
            [`--file=${gltfFile}`, `--config=${config}`, `--mapping_file=${mappingFile}`],
            {
                stdio: 'pipe',
                encoding: 'utf8',
                timeout: 60 * 1000,
            },
        );
    } catch (err: any) {
        if (err.stderr) {
            let count = 0;
            console.error();
            console.error(
                err.stderr.replace(/\'byteOffset\' property is missing\.\r?\n?/g, (x: string) =>
                    count++ === 0 ? x : '',
                ),
            );
            const typ = isLogic ? 'logic' : 'view';
            throw new Error(`GLTF to ${typ} ozz failed`);
        }
    }
}

import artTemplate from 'art-template';
import cp from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
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

export type ConfigFiles = {
    configFile: string;
    logicFile: string;
    viewFile: string;
};

export function gltf2ozz(
    gltfFile: string,
    jsonTrackDir: string | null,
    configFiles: ConfigFiles,
    skeletonName: string,
    dstDir: string,
) {
    const tmpName = tmp
        .tmpNameSync({
            template: 'gltf2ozz-XXXXXX.ozz',
            tries: 3,
        })
        .replace(/\\/g, '/');

    const jsonTracks: { json: string; filename: string }[] = [];
    if (jsonTrackDir) {
        if (!fs.existsSync(jsonTrackDir)) {
            throw new Error(`JSON track (${jsonTrackDir}) not found`);
        }
        for (const json of fs.readdirSync(jsonTrackDir)) {
            if (json.endsWith('.wm-json')) {
                jsonTracks.push({
                    json,
                    filename: path.posix.join(dstDir, json.replace('.wm-json', '.wm-ozz')),
                });
            } else if (json.endsWith('.rm-json')) {
                jsonTracks.push({
                    json,
                    filename: path.posix.join(dstDir, json.replace('.rm-json', '.rm-ozz')),
                });
            }
        }
    }

    const logicCfg = loadConfig(configFiles.configFile, {
        skeleton_suffix: 'ls',
        animation_suffix: 'la',
        clipped_file: tmpName,
        root_motion: true,
        json_tracks: jsonTracks,
        skeleton_name: skeletonName,
        out_dir: dstDir,
    });
    execute(gltfFile, logicCfg, configFiles.logicFile, true);

    const viewCfg = loadConfig(configFiles.configFile, {
        skeleton_suffix: 'vs',
        animation_suffix: 'va',
        clipped_file: tmpName,
        root_motion: false,
        json_tracks: [],
        skeleton_name: skeletonName,
        out_dir: dstDir,
    });
    execute(gltfFile, viewCfg, configFiles.viewFile, false);
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

let prebuilt = false;

function execute(gltfFile: string, config: string, mappingFile: string, isLogic: boolean) {
    if (!prebuilt) {
        if (!fs.existsSync(GLTF_2_OZZ)) {
            throw new Error(`${GLTF_2_OZZ} not found`);
        }
        prebuilt = true;
    }
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
        } else {
            throw err;
        }
    }
}

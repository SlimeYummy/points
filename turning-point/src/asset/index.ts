import exitHook from 'exit-hook';
import fs from 'node:fs';
import path from 'node:path/posix';
import { INPUT_ASSET, OUTPUT_ASSET } from '../common';
import * as animation from './animation';

class FileMeta {
    public readonly path: string;
    public readonly time: number;
    public readonly size: number;

    public constructor(file: string);
    public constructor(json: any);
    public constructor() {
        if (typeof arguments[0] === 'string') {
            this.path = path.normalize(arguments[0]);
            const stat = fs.statSync(this.path);
            this.time = stat.mtime.getTime();
            this.size = stat.size;
        } else if (
            typeof arguments[0] === 'object' &&
            typeof arguments[0].path === 'string' &&
            typeof arguments[0].time === 'number' &&
            typeof arguments[0].size === 'number'
        ) {
            this.path = arguments[0].path;
            this.time = arguments[0].time;
            this.size = arguments[0].size;
        } else {
            throw new Error('Invalid arguments');
        }
    }
}

class AssetMeta {
    public readonly id: string;
    public metas: FileMeta[];

    public constructor(id: string, files: string[]);
    public constructor(json: any);
    public constructor() {
        if (
            arguments.length === 2 &&
            typeof arguments[0] === 'string' &&
            Array.isArray(arguments[1])
        ) {
            this.id = arguments[0];
            this.metas = arguments[1].map((x: any) => new FileMeta(x));
        } else if (
            typeof arguments[0] === 'object' &&
            typeof arguments[0].id === 'string' &&
            Array.isArray(arguments[0].metas)
        ) {
            this.id = arguments[0].id;
            this.metas = arguments[0].metas.map((x: any) => new FileMeta(x));
        } else {
            throw new Error('Invalid arguments');
        }
    }

    public check(asset: AssetMeta): boolean {
        let updated = asset.metas.length !== this.metas.length;
        for (const newMeta of asset.metas) {
            const oldMeta = this.metas.find((x) => x.path === newMeta.path);
            updated = !oldMeta || oldMeta.time !== newMeta.time || oldMeta.size !== newMeta.size;
            if (updated) {
                break;
            }
        }
        return updated;
    }
}

class AssetDatabase {
    public readonly dbDir: string;
    public assets: Record<string, AssetMeta> = {};

    public constructor(dbDir: string) {
        if (fs.existsSync(dbDir)) {
            const assets = JSON.parse(fs.readFileSync(dbDir, 'utf-8'));
            if (!Array.isArray(assets)) {
                console.log('Invalid database file');
            } else {
                for (const asset of assets) {
                    const newAsset = new AssetMeta(asset);
                    this.assets[newAsset.id] = newAsset;
                }
            }
        }
        this.dbDir = dbDir;
    }

    public save(dbDir: string) {
        fs.writeFileSync(dbDir, JSON.stringify(Object.values(this.assets)));
    }

    public check(asset: AssetMeta): boolean {
        const oldMeta = this.assets[asset.id];
        if (!oldMeta) {
            return true;
        } else {
            return oldMeta.check(asset);
        }
    }

    public update(asset: AssetMeta) {
        this.assets[asset.id] = asset;
    }
}

export class Asset {
    private constructor() {}

    private static database?: AssetDatabase;

    public static enableIncrement() {
        this.database = new AssetDatabase(`${OUTPUT_ASSET}/.asset_increment.json`);

        exitHook(() => {
            if (this.database) {
                fs.mkdirSync(OUTPUT_ASSET, { recursive: true });
                this.database.save(`${OUTPUT_ASSET}/.asset_increment.json`);
            }
        });
    }

    private static incrementUpdate(id: string, files: string[], updateFn: () => void) {
        if (!this.database) {
            console.log(`Process ${id}`);
            updateFn();
        } else {
            const asset = new AssetMeta(id, files);
            if (this.database.check(asset)) {
                console.log(`Process ${id}`);
                updateFn();
            } else {
                console.log(`Skip ${id}`);
            }
            this.database.update(asset);
        }
    }

    public static copyFiles(
        srcDir: string,
        filter_map: (dir: string, file: string) => string | null | undefined,
    ) {
        const dstDirs = new Set<string>();
        fs.mkdirSync(INPUT_ASSET, { recursive: true });

        function copyFile(src: string, dst: string) {
            const dstDir = path.dirname(dst);
            if (!dstDirs.has(dstDir)) {
                fs.mkdirSync(dstDir, { recursive: true });
            }
            dstDirs.add(dstDir);
            fs.copyFileSync(src, dst);
        }

        function travelDir(dir: string) {
            for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
                if (entry.isDirectory()) {
                    travelDir(path.join(dir, entry.name));
                } else if (entry.isFile()) {
                    let dst = filter_map(dir, entry.name);
                    if (!dst) {
                        continue;
                    }
                    const src = path.join(dir, entry.name);
                    dst = path.join(OUTPUT_ASSET, dst);
                    Asset.incrementUpdate(`copy: ${dst}`, [src], () => copyFile(src, dst));
                }
            }
        }

        travelDir(srcDir);
    }

    public static gltf2ozz(
        srcDir: string,
        dstDir: string,
        files: [string, string, animation.MappingPair, string][],
    ) {
        fs.mkdirSync(path.join(OUTPUT_ASSET, dstDir), { recursive: true });
        for (const [gltf, config, mapping, pattern] of files) {
            const gltfFile = path.join(INPUT_ASSET, srcDir, gltf);
            const configFile = path.join(INPUT_ASSET, srcDir, config);
            const mappingPair = {
                logicFile: path.join(INPUT_ASSET, srcDir, mapping.logicFile),
                viewFile: path.join(INPUT_ASSET, srcDir, mapping.viewFile),
            };
            const dstPattern = path.join(OUTPUT_ASSET, dstDir, pattern);
            Asset.incrementUpdate(
                `gltf: ${dstPattern}`,
                [gltfFile, configFile, mappingPair.logicFile, mappingPair.viewFile],
                () => animation.gltf2ozz(gltfFile, configFile, mappingPair, dstPattern),
            );
        }
    }
}

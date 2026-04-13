import { FilePath, ID, IDPrefix, MAX_NAME_LEN, parseFile, parseString } from './common';
import { Resource } from './resource';

export type ZoneArgs = {
    /** Zone名字 */
    name: string;

    /** Zone文件路径（逻辑） */
    files: FilePath;

    /** Zone文件路径（渲染） */
    view_file: FilePath;
};

/**
 * 区域 游戏中的场景
 */
export class Zone extends Resource {
    public static override readonly prefix: IDPrefix = 'Zone';

    public static override find(id: string, where: string): Zone {
        const res = Resource.find(id, where);
        if (!(res instanceof Zone)) {
            throw new Error(`${where}: Resource type miss match`);
        }
        return res;
    }

    /** Zone名字 */
    public readonly name: string;

    /** Zone文件路径 一个通配的路径前缀 以xxx为例对应如下文件
     * - xxx.zp-rkyv/xxx.zp-json 区域物理(碰撞体)
     * - xxx.nm-bin 寻路数据(NavMesh)
     */
    public readonly files: FilePath;

    /** Zone文件路径（渲染） */
    public readonly view_file: FilePath;

    public constructor(id: ID, args: ZoneArgs) {
        super(id);
        this.name = parseString(args.name, this.w('name'), { max_len: MAX_NAME_LEN });
        this.files = parseFile(args.files, this.w('files'), { extension: '.*' });
        this.view_file = parseFile(args.view_file, this.w('view_file'));
    }

    public override verify() {}
}

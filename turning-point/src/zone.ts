import { FilePath, ID, IDPrefix, MAX_NAME_LEN, parseFile, parseString } from './common';
import { Resource } from './resource';

export type ZoneArgs = {
    /** Zone名字 */
    name: string;

    /** Zone文件路径（逻辑） */
    zone_file: FilePath;

    /** Zone文件路径（渲染） */
    view_zone_file: FilePath;
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

    /** Zone文件路径（逻辑） */
    public readonly zone_file: FilePath;

    /** Zone文件路径（渲染） */
    public readonly view_zone_file: FilePath;

    public constructor(id: ID, args: ZoneArgs) {
        super(id);
        this.name = parseString(args.name, this.w('name'), { max_len: MAX_NAME_LEN });
        this.zone_file = parseFile(args.zone_file, this.w('zone_file'), { extension: '.json' });
        this.view_zone_file = parseFile(args.view_zone_file, this.w('view_zone_file'));
    }

    public override verify() {}
}

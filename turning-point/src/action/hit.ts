import { float, ID, int, parseAngleXz, parseString, parseTime } from '../common';
import { Animation, AnimationArgs } from './animation';
import { Action, ActionArgs } from './base';

export type ActionHitBeHitArgs = AnimationArgs & {
    /** 进入该动画的受击角度 攻击向量与角色正面朝向得夹角（右手系XZ平面） */
    enter_angle: float | string;
};

export class ActionHitBeHit {
    /** Hit动画 */
    public readonly anim: Animation;

    /** 进入该动画的受击角度 攻击向量与角色正面朝向得夹角（右手系XZ平面） */
    public readonly enter_angle: float;

    public constructor(args: ActionHitBeHitArgs, where: string) {
        this.anim = new Animation(args, where, { root_motion: true });
        this.enter_angle = parseAngleXz(args.enter_angle, `${where}.enter_angle`);
    }

    public static parseArray(args: ReadonlyArray<ActionHitBeHitArgs>, where: string): ReadonlyArray<ActionHitBeHit> {
        if (args.length < 1) {
            throw new Error(`${where}: length must >= 1`);
        }
        const hits = args.map((args, idx) => new ActionHitBeHit(args, `${where}[idx]`));
        hits.sort((a, b) => a.enter_angle - b.enter_angle);
        return hits;
    }
}

export type ActionHitArgs = ActionArgs & {
    /** 进入按键 */
    enter_key: 'Hit1' | 'Hit2' | 'Hit3';

    /** 受击动画 */
    anim_be_hits: ReadonlyArray<ActionHitBeHitArgs>;

    /** 受击倒地动画 */
    anim_down?: AnimationArgs;

    /** 最长倒地时间 超时后自动恢复（recovery） */
    max_down_time?: float | string;

    /** 受击恢复动画 */
    anim_recovery?: AnimationArgs;

    /** 受击硬直结束时间 */
    hit_stun_end?: float | string;
};

export class ActionHit extends Action {
    /** 进入按键 */
    public readonly enter_key: 'Hit1' | 'Hit2' | 'Hit3';

    /** 受击动画 */
    public readonly be_hits: ReadonlyArray<ActionHitBeHit>;

    /** 受击倒地动画 */
    public readonly anim_down?: Animation;

    /** 最长倒地时间 超时后自动恢复（recovery） */
    public readonly max_down_time?: float;

    /** 受击恢复动画 */
    public readonly anim_recovery?: Animation;

    /** 受击硬直结束时间 */
    public readonly hit_stun_end: float;

    /** 进入等级 */
    public readonly enter_level: int;

    /** 派生等级 */
    public readonly derive_level: int;

    public constructor(id: ID, args: ActionHitArgs) {
        super(id, args);
        this.enter_key = parseString(args.enter_key as string, this.w('enter_key'), {
            includes: ['Hit1', 'Hit2', 'Hit3'],
        }) as any;
        this.be_hits = ActionHitBeHit.parseArray(args.anim_be_hits, this.w('anim_be_hits'));
        this.anim_down = !args.anim_down
            ? undefined
            : new Animation(args.anim_down, this.w('anim_down'));
        this.max_down_time = !args.max_down_time
            ? undefined
            : parseTime(args.max_down_time, this.w('max_down_time'), { min: 0 });
        this.anim_recovery = !args.anim_recovery
            ? undefined
            : new Animation(args.anim_recovery, this.w('anim_recovery'));
        this.hit_stun_end = parseTime(args.hit_stun_end || '0s', this.w('hit_stun_end'), {
            min: 0,
        });
        this.enter_level = key_to_enter_level(this.enter_key);
        this.derive_level = key_to_derive_level(this.enter_key);

        Animation.generateLocalID([
            ...this.be_hits.map((anim) => anim.anim),
            this.anim_down,
            this.anim_recovery,
        ]);
    }
}

function key_to_enter_level(key: 'Hit1' | 'Hit2' | 'Hit3'): int {
    switch (key) {
        case 'Hit1': return 610;
        case 'Hit2': return 630;
        case 'Hit3': return 650;
    }
}

function key_to_derive_level(key: 'Hit1' | 'Hit2' | 'Hit3'): int {
    switch (key) {
        case 'Hit1': return 600;
        case 'Hit2': return 620;
        case 'Hit3': return 640;
    }
}

// export type NpcActionHitArgs = NpcActionArgs & {
//     /** 进入按键 */
//     enter_key: 'Hit1' | 'Hit2' | 'Hit3';

//     /** 受击动画 */
//     anim_be_hits: ReadonlyArray<ActionHitBeHitArgs>;

//     /** 受击倒地动画 */
//     anim_down?: AnimationArgs;

//     /** 最长倒地时间 超时后自动恢复（recovery） */
//     max_down_time?: float | string;

//     /** 受击恢复动画 */
//     anim_recovery?: AnimationArgs;

//     /** 受击硬直结束时间 */
//     hit_stun_end?: float | string;
// };

// export class NpcActionHit extends NpcAction {
//     /** 进入按键 */
//     public readonly enter_key: 'Hit1' | 'Hit2' | 'Hit3';

//     /** 受击动画 */
//     public readonly be_hits: ReadonlyArray<ActionHitBeHit>;

//     /** 受击倒地动画 */
//     public readonly anim_down?: Animation;

//     /** 最长倒地时间 超时后自动恢复（recovery） */
//     public readonly max_down_time?: float;

//     /** 受击恢复动画 */
//     public readonly anim_recovery?: Animation;

//     /** 受击硬直结束时间 */
//     public readonly hit_stun_end: float;

//     public constructor(id: ID, args: NpcActionHitArgs) {
//         super(id, args);
//         this.enter_key = parseString(args.enter_key as string, this.w('enter_key'), {
//             includes: ['Hit1', 'Hit2', 'Hit3'],
//         }) as any;
//         this.be_hits = ActionHitBeHit.parseArray(args.anim_be_hits, this.w('anim_be_hits'));
//         this.anim_down = !args.anim_down
//             ? undefined
//             : new Animation(args.anim_down, this.w('anim_down'));
//         this.max_down_time = !args.max_down_time
//             ? undefined
//             : parseTime(args.max_down_time, this.w('max_down_time'), { min: 0 });
//         this.anim_recovery = !args.anim_recovery
//             ? undefined
//             : new Animation(args.anim_recovery, this.w('anim_recovery'));
//         this.hit_stun_end = parseTime(args.hit_stun_end || '0s', this.w('hit_stun_end'), {
//             min: 0,
//         });

//         Animation.generateLocalID([
//             ...this.be_hits.map((anim) => anim.anim),
//             this.anim_down,
//             this.anim_recovery,
//         ]);
//     }
// }

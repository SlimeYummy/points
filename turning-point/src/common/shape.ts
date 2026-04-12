import { float, parseFloat } from './builtin';

export abstract class Shape {
    public toJSON() {
        return {
            T: this.constructor.name,
            ...this,
        };
    }
}

export type BoxArgs = {
    half_x: float | string;
    half_y: float | string;
    half_z: float | string;
};

export class Box extends Shape {
    public readonly half_x: float;
    public readonly half_y: float;
    public readonly half_z: float;

    public constructor(half_x: float, half_y: float, half_z: float);
    public constructor(args: BoxArgs, where: string);
    public constructor(...args: any[]) {
        super();
        if (typeof args[0] === 'object') {
            this.half_x = parseFloat(args[0].half_x, `${args[1]}.half_x`, { min: 0 });
            this.half_y = parseFloat(args[0].half_y, `${args[1]}.half_y`, { min: 0 });
            this.half_z = parseFloat(args[0].half_z, `${args[1]}.half_z`, { min: 0 });
        } else {
            this.half_x = parseFloat(args[0], 'Box.half_x', { min: 0 });
            this.half_y = parseFloat(args[1], 'Box.half_y', { min: 0 });
            this.half_z = parseFloat(args[2], 'Box.half_z', { min: 0 });
        }
    }
}

export type SphereArgs = {
    radius: float | string;
};

export class Sphere extends Shape {
    public readonly radius: number;

    public constructor(radius: float);
    public constructor(args: SphereArgs, where: string);
    public constructor(...args: any[]) {
        super();
        if (typeof args[0] === 'object') {
            this.radius = parseFloat(args[0].radius, `${args[1]}.radius`, { min: 0 });
        } else {
            this.radius = parseFloat(args[0], 'Sphere.radius', { min: 0 });
        }
    }
}

export type CapsuleArgs = {
    half_height: float | string;
    radius: float | string;
};

export class Capsule extends Shape {
    public readonly half_height: number;
    public readonly radius: number;

    public constructor(half_height: float, radius: float);
    public constructor(args: CapsuleArgs, where: string);
    public constructor(...args: any[]) {
        super();
        if (typeof args[0] === 'object') {
            this.half_height = parseFloat(args[0].half_height, `${args[1]}.half_height`, { min: 0 });
            this.radius = parseFloat(args[0].radius, `${args[1]}.radius`, { min: 0 });
        } else {
            this.half_height = parseFloat(args[0], 'Capsule.half_height', { min: 0 });
            this.radius = parseFloat(args[1], 'Capsule.radius', { min: 0 });
        }
    }
}

export type TaperedCapsuleArgs = {
    half_height: float | string;
    top_radius: float | string;
    bottom_radius: float | string;
};

export class TaperedCapsule extends Shape {
    public readonly half_height: number;
    public readonly top_radius: number;
    public readonly bottom_radius: number;

    public constructor(half_height: float, top_radius: float, bottom_radius: float);
    public constructor(args: TaperedCapsuleArgs, where: string);
    public constructor(...args: any[]) {
        super();
        if (typeof args[0] === 'object') {
            this.half_height = parseFloat(args[0].half_height, `${args[1]}.half_height`, { min: 0 });
            this.top_radius = parseFloat(args[0].top_radius, `${args[1]}.top_radius`, { min: 0 });
            this.bottom_radius = parseFloat(args[0].bottom_radius, `${args[1]}.bottom_radius`, { min: 0 });
        } else {
            this.half_height = parseFloat(args[0], 'TaperedCapsule.half_height', { min: 0 });
            this.top_radius = parseFloat(args[1], 'TaperedCapsule.top_radius', { min: 0 });
            this.bottom_radius = parseFloat(args[2], 'TaperedCapsule.bottom_radius', { min: 0 });
        }
    }
}

export type CylinderArgs = {
    half_height: float | string;
    radius: float | string;
};

export class Cylinder extends Shape {
    public readonly half_height: number;
    public readonly radius: number;

    public constructor(half_height: float, radius: float);
    public constructor(args: CylinderArgs, where: string);
    public constructor(...args: any[]) {
        super();
        if (typeof args[0] === 'object') {
            this.half_height = parseFloat(args[0].half_height, `${args[1]}.half_height`, { min: 0 });
            this.radius = parseFloat(args[0].radius, `${args[1]}.radius`, { min: 0 });
        } else {
            this.half_height = parseFloat(args[0], 'Cylinder.half_height', { min: 0 });
            this.radius = parseFloat(args[1], 'Cylinder.radius', { min: 0 });
        }
    }
}

export type TaperedCylinderArgs = {
    half_height: float | string;
    top_radius: float | string;
    bottom_radius: float | string;
};

export class TaperedCylinder extends Shape {
    public readonly half_height: number;
    public readonly top_radius: number;
    public readonly bottom_radius: number;

    public constructor(half_height: float, top_radius: float, bottom_radius: float);
    public constructor(args: TaperedCylinderArgs, where: string);
    public constructor(...args: any[]) {
        super();
        if (typeof args[0] === 'object') {
            this.half_height = parseFloat(args[0].half_height, `${args[1]}.half_height`, { min: 0 });
            this.top_radius = parseFloat(args[0].top_radius, `${args[1]}.top_radius`, { min: 0 });
            this.bottom_radius = parseFloat(args[0].bottom_radius, `${args[1]}.bottom_radius`, { min: 0 });
        } else {
            this.half_height = parseFloat(args[0], 'TaperedCylinder.half_height', { min: 0 });
            this.top_radius = parseFloat(args[1], 'TaperedCylinder.top_radius', { min: 0 });
            this.bottom_radius = parseFloat(args[2], 'TaperedCylinder.bottom_radius', { min: 0 });
        }
    }
}

export type SphericalConeArgs = {
    radius: float | string;
    half_angle: float | string;
};

export class SphericalCone extends Shape {
    public readonly radius: number;
    public readonly half_angle: number;

    public constructor(radius: float, half_angle: float);
    public constructor(args: SphericalConeArgs, where: string);
    public constructor(...args: any[]) {
        super();
        if (typeof args[0] === 'object') {
            this.radius = parseFloat(args[0].radius, `${args[1]}.radius`, { min: 0 });
            this.half_angle = parseFloat(args[0].half_angle, `${args[1]}.half_angle`, {
                min: 0,
                max: 180,
            }) * Math.PI / 180;
        } else {
            this.radius = parseFloat(args[0], 'SphericalCone.radius', { min: 0 });
            this.half_angle = parseFloat(args[1], 'SphericalCone.half_angle', {
                min: 0,
                max: 180,
            }) * Math.PI / 180;
        }
    }
}

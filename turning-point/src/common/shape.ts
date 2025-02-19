import { float, parseFloat } from './builtin';

export class Shape {}

export class Box extends Shape {
    public readonly half_x: float;
    public readonly half_y: float;
    public readonly half_z: float;

    public constructor(half_x: float, half_y: float, half_z: float) {
        super();
        this.half_x = parseFloat(half_x, 'Box.half_x', { min: 0 });
        this.half_y = parseFloat(half_y, 'Box.half_y', { min: 0 });
        this.half_z = parseFloat(half_z, 'Box.half_z', { min: 0 });
    }
}

export class Sphere extends Shape {
    public readonly radius: number;

    public constructor(radius: number) {
        super();
        this.radius = parseFloat(radius, 'Sphere.radius', { min: 0 });
    }
}

export class Capsule extends Shape {
    public readonly half_height: number;
    public readonly radius: number;

    public constructor(half_height: number, radius: number) {
        super();
        this.half_height = parseFloat(half_height, 'Capsule.half_height', {
            min: 0,
        });
        this.radius = parseFloat(radius, 'Capsule.radius', { min: 0 });
    }
}

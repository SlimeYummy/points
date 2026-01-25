using MessagePack.Formatters;
using MessagePack;
using System;
using System.Runtime.InteropServices;

using Cs = System.Numerics;
#if GODOT
using Gd = Godot;
#elif UNITY_EDITOR || UNITY_STANDALONE
using U3d = UnityEngine;
#endif

namespace CriticalPoint {

    //
    // glam Vec2
    //

    [StructLayout(LayoutKind.Sequential, Pack=4)]
    public struct Vec2 : IEquatable<Vec2> {
        public float x;
        public float y;

        public Vec2(float x, float y) {
            this.x = x;
            this.y = y;
        }

        internal float this[int i] {
            get {
                switch (i) {
                    case 0: return x;
                    case 1: return y;
                    default: throw new IndexOutOfRangeException();
                }
            }
            set {
                switch (i) {
                    case 0: x = value; break;
                    case 1: y = value; break;
                    default: throw new IndexOutOfRangeException();
                }
            }
        }

        public bool Equals(Vec2 other) => x == other.x && y == other.y;
        public override bool Equals(object? obj) => obj is Vec2 && Equals((Vec2) obj);
        public override int GetHashCode() => HashCode.Combine(x, y);

        public static bool operator ==(Vec2 a, Vec2 b) => a.x == b.x && a.y == b.y;
        public static bool operator !=(Vec2 a, Vec2 b) => a.x != b.x || a.y != b.y;

        public static explicit operator Vec2(Cs.Vector2 v) => new Vec2(v.X, v.Y);
        public static explicit operator Cs.Vector2(Vec2 v) => new Cs.Vector2(v.x, v.y);

#if GODOT
        public static explicit operator Vec2(Gd.Vector2 v) => new Vec2(v.X, v.Y);
        public static explicit operator Gd.Vector2(Vec2 v) => new Gd.Vector2(v.x, v.y);
#elif UNITY_EDITOR || UNITY_STANDALONE
        public static explicit operator Vec2(U3d.Vector2 v) => new Vec2(v.x, v.y);
        public static explicit operator U3d.Vector2(Vec2 v) => new U3d.Vector2(v.x, v.y);
#endif

        public static readonly Vec2 ZERO = new Vec2(0, 0);
        public bool IsZero => x == 0 && y == 0;

        public static readonly Vec2 ONE = new Vec2(1, 1);
        public bool IsOne => x == 1 && y == 1;
    }

    public class Vec2Formatter : IMessagePackFormatter<Vec2> {
        public void Serialize(ref MessagePackWriter writer, Vec2 vec2, MessagePackSerializerOptions options) {
            writer.WriteArrayHeader(2);
            writer.Write(vec2.x);
            writer.Write(vec2.y);
        }

        public Vec2 Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            if (reader.TryReadNil()) {
                return new Vec2();
            }
            int count = reader.ReadArrayHeader();
            if (count != 2) {
                throw new MessagePackSerializationException("Invalid Vec2 format");
            }
            return new Vec2(reader.ReadSingle(), reader.ReadSingle());
        }
    }

    //
    // glam Vec3
    //

    public struct Vec3 : IEquatable<Vec3> {
        public float x;
        public float y;
        public float z;

        public Vec3(float x, float y, float z) {
            this.x = x;
            this.y = y;
            this.z = z;
        }

        internal float this[int i] {
            get {
                switch (i) {
                    case 0: return x;
                    case 1: return y;
                    case 2: return z;
                    default: throw new IndexOutOfRangeException();
                }
            }
            set {
                switch (i) {
                    case 0: x = value; break;
                    case 1: y = value; break;
                    case 2: z = value; break;
                    default: throw new IndexOutOfRangeException();
                }
            }
        }

        public bool Equals(Vec3 other) => x == other.x && y == other.y && z == other.z;
        public override bool Equals(object? obj) => obj is Vec3 && Equals((Vec3) obj);
        public override int GetHashCode() => HashCode.Combine(x, y, z);
        public bool Equals(Vec3A other) => x == other.x && y == other.y && z == other.z;

        public static bool operator ==(Vec3 a, Vec3 b) => a.x == b.x && a.y == b.y && a.z == b.z;
        public static bool operator !=(Vec3 a, Vec3 b) => a.x != b.x || a.y != b.y || a.z != b.z;
        public static bool operator ==(Vec3 a, Vec3A b) => a.x == b.x && a.y == b.y && a.z == b.z;
        public static bool operator !=(Vec3 a, Vec3A b) => a.x != b.x || a.y != b.y || a.z != b.z;

        public static explicit operator Vec3(Cs.Vector3 v) => new Vec3(v.X, v.Y, v.Z);
        public static explicit operator Cs.Vector3(Vec3 v) => new Cs.Vector3(v.x, v.y, v.z);
        public static explicit operator Vec3A(Vec3 v) => new Vec3A(v.x, v.y, v.z);

#if GODOT
        public static explicit operator Vec3(Gd.Vector3 v) => new Vec3(v.X, v.Y, v.Z);
        public static explicit operator Gd.Vector3(Vec3 v) => new Gd.Vector3(v.x, v.y, v.z);
#elif UNITY_EDITOR || UNITY_STANDALONE
        public static explicit operator Vec3(U3d.Vector3 v) => new Vec3(v.x, v.y, v.z);
        public static explicit operator U3d.Vector3(Vec3 v) => new U3d.Vector3(v.x, v.y, v.z);
#endif

        public static readonly Vec3 ZERO = new Vec3(0, 0, 0);
        public bool IsZero => x == 0 && y == 0 && z == 0;

        public static readonly Vec3 ONE = new Vec3(1, 1, 1);
        public bool IsOne => x == 1 && y == 1 && z == 1;
    }

    public class Vec3Formatter : IMessagePackFormatter<Vec3> {
        public void Serialize(ref MessagePackWriter writer, Vec3 vec3, MessagePackSerializerOptions options) {
            writer.WriteArrayHeader(3);
            writer.Write(vec3.x);
            writer.Write(vec3.y);
            writer.Write(vec3.z);
        }

        public Vec3 Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            if (reader.TryReadNil()) {
                return new Vec3();
            }
            int count = reader.ReadArrayHeader();
            if (count != 3) {
                throw new MessagePackSerializationException("Invalid Vec3 format");
            }
            return new Vec3(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
        }
    }

    //
    // glam Vec3A
    //

    [StructLayout(LayoutKind.Sequential, Pack=4)]
    public struct Vec3A : IEquatable<Vec3A> {
        public float x;
        public float y;
        public float z;
        public float _;

        public Vec3A(float x, float y, float z) {
            this.x = x;
            this.y = y;
            this.z = z;
            this._ = 0;
        }

        internal float this[int i] {
            get {
                switch (i) {
                    case 0: return x;
                    case 1: return y;
                    case 2: return z;
                    default: throw new IndexOutOfRangeException();
                }
            }
            set {
                switch (i) {
                    case 0: x = value; break;
                    case 1: y = value; break;
                    case 2: z = value; break;
                    default: throw new IndexOutOfRangeException();
                }
            }
        }

        public bool Equals(Vec3 other) => x == other.x && y == other.y && z == other.z;
        public override bool Equals(object? obj) => obj is Vec3A && Equals((Vec3A) obj);
        public override int GetHashCode() => HashCode.Combine(x, y, z);
        public bool Equals(Vec3A other) => x == other.x && y == other.y && z == other.z;

        public static bool operator ==(Vec3A a, Vec3A b) => a.x == b.x && a.y == b.y && a.z == b.z;
        public static bool operator !=(Vec3A a, Vec3A b) => a.x != b.x || a.y != b.y || a.z != b.z;
        public static bool operator ==(Vec3A a, Vec3 b) => a.x == b.x && a.y == b.y && a.z == b.z;
        public static bool operator !=(Vec3A a, Vec3 b) => a.x != b.x || a.y != b.y || a.z != b.z;

        public static explicit operator Vec3A(Cs.Vector3 v) => new Vec3A(v.X, v.Y, v.Z);
        public static explicit operator Cs.Vector3(Vec3A v) => new Cs.Vector3(v.x, v.y, v.z);
        public static explicit operator Vec3A(Vec3 v) => new Vec3A(v.x, v.y, v.z);

#if GODOT
        public static explicit operator Vec3A(Gd.Vector3 v) => new Vec3A(v.X, v.Y, v.Z);
        public static explicit operator Gd.Vector3(Vec3A v) => new Gd.Vector3(v.x, v.y, v.z);
#elif UNITY_EDITOR || UNITY_STANDALONE
        public static explicit operator Vec3A(U3d.Vector3 v) => new Vec3A(v.x, v.y, v.z);
        public static explicit operator U3d.Vector3(Vec3A v) => new U3d.Vector3(v.x, v.y, v.z);
#endif

        public static readonly Vec3A ZERO = new Vec3A(0, 0, 0);
        public bool IsZero => x == 0 && y == 0 && z == 0;

        public static readonly Vec3A ONE = new Vec3A(1, 1, 1);
        public bool IsOne => x == 1 && y == 1 && z == 1;
    }

    public class Vec3AFormatter : IMessagePackFormatter<Vec3A> {
        public void Serialize(ref MessagePackWriter writer, Vec3A vec3, MessagePackSerializerOptions options) {
            writer.WriteArrayHeader(3);
            writer.Write(vec3.x);
            writer.Write(vec3.y);
            writer.Write(vec3.z);
        }

        public Vec3A Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            if (reader.TryReadNil()) {
                return new Vec3A();
            }
            int count = reader.ReadArrayHeader();
            if (count != 3) {
                throw new MessagePackSerializationException("Invalid Vec3A format");
            }
            return new Vec3A(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
        }
    }

    //
    // glam Vec4
    //

    [StructLayout(LayoutKind.Sequential, Pack=4)]
    public struct Vec4 : IEquatable<Vec4> {
        public float x;
        public float y;
        public float z;
        public float w;

        public Vec4(float x, float y, float z, float w) {
            this.x = x;
            this.y = y;
            this.z = z;
            this.w = w;
        }

        internal float this[int i] {
            get {
                switch (i) {
                    case 0: return x;
                    case 1: return y;
                    case 2: return z;
                    case 3: return w;
                    default: throw new IndexOutOfRangeException();
                }
            }
            set {
                switch (i) {
                    case 0: x = value; break;
                    case 1: y = value; break;
                    case 2: z = value; break;
                    case 3: w = value; break;
                    default: throw new IndexOutOfRangeException();
                }
            }
        }

        public bool Equals(Vec4 other) => x == other.x && y == other.y && z == other.z && w == other.w;
        public override bool Equals(object? obj) => obj is Vec4 && Equals((Vec4) obj);
        public override int GetHashCode() => HashCode.Combine(x, y, z, w);

        public static bool operator ==(Vec4 a, Vec4 b) => a.x == b.x && a.y == b.y && a.z == b.z && a.w == b.w;
        public static bool operator !=(Vec4 a, Vec4 b) => a.x != b.x || a.y != b.y || a.z != b.z || a.w != b.w;

        public static explicit operator Vec4(Cs.Vector4 v) => new Vec4(v.X, v.Y, v.Z, v.W);
        public static explicit operator Cs.Vector4(Vec4 v) => new Cs.Vector4(v.x, v.y, v.z, v.w);

#if GODOT
        public static explicit operator Vec4(Gd.Vector4 v) => new Vec4(v.X, v.Y, v.Z, v.W);
        public static explicit operator Gd.Vector4(Vec4 v) => new Gd.Vector4(v.x, v.y, v.z, v.w);
#elif UNITY_EDITOR || UNITY_STANDALONE
        public static explicit operator Vec4(U3d.Vector4 v) => new Vec4(v.x, v.y, v.z, v.w);
        public static explicit operator U3d.Vector4(Vec4 v) => new U3d.Vector4(v.x, v.y, v.z, v.w);
#endif

        public static readonly Vec4 ZERO = new Vec4(0, 0, 0, 0);
        public bool IsZero => x == 0 && y == 0 && z == 0 && w == 0;

        public static readonly Vec4 ONE = new Vec4(1, 1, 1, 1);
        public bool IsOne => x == 1 && y == 1 && z == 1 && w == 1;
    }

    public class Vec4Formatter : IMessagePackFormatter<Vec4> {
        public void Serialize(ref MessagePackWriter writer, Vec4 vec4, MessagePackSerializerOptions options) {
            writer.WriteArrayHeader(4);
            writer.Write(vec4.x);
            writer.Write(vec4.y);
            writer.Write(vec4.z);
            writer.Write(vec4.w);
        }

        public Vec4 Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            if (reader.TryReadNil()) {
                return new Vec4();
            }
            int count = reader.ReadArrayHeader();
            if (count != 4) {
                throw new MessagePackSerializationException("Invalid Vec4 format");
            }
            return new Vec4(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
        }
    }

    //
    // glam Quat
    //

    [StructLayout(LayoutKind.Sequential, Pack=4)]
    public struct Quat : IEquatable<Quat> {
        public float x;
        public float y;
        public float z;
        public float w;

        public Quat(float x, float y, float z, float w) {
            this.x = x;
            this.y = y;
            this.z = z;
            this.w = w;
        }

        public bool Equals(Quat other) => x == other.x && y == other.y && z == other.z && w == other.w;
        public override bool Equals(object? obj) => obj is Quat && Equals((Quat) obj);
        public override int GetHashCode() => HashCode.Combine(x, y, z, w);

        public static bool operator ==(Quat a, Quat b) => a.x == b.x && a.y == b.y && a.z == b.z && a.w == b.w;
        public static bool operator !=(Quat a, Quat b) => a.x != b.x || a.y != b.y || a.z != b.z || a.w != b.w;

        public static explicit operator Quat(Cs.Quaternion q) => new Quat(q.X, q.Y, q.Z, q.W);
        public static explicit operator Cs.Quaternion(Quat q) => new Cs.Quaternion(q.x, q.y, q.z, q.w);

#if GODOT
        public static explicit operator Quat(Gd.Quaternion q) => new Quat(q.X, q.Y, q.Z, q.W);
        public static explicit operator Gd.Quaternion(Quat q) => new Gd.Quaternion(q.x, q.y, q.z, q.w);
#elif UNITY_EDITOR || UNITY_STANDALONE
        public static explicit operator Quat(U3d.Quaternion q) => new Quat(q.x, q.y, q.z, q.w);
        public static explicit operator U3d.Quaternion(Quat q) => new U3d.Quaternion(q.x, q.y, q.z, q.w);
#endif

        public static readonly Quat ZERO = new Quat(0, 0, 0, 0);
        public bool IsZero => x == 0 && y == 0 && z == 0 && w == 0;

        public static readonly Quat IDENTITY = new Quat(0, 0, 0, 1);
        public bool IsIdentity => x == 0 && y == 0 && z == 0 && w == 1;
    }

    public class QuatFormatter : IMessagePackFormatter<Quat> {
        public void Serialize(ref MessagePackWriter writer, Quat quat, MessagePackSerializerOptions options) {
            writer.WriteArrayHeader(4);
            writer.Write(quat.x);
            writer.Write(quat.y);
            writer.Write(quat.z);
            writer.Write(quat.w);
        }

        public Quat Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            if (reader.TryReadNil()) {
                return new Quat();
            }
            int count = reader.ReadArrayHeader();
            if (count != 4) {
                throw new MessagePackSerializationException("Invalid Quat format");
            }
            return new Quat(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
        }
    }

    //
    // glam Mat4 (column-major)
    //
    
    [StructLayout(LayoutKind.Sequential, Pack=16)]
    public struct Mat4 : IEquatable<Mat4> {
        public Vec4 x_axis;
        public Vec4 y_axis;
        public Vec4 z_axis;
        public Vec4 w_axis;

        public Mat4(
            float m00, float m01, float m02, float m03,
            float m10, float m11, float m12, float m13,
            float m20, float m21, float m22, float m23,
            float m30, float m31, float m32, float m33
        ) {
            x_axis = new Vec4(m00, m01, m02, m03);
            y_axis = new Vec4(m10, m11, m12, m13);
            z_axis = new Vec4(m20, m21, m22, m23);
            w_axis = new Vec4(m30, m31, m32, m33);
        }

        public Mat4(Vec4 x_axis, Vec4 y_axis, Vec4 z_axis, Vec4 w_axis) {
            this.x_axis = x_axis;
            this.y_axis = y_axis;
            this.z_axis = z_axis;
            this.w_axis = w_axis;
        }

        public bool Equals(Mat4 other) =>
            x_axis.Equals(other.x_axis) &&
            y_axis.Equals(other.y_axis) &&
            z_axis.Equals(other.z_axis) &&
            w_axis.Equals(other.w_axis);
        public override bool Equals(object? obj) => obj is Mat4 other && Equals(other);
        public override int GetHashCode() => HashCode.Combine(x_axis, y_axis, z_axis, w_axis);
        
        public static bool operator ==(Mat4 a, Mat4 b) =>
            a.x_axis == b.x_axis &&
            a.y_axis == b.y_axis &&
            a.z_axis == b.z_axis &&
            a.w_axis == b.w_axis;
        
        public static bool operator !=(Mat4 a, Mat4 b) =>
            a.x_axis != b.x_axis ||
            a.y_axis != b.y_axis ||
            a.z_axis != b.z_axis ||
            a.w_axis != b.w_axis;
        
        // C# math library uses row-major matrices
        public static explicit operator Mat4(Cs.Matrix4x4 m) => new Mat4(
            m.M11, m.M21, m.M31, m.M41,
            m.M12, m.M22, m.M32, m.M42,
            m.M13, m.M23, m.M33, m.M43,
            m.M14, m.M24, m.M34, m.M44
        );

        // C# math library uses row-major matrices
        public static explicit operator Cs.Matrix4x4(Mat4 m) => new Cs.Matrix4x4(
            m.x_axis.x, m.y_axis.x, m.z_axis.x, m.w_axis.x,
            m.x_axis.y, m.y_axis.y, m.z_axis.y, m.w_axis.y,
            m.x_axis.z, m.y_axis.z, m.z_axis.z, m.w_axis.z,
            m.x_axis.w, m.y_axis.w, m.z_axis.w, m.w_axis.w
        );

#if UNITY_EDITOR || UNITY_STANDALONE
        // Unity3d uses column-major matrices
        public static explicit operator Mat4(U3d.Matrix4x4 m) => new Mat4(
            m[0,0], m[0,1], m[0,2], m[0,3],
            m[1,0], m[1,1], m[1,2], m[1,3],
            m[2,0], m[2,1], m[2,2], m[2,3],
            m[3,0], m[3,1], m[3,2], m[3,3]
        );

        // Unity3d uses column-major matrices
        public static explicit operator U3d.Matrix4x4(Mat4 m) => new U3d.Matrix4x4(
            (U3d.Vector4)m.x_axis,
            (U3d.Vector4)m.y_axis,
            (U3d.Vector4)m.z_axis,
            (U3d.Vector4)m.w_axis
        );
#endif
        
        public static readonly Mat4 ZERO = new Mat4(Vec4.ZERO, Vec4.ZERO, Vec4.ZERO, Vec4.ZERO);
        public static readonly Mat4 IDENTITY = new Mat4(new Vec4(1, 0, 0, 0), new Vec4(0, 1, 0, 0), new Vec4(0, 0, 1, 0), new Vec4(0, 0, 0, 1));
    }

    public class Mat4Formatter : IMessagePackFormatter<Mat4> {
        public void Serialize(ref MessagePackWriter writer, Mat4 mat4, MessagePackSerializerOptions options) {
            writer.WriteArrayHeader(16);
            writer.Write(mat4.x_axis.x);
            writer.Write(mat4.x_axis.y);
            writer.Write(mat4.x_axis.z);
            writer.Write(mat4.x_axis.w);
            writer.Write(mat4.y_axis.x);
            writer.Write(mat4.y_axis.y);
            writer.Write(mat4.y_axis.z);
            writer.Write(mat4.y_axis.w);
            writer.Write(mat4.z_axis.x);
            writer.Write(mat4.z_axis.y);
            writer.Write(mat4.z_axis.z);
            writer.Write(mat4.z_axis.w);
            writer.Write(mat4.w_axis.x);
            writer.Write(mat4.w_axis.y);
            writer.Write(mat4.w_axis.z);
            writer.Write(mat4.w_axis.w);
        }

        public Mat4 Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            if (reader.TryReadNil()) {
                return new Mat4();
            }
            int count = reader.ReadArrayHeader();
            if (count != 16) {
                throw new MessagePackSerializationException("Invalid Mat4 format");
            }
            return new Mat4(
                new Vec4(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle()),
                new Vec4(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle()),
                new Vec4(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle()),
                new Vec4(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle())
            );
        }
    }

    //
    // glam Transform3A
    //

    [StructLayout(LayoutKind.Sequential, Pack=4)]
    public struct Transform3A : IEquatable<Transform3A> {
        public Vec3A translation;
        public Quat rotation;
        public Vec3A scale;

        public Transform3A(Vec3A translation, Quat rotation, Vec3A scale) {
            this.translation = translation;
            this.rotation = rotation;
            this.scale = scale;
        }

        public bool Equals(Transform3A other) =>
            translation.Equals(other.translation)
                && rotation.Equals(other.rotation)
                && scale.Equals(other.scale);
        public override bool Equals(object? obj) => obj is Transform3A other && Equals(other);
        public override int GetHashCode() => HashCode.Combine(translation, rotation, scale);

        public static readonly Transform3A ZERO = new Transform3A(Vec3A.ZERO, Quat.IDENTITY, Vec3A.ONE);
        public bool IsZero => translation.IsZero && rotation.IsZero && scale.IsZero;

        public static readonly Transform3A IDENTITY = new Transform3A(Vec3A.ZERO, Quat.IDENTITY, Vec3A.ONE);
        public bool IsIdentity => translation.IsZero && rotation.IsIdentity && scale.IsOne;
    }

    public class Transform3AFormatter : IMessagePackFormatter<Transform3A> {
        public void Serialize(ref MessagePackWriter writer, Transform3A transform, MessagePackSerializerOptions options) {
            writer.WriteArrayHeader(10);
            writer.Write(transform.translation.x);
            writer.Write(transform.translation.y);
            writer.Write(transform.translation.z);
            writer.Write(transform.rotation.x);
            writer.Write(transform.rotation.y);
            writer.Write(transform.rotation.z);
            writer.Write(transform.rotation.w);
            writer.Write(transform.scale.x);
            writer.Write(transform.scale.y);
            writer.Write(transform.scale.z);
        }

        public Transform3A Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            if (reader.TryReadNil()) {
                return new Transform3A();
            }
            int count = reader.ReadArrayHeader();
            if (count != 10) {
                throw new MessagePackSerializationException("Invalid Transform3A format");
            }
            Vec3A translation = new Vec3A(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
            Quat rotation = new Quat(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
            Vec3A scale = new Vec3A(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
            return new Transform3A(translation, rotation, scale);
        }
    }

    //
    // glam-ext Isometry3A
    //

    [StructLayout(LayoutKind.Sequential, Pack = 4)]
    public struct Isometry3A : IEquatable<Isometry3A> {
        public Vec3A translation;
        public Quat rotation;

        public Isometry3A(Vec3A translation, Quat rotation) {
            this.translation = translation;
            this.rotation = rotation;
        }

        public bool Equals(Isometry3A other) => translation.Equals(other.translation) && rotation.Equals(other.rotation);
        public override bool Equals(object? obj) => obj is Isometry3A other && Equals(other);
        public override int GetHashCode() => HashCode.Combine(translation, rotation);

        public static readonly Transform3A ZERO = new Transform3A(Vec3A.ZERO, Quat.IDENTITY, Vec3A.ONE);
        public bool IsZero => translation.IsZero && rotation.IsZero;

        public static readonly Transform3A IDENTITY = new Transform3A(Vec3A.ZERO, Quat.IDENTITY, Vec3A.ONE);
        public bool IsIdentity => translation.IsZero && rotation.IsIdentity;
    }

    public class Isometry3AFormatter : IMessagePackFormatter<Isometry3A> {
        public void Serialize(ref MessagePackWriter writer, Isometry3A transform, MessagePackSerializerOptions options) {
            writer.WriteArrayHeader(10);
            writer.Write(transform.translation.x);
            writer.Write(transform.translation.y);
            writer.Write(transform.translation.z);
            writer.Write(transform.rotation.x);
            writer.Write(transform.rotation.y);
            writer.Write(transform.rotation.z);
            writer.Write(transform.rotation.w);
        }

        public Isometry3A Deserialize(ref MessagePackReader reader, MessagePackSerializerOptions options) {
            if (reader.TryReadNil()) {
                return new Isometry3A();
            }
            int count = reader.ReadArrayHeader();
            if (count != 10) {
                throw new MessagePackSerializationException("Invalid Isometry3A format");
            }
            Vec3A translation = new Vec3A(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
            Quat rotation = new Quat(reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle(), reader.ReadSingle());
            return new Isometry3A(translation, rotation);
        }
    }

    //
    // ozz-animaiton Soa
    //

    [StructLayout(LayoutKind.Sequential, Pack=16)]
    public struct SoaVec3 {
        public Vec4 x;
        public Vec4 y;
        public Vec4 z;

        public SoaVec3(Vec4 x, Vec4 y, Vec4 z) {
            this.x = x;
            this.y = y;
            this.z = z;
        }

        public Vec3 this[int i] {
            get => new Vec3(x[i], y[i], z[i]);
            set {
                x[i] = value.x;
                y[i] = value.y;
                z[i] = value.z;
            }
        }
    }
    
    [StructLayout(LayoutKind.Sequential, Pack=16)]
    public struct SoaQuat {
        public Vec4 x;
        public Vec4 y;
        public Vec4 z;
        public Vec4 w;

        public SoaQuat(Vec4 x, Vec4 y, Vec4 z, Vec4 w) {
            this.x = x;
            this.y = y;
            this.z = z;
            this.w = w;
        }

        public Quat this[int i] {
            get => new Quat(x[i], y[i], z[i], w[i]);
            set {
                x[i] = value.x;
                y[i] = value.y;
                z[i] = value.z;
                w[i] = value.w;
            }
        }
    }
    
    [StructLayout(LayoutKind.Sequential, Pack=16)]
    public struct SoaTransform {
        public SoaVec3 translation;
        public SoaQuat rotation;
        public SoaVec3 scale;

        public SoaTransform(SoaVec3 translation, SoaQuat rotation, SoaVec3 scale) {
            this.translation = translation;
            this.rotation = rotation;
            this.scale = scale;
        }
    }
}

use glam_ext::{Mat4, Transform3A};
use ozz_animation_rs::{LocalToModelJob, LocalToModelJobRef, Skeleton, SoaTransform};

use crate::utils::{xfrom, xres, XResult};

pub fn soa_transforms_to_transforms(soa_transforms: &[SoaTransform], transforms: &mut [Transform3A]) -> XResult<()> {
    if transforms.len() < soa_transforms.len() * 4 {
        return xres!(BadArgument; "buffer too short");
    }
    for (idx, out) in transforms.iter_mut().take(soa_transforms.len() * 4).enumerate() {
        *out = soa_transforms[idx / 4].transform(idx % 4);
    }
    Ok(())
}

pub fn soa_transforms_to_matrices(soa_transforms: &[SoaTransform], matrices: &mut [Mat4]) -> XResult<()> {
    if matrices.len() < soa_transforms.len() * 4 {
        return xres!(BadArgument; "buffer too short");
    }
    for (idx, out) in matrices.iter_mut().take(soa_transforms.len() * 4).enumerate() {
        *out = Mat4::from(soa_transforms[idx / 4].transform(idx % 4));
    }
    Ok(())
}

pub fn matrices_to_transforms(matrices: &[Mat4], transforms: &mut [Transform3A]) -> XResult<()> {
    if transforms.len() < matrices.len() {
        return xres!(BadArgument; "buffer too short");
    }
    for (idx, out) in transforms.iter_mut().take(matrices.len()).enumerate() {
        *out = Transform3A::from_mat4(matrices[idx]);
    }
    Ok(())
}

pub fn rest_poses_to_model_matrices(skeleton: &Skeleton, matrices: &mut [Mat4]) -> XResult<()> {
    let mut l2m: LocalToModelJobRef<'_> = LocalToModelJob::default();
    l2m.set_skeleton(&skeleton);
    l2m.set_input(skeleton.joint_rest_poses());
    l2m.set_output(matrices);
    l2m.run().map_err(xfrom!())?;
    Ok(())
}

pub fn rest_poses_to_model_transforms(skeleton: &Skeleton, transfroms: &mut [Transform3A]) -> XResult<()> {
    let mut matrices = vec![Mat4::ZERO; skeleton.num_joints()];
    rest_poses_to_model_matrices(skeleton, &mut matrices)?;
    matrices_to_transforms(&matrices, transfroms)?;
    Ok(())
}

use glam_ext::{Mat4, Transform3A};
use ozz_animation_rs::{LocalToModelJob, LocalToModelJobRef, Skeleton, SoaTransform};

use crate::utils::{xfrom, XResult};

pub fn soa_transforms_to_transforms(soa_transforms: &[SoaTransform], transforms: &mut [Transform3A]) {
    for idx in 0..transforms.len() {
        transforms[idx] = soa_transforms[idx / 4].transform(idx % 4);
    }
}

pub fn soa_transforms_to_matrices(soa_transforms: &[SoaTransform], matrices: &mut [Mat4]) {
    for idx in 0..matrices.len() {
        matrices[idx] = Mat4::from(soa_transforms[idx / 4].transform(idx % 4));
    }
}

pub fn rest_poses_to_local_transforms(skeleton: &Skeleton) -> XResult<Vec<Transform3A>> {
    let mut transforms = vec![Transform3A::ZERO; skeleton.num_joints()];
    soa_transforms_to_transforms(skeleton.joint_rest_poses(), &mut transforms);
    return Ok(transforms);
}

pub fn rest_poses_to_local_matrices(skeleton: &Skeleton) -> XResult<Vec<Mat4>> {
    let mut matrices = vec![Mat4::ZERO; skeleton.num_joints()];
    soa_transforms_to_matrices(skeleton.joint_rest_poses(), &mut matrices);
    return Ok(matrices);
}

pub fn rest_poses_to_model_matrices(skeleton: &Skeleton) -> XResult<Vec<Mat4>> {
    let mut matrices = vec![Mat4::ZERO; skeleton.num_joints()];
    let mut l2m: LocalToModelJobRef<'_> = LocalToModelJob::default();
    l2m.set_skeleton(&skeleton);
    l2m.set_input(skeleton.joint_rest_poses());
    l2m.set_output(&mut matrices);
    l2m.run().map_err(xfrom!())?;
    return Ok(matrices);
}

pub fn rest_poses_to_model_transforms(skeleton: &Skeleton) -> XResult<Vec<Transform3A>> {
    let matrices = rest_poses_to_model_matrices(skeleton)?;
    let transforms = matrices.iter().map(|mat| Transform3A::from_mat4(*mat)).collect();
    return Ok(transforms);
}

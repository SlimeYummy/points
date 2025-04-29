use cirtical_point_core::utils::XError;
use napi::{Error, Status};
use ozz_animation_rs::OzzError;

#[inline]
pub fn cp_err(xerr: XError) -> Error<Status> {
    Error::new(Status::GenericFailure, xerr.msg())
}

#[inline]
pub fn ozz_err(ozz_err: OzzError) -> Error<Status> {
    Error::new(Status::GenericFailure, ozz_err.to_string())
}

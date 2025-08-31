#![allow(dead_code)]

use cirtical_point_core::utils::XError;
use napi::{Error, Status};
use ozz_animation_rs::OzzError;

#[inline]
pub fn cp_err(xerr: XError) -> Error<Status> {
    Error::new(Status::GenericFailure, xerr.msg())
}

#[inline]
pub fn cp_err_msg(xerr: XError, msg: &str) -> Error<Status> {
    Error::new(Status::GenericFailure, format!("{} => {}", xerr.msg(), msg))
}

#[inline]
pub fn ozz_err(ozz_err: OzzError) -> Error<Status> {
    Error::new(Status::GenericFailure, ozz_err.to_string())
}

#[inline]
pub fn ozz_err_msg(ozz_err: OzzError, msg: &str) -> Error<Status> {
    Error::new(Status::GenericFailure, format!("{} => {}", ozz_err, msg))
}

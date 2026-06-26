#![feature(try_blocks)]
#![feature(ptr_metadata)]
#![feature(allocator_api)]
#![feature(likely_unlikely)]
#![feature(stmt_expr_attributes)]
#![feature(coroutines, iter_from_coroutine)]
#![feature(test)]
extern crate test;

#[cfg(not(debug_assertions))]
use mimalloc::MiMalloc;

#[cfg(not(debug_assertions))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub mod animation;
pub mod asset;
pub mod consts;
pub mod instance;
pub mod parameter;
pub mod template;
pub mod utils;

#[cfg(not(feature = "for-turning-point"))]
pub mod engine;
#[cfg(not(feature = "for-turning-point"))]
pub mod input;
#[cfg(not(feature = "for-turning-point"))]
pub mod logic;
#[cfg(not(feature = "for-turning-point"))]
pub mod save;
#[cfg(not(feature = "for-turning-point"))]
pub mod script;

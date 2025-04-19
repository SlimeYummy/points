#![deny(rust_2018_idioms)]
#![feature(ptr_metadata)]
#![feature(trivial_bounds)]
#![feature(allocator_api)]
#![feature(error_generic_member_access)]
#![allow(unexpected_cfgs)] // TODO: Upgrade rkyv to 0.8

#[cfg(not(debug_assertions))]
use mimalloc::MiMalloc;

#[cfg(not(debug_assertions))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub mod animation;
pub mod asset;
pub mod consts;
pub mod engine;
pub mod instance;
pub mod logic;
pub mod parameter;
pub mod script;
pub mod template;
pub mod template2;
pub mod utils;

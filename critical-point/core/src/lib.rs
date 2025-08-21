#![feature(ptr_metadata)]
#![feature(allocator_api)]
#![feature(likely_unlikely)]
#![feature(coroutines, coroutine_trait, stmt_expr_attributes, iter_from_coroutine)]
#![feature(const_pathbuf_osstring_new)]
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
pub mod engine;
pub mod instance;
pub mod logic;
pub mod parameter;
// pub mod script;
pub mod template;
// pub mod template3;
pub mod utils;

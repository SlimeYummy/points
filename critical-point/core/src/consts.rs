use glam::{Vec2, Vec3A};

pub const FPS_U32: u32 = 30;
pub const FPS_USIZE: usize = FPS_U32 as usize;
pub const FPS: f32 = FPS_U32 as f32; // frames per second
pub const SPF: f32 = 1.0 / FPS; // seconds per frame

pub const STD_FPS_U32: u32 = 15;
pub const STD_FPS_USIZE: u32 = STD_FPS_U32 as u32;
pub const STD_FPS: f32 = STD_FPS_U32 as f32;
pub const STD_SPF: f32 = 1.0 / STD_FPS;

pub const MAX_PLAYER: usize = 8;
pub const MAX_INPUT_WINDOW: u32 = 1 * FPS_U32;

pub const EQUIPMENT_MAX_COUNT: usize = 3;
pub const ACCESSORY_MAX_COUNT: usize = 4;
pub const MAX_ENTRY_PLUS: u32 = 3;

pub const MAX_ACTION_ANIMATION: usize = 4;
pub const MAX_ACTION_STATE: usize = 4;
pub const ACTION_WEIGHT_THRESHOLD: f32 = 0.01;
pub const ACTION_DEFAULT_FADE_IN: f32 = 5.0 / FPS;

pub const MAX_WALK_DIR_LENGTH: f32 = 0.5;
pub const MIN_RUN_DIR_LENGTH: f32 = 0.5;

/// default camera direction
pub const DEFAULT_VIEW_DIR_2D: Vec2 = Vec2::NEG_Y;
/// default camera direction
pub const DEFAULT_VIEW_DIR_3D: Vec3A = Vec3A::NEG_Z;

/// default character toward direction
pub const DEFAULT_TOWARD_DIR_2D: Vec2 = Vec2::Y;
/// default character toward direction
pub const DEFAULT_TOWARD_DIR_3D: Vec3A = Vec3A::Z;

#[cfg(test)]
pub const TEST_TMP_PATH: &str = "../../test-tmp";
#[cfg(test)]
pub const TEST_TMP_RES_PATH: &str = "../../test-tmp/test-template";
#[cfg(test)]
pub const TEST_ASSET_PATH: &str = "../../test-asset";

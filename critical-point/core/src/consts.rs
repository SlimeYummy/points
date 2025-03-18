pub const FLOAT_EPSILON: f32 = 1e-4;

pub const FPS: u32 = 15;
pub const MAX_PLAYER: usize = 8;
pub const MAX_INPUT_WINDOW: u32 = 3 * FPS;

pub const MAX_EQUIPMENT_COUNT: usize = 3;
pub const MAX_ACCESSORY_COUNT: usize = 4;
pub const MAX_ENTRY_PLUS: u32 = 3;

pub const MAX_ACTION_ANIMATION: usize = 4;
pub const WEIGHT_THRESHOLD: f32 = 0.01;

pub const MAX_WALK_DIR_LENGTH: f32 = 0.5;
pub const MIN_RUN_DIR_LENGTH: f32 = 0.5;

#[cfg(test)]
pub const TEST_TEMPLATE_PATH: &str = "../../turning-point/test-templates";
#[cfg(test)]
pub const TEST_ASSET_PATH: &str = "../../turning-point/test-assets";

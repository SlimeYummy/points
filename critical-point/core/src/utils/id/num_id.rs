use crate::consts::MAX_PLAYER;

pub type NumID = u64;

#[inline]
pub fn is_invalid_num_id(id: NumID) -> bool {
    id == u64::MAX
}

pub const GAME_ID: NumID = 1;
pub const STAGE_ID: NumID = 2;
pub const MIN_PLAYER_ID: NumID = 100;
pub const MAX_PLAYER_ID: NumID = MIN_PLAYER_ID + (MAX_PLAYER as u64);

#[inline]
pub fn is_valid_player_id(id: NumID) -> bool {
    id >= MIN_PLAYER_ID && id <= MAX_PLAYER_ID
}

#[inline]
pub fn is_invalid_player_id(id: NumID) -> bool {
    id < MIN_PLAYER_ID || id > MAX_PLAYER_ID
}

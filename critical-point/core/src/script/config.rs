// mutable segments
pub const SEGMENT_REGISTER: u8 = 0; // for stask and local vars
pub const SEGMENT_CLOSURE: u8 = 1; // for arguments and self vars
pub const SEGMENT_OUT_MIN: u8 = 2;
pub const SEGMENT_OUT_MAX: u8 = 6;

pub const SEGMENT_CONSTANT: u8 = 7; // for constant, such as number or pc(code pointer)
pub const SEGMENT_IN_MIN: u8 = 8;
pub const SEGMENT_IN_MAX: u8 = 14;
pub const SEGMENT_STRING: u8 = 15; // for symbol(string), such as ids or tags

pub const SEGMENT_COUNT: usize = 16;
pub const NUM_SEGMENT_COUNT: usize = 16;
pub const MUT_SEGMENT_COUNT: usize = 6;
pub const IN_SEGMENT_COUNT: usize = (SEGMENT_OUT_MAX - SEGMENT_OUT_MIN + 1) as usize;
pub const OUT_SEGMENT_COUNT: usize = (SEGMENT_IN_MAX - SEGMENT_IN_MIN + 1) as usize;

pub const SEGMENT_NAMES: [&'static str; SEGMENT_COUNT] = [
    "register", "closure", "out_0", "out_1", "out_2", "out_3", "out_4", "constant", "in_0", "in_1", "in_2", "in_3",
    "in_4", "in_5", "in_6", "string",
];

pub const MAX_REGISTER: usize = 48;
pub const MAX_LOCAL: usize = 24;
pub const MAX_CLOSURE: usize = 32;
pub const MAX_OFFSET: usize = 0xFFF;

pub const MAX_FUNCTION_ARGUMENTS: usize = 8;
pub const MAX_INOUT_OFFSET: usize = 0xFFF * 8;

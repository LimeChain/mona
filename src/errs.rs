use pinocchio::error::ProgramError;

pub const UNSUPPORTED_IX: u32 = 0x00;
pub const UNSUPPORTED_DEX: u32 = 0x01;
pub const NOT_ENOUGH_ACCS: u32 = 0x02;
pub const OUTPUT_BELOW_MIN: u32 = 0x03;
pub const ZERO_SWAP_DATA: u32 = 0x04;
pub const IX_DATA_TOO_SHORT: u32 = 0x05;
pub const ROUTE_PLAN_IS_INADEQUATE: u32 = 0x06;
pub const UNKNOWN_FLAGS: u32 = 0x07;

pub fn unsupported_ix() -> ProgramError {
    ProgramError::Custom(UNSUPPORTED_IX)
}

pub fn unsupported_dex() -> ProgramError {
    ProgramError::Custom(UNSUPPORTED_DEX)
}

/// Encode step index and dex discriminant into the error code.
pub fn not_enough_accs(step: u8, dex: u8) -> ProgramError {
    ProgramError::Custom(NOT_ENOUGH_ACCS | (dex as u32) << 16 | (step as u32) << 24)
}

pub fn output_below_min() -> ProgramError {
    ProgramError::Custom(OUTPUT_BELOW_MIN)
}

pub fn zero_swap_data() -> ProgramError {
    ProgramError::Custom(ZERO_SWAP_DATA)
}

pub fn ix_data_too_short() -> ProgramError {
    ProgramError::Custom(IX_DATA_TOO_SHORT)
}

pub fn route_plan_is_inadequate() -> ProgramError {
    ProgramError::Custom(ROUTE_PLAN_IS_INADEQUATE)
}

pub fn unknown_flags() -> ProgramError {
    ProgramError::Custom(UNKNOWN_FLAGS)
}

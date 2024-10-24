use crate::Byte;

// Load Accumulator

pub const LDA_IM: Byte = 0xA9;
pub const LDA_ZP: Byte = 0xA5;
pub const LDA_ZPX: Byte = 0xB5;
pub const LDA_ABS: Byte = 0xAD;
pub const LDA_ABSX: Byte = 0xBD;
pub const LDA_ABSY: Byte = 0xB9;
pub const LDA_INDX: Byte = 0xA1;
pub const LDA_INDY: Byte = 0xB1;

// Load X Register

pub const LDX_IM: Byte = 0xA2;
pub const LDX_ZP: Byte = 0xA6;
pub const LDX_ZPY: Byte = 0xB6;
pub const LDX_ABS: Byte = 0xAE;
pub const LDX_ABSY: Byte = 0xBE;

// Load Y Register

pub const LDY_IM: Byte = 0xA0;
pub const LDY_ZP: Byte = 0xA4;
pub const LDY_ZPX: Byte = 0xB4;
pub const LDY_ABS: Byte = 0xAC;
pub const LDY_ABSX: Byte = 0xBC;
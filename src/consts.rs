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

// Store Accumulator
pub const STA_ZP: Byte = 0x85;
pub const STA_ZPX: Byte = 0x95;
pub const STA_ABS: Byte = 0x8D;
pub const STA_ABSX: Byte = 0x9D;
pub const STA_ABSY: Byte = 0x99;
pub const STA_INDX: Byte = 0x81;
pub const STA_INDY: Byte = 0x91;

// Store X Register
pub const STX_ZP: Byte = 0x86;
pub const STX_ZPY: Byte = 0x96;
pub const STX_ABS: Byte = 0x8E;

// Store Y Register
pub const STY_ZP: Byte = 0x84;
pub const STY_ZPX: Byte = 0x94;
pub const STY_ABS: Byte = 0x8C;

// Transfer Accumulator to X
pub const TAX: Byte = 0xAA;

// Transfer Accumulator to Y
pub const TAY: Byte = 0xA8;

// Transfer X to Accumulator
pub const TXA: Byte = 0x8A;

// Transfer Y to Accumulator
pub const TYA: Byte = 0x98;
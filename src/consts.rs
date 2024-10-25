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

// Transfer Stack Pointer to X
pub const TSX: Byte = 0xBA;

// Transfer X to Stack Pointer
pub const TXS: Byte = 0x9A;

// Push Accumulator
pub const PHA: Byte = 0x48;

// Push Processor Status
pub const PHP: Byte = 0x08;

// Pull Accumulator
pub const PLA: Byte = 0x68;

// Pull Processor Status
pub const PLP: Byte = 0x28;

// Logical AND
pub const AND_IM: Byte = 0x29;
pub const AND_ZP: Byte = 0x25;
pub const AND_ZPX: Byte = 0x35;
pub const AND_ABS: Byte = 0x2D;
pub const AND_ABSX: Byte = 0x3D;
pub const AND_ABSY: Byte = 0x39;
pub const AND_INDX: Byte = 0x21;
pub const AND_INDY: Byte = 0x31;

// Exclusive OR
pub const EOR_IM: Byte = 0x49;
pub const EOR_ZP: Byte = 0x45;
pub const EOR_ZPX: Byte = 0x55;
pub const EOR_ABS: Byte = 0x4D;
pub const EOR_ABSX: Byte = 0x5D;
pub const EOR_ABSY: Byte = 0x59;
pub const EOR_INDX: Byte = 0x41;
pub const EOR_INDY: Byte = 0x51;

// Logical Inclusive OR
pub const ORA_IM: Byte = 0x09;
pub const ORA_ZP: Byte = 0x05;
pub const ORA_ZPX: Byte = 0x15;
pub const ORA_ABS: Byte = 0x0D;
pub const ORA_ABSX: Byte = 0x1D;
pub const ORA_ABSY: Byte = 0x19;
pub const ORA_INDX: Byte = 0x01;
pub const ORA_INDY: Byte = 0x11;

// Bit Test
pub const BIT_ZP: Byte = 0x24;
pub const BIT_ABS: Byte = 0x2C;
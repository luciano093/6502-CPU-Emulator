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

// Add with Carry
pub const ADC_IM: Byte = 0x69;
pub const ADC_ZP: Byte = 0x65;
pub const ADC_ZPX: Byte = 0x75;
pub const ADC_ABS: Byte = 0x6D;
pub const ADC_ABSX: Byte = 0x7D;
pub const ADC_ABSY: Byte = 0x79;
pub const ADC_INDX: Byte = 0x61;
pub const ADC_INDY: Byte = 0x71;

// Substract with Carry
pub const SBC_IM: Byte = 0xE9;
pub const SBC_ZP: Byte = 0xE5;
pub const SBC_ZPX: Byte = 0xF5;
pub const SBC_ABS: Byte = 0xED;
pub const SBC_ABSX: Byte = 0xFD;
pub const SBC_ABSY: Byte = 0xF9;
pub const SBC_INDX: Byte = 0xE1;
pub const SBC_INDY: Byte = 0xF1;

// Compare
pub const CMP_IM: Byte = 0xC9;
pub const CMP_ZP: Byte = 0xC5;
pub const CMP_ZPX: Byte = 0xD5;
pub const CMP_ABS: Byte = 0xCD;
pub const CMP_ABSX: Byte = 0xDD;
pub const CMP_ABSY: Byte = 0xD9;
pub const CMP_INDX: Byte = 0xC1;
pub const CMP_INDY: Byte = 0xD1;

// Compare X Register
pub const CPX_IM: Byte = 0xE0;
pub const CPX_ZP: Byte = 0xE4;
pub const CPX_ABS: Byte = 0xEC;

// Compare Y Register
pub const CPY_IM: Byte = 0xC0;
pub const CPY_ZP: Byte = 0xC4;
pub const CPY_ABS: Byte = 0xCC;

// Increment Memory
pub const INC_ZP: Byte = 0xE6;
pub const INC_ZPX: Byte = 0xF6;
pub const INC_ABS: Byte = 0xEE;
pub const INC_ABSX: Byte = 0xFE;

// Increment X Register
pub const INX: Byte = 0xE8;

// Increment Y Register
pub const INY: Byte = 0xC8;

// Decrement Memory
pub const DEC_ZP: Byte = 0xC6;
pub const DEC_ZPX: Byte = 0xD6;
pub const DEC_ABS: Byte = 0xCE;
pub const DEC_ABSX: Byte = 0xDE;

// Decrement X Register
pub const DEX: Byte = 0xCA;

// Decrement Y Register
pub const DEY: Byte = 0x88;

// Arithmetic Shift Left
pub const ASL_A: Byte = 0x0A;
pub const ASL_ZP: Byte = 0x06;
pub const ASL_ZPX: Byte = 0x16;
pub const ASL_ABS: Byte = 0x0E;
pub const ASL_ABSX: Byte = 0x1E;

// Logical Shift Right
pub const LSR_A: Byte = 0x4A;
pub const LSR_ZP: Byte = 0x46;
pub const LSR_ZPX: Byte = 0x56;
pub const LSR_ABS: Byte = 0x4E;
pub const LSR_ABSX: Byte = 0x5E;

// Rotate Left
pub const ROL_A: Byte = 0x2A;
pub const ROL_ZP: Byte = 0x26;
pub const ROL_ZPX: Byte = 0x36;
pub const ROL_ABS: Byte = 0x2E;
pub const ROL_ABSX: Byte = 0x3E;

// Rotate Right
pub const ROR_A: Byte = 0x6A;
pub const ROR_ZP: Byte = 0x66;
pub const ROR_ZPX: Byte = 0x76;
pub const ROR_ABS: Byte = 0x6E;
pub const ROR_ABSX: Byte = 0x7E;

// Jump
pub const JMP_ABS: Byte = 0x4C;
pub const JMP_IND: Byte = 0x6C;

// Jump to Subroutine
pub const JSR: Byte = 0x20;

// Return from Subroutine
pub const RTS: Byte = 0x60;
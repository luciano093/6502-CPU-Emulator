use crate::Byte;

pub const LDA_IM: Byte = 0xA9;
pub const LDA_ZP: Byte = 0xA5;
pub const LDA_ZPX: Byte = 0xB5;
pub const LDA_ABS: Byte = 0xAD;
pub const LDA_ABSX: Byte = 0xBD;
pub const LDA_ABSY: Byte = 0xB9;
pub const LDA_INDX: Byte = 0xA1;
pub const LDA_INDY: Byte = 0xB1;
use bitflags::bitflags;
use crate::consts::LDA_INDY;
use crate::{Byte, Word};
use crate::memory::Memory;
use crate::consts::*;

bitflags! {
    #[derive(Default, Debug)]
    pub struct Status: u8 {
        const C = 0b00000001; // Carry Flag
        const Z = 0b00000010; // Zero Flag
        const I = 0b00000100; // Interruptor Disable Flag
        const D = 0b00001000; // Decimal Mode Flag
        const B = 0b00010000; // Break Command Flag
        const V = 0b01000000; // Overflow Flag
        const N = 0b10000000; // Negative Flag
    }
}

#[derive(Default)]
pub struct CPU {
    pub pc: Word,    // Program Counter
    pub sp: Byte,   // Stack Pointer
    pub a: Byte,    // Accumulator
    pub x: Byte,    // Index Register X
    pub y: Byte,    // Index Register Y
    pub p: Status,  // Processor Status
}

impl CPU {
    pub fn reset(&mut self) {
        self.pc = 0xFFFC;
        self.sp = 0x0000; // starts at 0x0100 in stack
    }

    /// takes 1 cycle
    fn fetch_byte(&mut self, cycles: &mut u32, memory: &mut Memory) -> Byte {
        let byte = memory[self.pc];
        self.pc += 1;
        *cycles -= 1;
        
        byte
    }

    /// takes 2 cycles
    fn fetch_word(&mut self, cycles: &mut u32, memory: &mut Memory) -> Word {
        let low_byte = memory[self.pc];
        self.pc += 1;
        *cycles -= 1;

        let high_byte = memory[self.pc];
        self.pc += 1;
        *cycles -= 1;

        // little endian
        let word = ((high_byte as u16) << 8) | low_byte as u16;
        
        word
    }

    /// `effective_address` refers to the physical memory location\
    /// takes 1 cycle
    fn read_memory(&mut self, cycles: &mut u32, memory: &mut Memory, effective_address: u16) -> Byte {
        let byte= memory[effective_address];
        *cycles -= 1;

        byte
    }

    /// `effective_address` refers to the physical memory location\
    /// takes 2 cycles
    fn read_word_memory(&mut self, cycles: &mut u32, memory: &mut Memory, effective_address: u16) -> Word {
        let low_byte = self.read_memory(cycles, memory, effective_address as u16);

        // todo: fix what happens if high byte is at effective address greater than allowed
        let high_byte = self.read_memory(cycles, memory, effective_address as u16 + 1);

        (low_byte as u16) | ((high_byte as u16) << 8)
    }

    fn fetch_zero_page(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address= self.fetch_byte(cycles, memory);
        self.read_memory(cycles, memory, address as u16)
    }

    fn fetch_zero_page_x(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address= self.fetch_byte(cycles, memory);
        let effective_address = (self.x as u16 + address as u16) % 256; // % 256 wraps around so that the max is a byte
        *cycles -= 1;

        self.read_memory(cycles, memory, effective_address)
    }

    fn fetch_zero_page_y(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address= self.fetch_byte(cycles, memory);
        let effective_address = (self.y as u16 + address as u16) % 256; // % 256 wraps around so that the max is a byte
        *cycles -= 1;

        self.read_memory(cycles, memory, effective_address)
    }

    fn fetch_absolute(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address = self.fetch_word(cycles, memory);
        self.read_memory(cycles, memory, address)
    }

    fn fetch_absolute_x(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address = self.fetch_word(cycles, memory);

        let effective_address = self.x as u16 + address;

        // checks if page was crossed (high byte of word are the same)
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        self.read_memory(cycles, memory, effective_address)
    }

    fn fetch_absolute_y(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address = self.fetch_word(cycles, memory);

        let effective_address = self.y as u16 + address;

        // checks if page was crossed (high byte of word are the same)
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        self.read_memory(cycles, memory, effective_address)
    }

    fn fetch_indirect_x(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address = self.fetch_byte(cycles, memory);

        let effective_address = address.wrapping_add(self.x);
        *cycles -= 1;

        let effective_address = self.read_word_memory(cycles, memory, effective_address as u16);

        self.read_memory(cycles, memory, effective_address)
    }

    fn fetch_indirect_y(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let effective_address = self.fetch_byte(cycles, memory);

        let address = self.read_word_memory(cycles, memory, effective_address as u16);
        let effective_address = address + self.y as u16;
        
        // crosses a page
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        self.read_memory(cycles, memory, effective_address)
    }

    pub fn execute(&mut self, mut cycles: u32, memory: &mut Memory) {
        while cycles > 0 {
            let instruction = self.fetch_byte(&mut cycles, memory);

            match instruction {
                LDA_IM => {
                    self.a = self.fetch_byte(&mut cycles, memory);

                    self.set_lda_flags();
                }
                LDA_ZP => {
                    self.a = self.fetch_zero_page(&mut cycles, memory);

                    self.set_lda_flags();
                }
                LDA_ZPX => {
                    self.a = self.fetch_zero_page_x(&mut cycles, memory);

                    self.set_lda_flags();
                }
                LDA_ABS => {
                    self.a = self.fetch_absolute(&mut cycles, memory);

                    self.set_lda_flags();
                }
                LDA_ABSX => {
                    self.a = self.fetch_absolute_x(&mut cycles, memory);

                    self.set_lda_flags();
                }
                LDA_ABSY => {
                    self.a = self.fetch_absolute_y(&mut cycles, memory);

                    self.set_lda_flags();
                }
                LDA_INDX => {
                    self.a = self.fetch_indirect_x(&mut cycles, memory);

                    self.set_lda_flags();
                }
                LDA_INDY => {
                    self.a = self.fetch_indirect_y(&mut cycles, memory);

                    self.set_lda_flags();
                }
                LDX_IM => {
                    self.x = self.fetch_byte(&mut cycles, memory);

                    self.set_ldx_flags();
                }
                LDX_ZP => {
                    self.x = self.fetch_zero_page(&mut cycles, memory);

                    self.set_ldx_flags();
                }
                LDX_ZPY => {
                    self.x = self.fetch_zero_page_y(&mut cycles, memory);

                    self.set_ldx_flags();
                }
                LDX_ABS => {
                    self.x = self.fetch_absolute(&mut cycles, memory);

                    self.set_ldx_flags();
                }
                LDX_ABSY => {
                    self.x = self.fetch_absolute_y(&mut cycles, memory);

                    self.set_ldx_flags();
                }
                LDY_IM => {
                    self.y = self.fetch_byte(&mut cycles, memory);

                    self.set_ldy_flags();
                }
                LDY_ZP => {
                    self.y = self.fetch_zero_page(&mut cycles, memory);

                    self.set_ldy_flags();
                }
                LDY_ZPX => {
                    self.y = self.fetch_zero_page_x(&mut cycles, memory);

                    self.set_ldy_flags();
                }
                LDY_ABS => {
                    self.y = self.fetch_absolute(&mut cycles, memory);

                    self.set_ldy_flags();
                }
                LDY_ABSX => {
                    self.y = self.fetch_absolute_x(&mut cycles, memory);

                    self.set_ldy_flags();
                }
                _ => panic!("Tried to execute unknown instruction"),
            }
        }
    }

    fn set_lda_flags(&mut self) {
        if self.a == 0 {
            self.p |= Status::from_bits(0b00000010).unwrap();
        }

        if self.a & 0b10000000 == 0b10000000 {
            self.p |= Status::from_bits(0b10000000).unwrap();
        }  
    }

    fn set_ldx_flags(&mut self) {
        if self.x == 0 {
            self.p |= Status::from_bits(0b00000010).unwrap();
        }

        if self.x & 0b10000000 == 0b10000000 {
            self.p |= Status::from_bits(0b10000000).unwrap();
        } 
    }

    fn set_ldy_flags(&mut self) {
        if self.y == 0 {
            self.p |= Status::from_bits(0b00000010).unwrap();
        }

        if self.y & 0b10000000 == 0b10000000 {
            self.p |= Status::from_bits(0b10000000).unwrap();
        } 
    }
}
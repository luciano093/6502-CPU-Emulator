use bitflags::bitflags;
use crate::consts::LDA_INDY;
use crate::{Byte, Word};
use crate::memory::Memory;
use crate::consts::*;

bitflags! {
    // bit 5 is unused
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

    fn zero_page_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        self.fetch_byte(cycles, memory)
    }

    fn zero_page_x_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address= self.fetch_byte(cycles, memory);
        let effective_address = (self.x as u16 + address as u16) % 256; // % 256 wraps around so that the max is a byte
        *cycles -= 1;

        effective_address as u8
    }

    fn zero_page_y_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address= self.fetch_byte(cycles, memory);
        let effective_address = (self.y as u16 + address as u16) % 256; // % 256 wraps around so that the max is a byte
        *cycles -= 1;

        effective_address as u8
    }

    fn absolute_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u16 {
        self.fetch_word(cycles, memory)
    }

    fn absolute_x_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u16 {
        let address = self.fetch_word(cycles, memory);

        let effective_address = self.x as u16 + address;

        // checks if page was crossed (high byte of word are the same)
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        effective_address
    }

    fn absolute_y_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u16 {
        let address = self.fetch_word(cycles, memory);

        let effective_address = self.y as u16 + address;

        // checks if page was crossed (high byte of word are the same)
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        effective_address
    }

    fn indirect_x_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u16 {
        let address = self.fetch_byte(cycles, memory);

        let effective_address = address.wrapping_add(self.x);
        *cycles -= 1;

        self.read_word_memory(cycles, memory, effective_address as u16)
    }

    fn indirect_y_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u16 {
        let effective_address = self.fetch_byte(cycles, memory);

        let address = self.read_word_memory(cycles, memory, effective_address as u16);
        let effective_address = address + self.y as u16;
        
        // crosses a page
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        effective_address
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
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    self.a = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_lda_flags();
                }
                LDA_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);
                    self.a = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_lda_flags();
                }
                LDA_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    self.a = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_lda_flags();
                }
                LDA_ABSX => {
                    let effective_address = self.absolute_x_addressing(&mut cycles, memory);
                    self.a = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_lda_flags();
                }
                LDA_ABSY => {
                    let effective_address = self.absolute_y_addressing(&mut cycles, memory);
                    self.a = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_lda_flags();
                }
                LDA_INDX => {
                    let effective_address = self.indirect_x_addressing(&mut cycles, memory);
                    self.a = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_lda_flags();
                }
                LDA_INDY => {
                    let effective_address = self.indirect_y_addressing(&mut cycles, memory);
                    self.a = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_lda_flags();
                }
                LDX_IM => {
                    self.x = self.fetch_byte(&mut cycles, memory);

                    self.set_ldx_flags();
                }
                LDX_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    self.x = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_ldx_flags();
                }
                LDX_ZPY => {
                    let effective_address = self.zero_page_y_addressing(&mut cycles, memory);
                    self.x = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_ldx_flags();
                }
                LDX_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    self.x = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_ldx_flags();
                }
                LDX_ABSY => {
                    let effective_address = self.absolute_y_addressing(&mut cycles, memory);
                    self.x = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_ldx_flags();
                }
                LDY_IM => {
                    self.y = self.fetch_byte(&mut cycles, memory);

                    self.set_ldy_flags();
                }
                LDY_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    self.y = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_ldy_flags();
                }
                LDY_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);
                    self.y = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_ldy_flags();
                }
                LDY_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    self.y = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_ldy_flags();
                }
                LDY_ABSX => {
                    let effective_address = self.absolute_x_addressing(&mut cycles, memory);
                    self.y = self.read_memory(&mut cycles, memory, effective_address as u16);

                    self.set_ldy_flags();
                }
                STA_ZP => {
                    let effective_address= self.zero_page_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.a;
                }
                STA_ZPX => {
                    let effective_address= self.zero_page_x_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.a;
                }
                STA_ABS => {
                    let effective_address= self.absolute_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.a;
                }
                STA_ABSX => {
                    let effective_address= self.absolute_x_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.a;
                }
                STA_ABSY => {
                    let effective_address= self.absolute_y_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.a;
                }
                STA_INDX => {
                    let effective_address= self.indirect_x_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.a;
                }
                STA_INDY => {
                    let effective_address= self.indirect_y_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.a;
                }
                STX_ZP => {
                    let effective_address= self.zero_page_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.x;
                }
                STX_ZPY => {
                    let effective_address= self.zero_page_y_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.x;
                }
                STX_ABS => {
                    let effective_address= self.absolute_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.x;
                }
                STY_ZP => {
                    let effective_address= self.zero_page_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.y;
                }
                STY_ZPX => {
                    let effective_address= self.zero_page_y_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.y;
                }
                STY_ABS => {
                    let effective_address= self.absolute_addressing(&mut cycles, memory);

                    memory[effective_address as u16] = self.y;
                }
                TAX => {
                    self.x = self.a;
                    cycles -= 1;

                    if self.x == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.x & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }  
                }
                TAY => {
                    self.y = self.a;
                    cycles -= 1;

                    if self.y == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.y & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }  
                }
                TXA => {
                    self.a = self.x;
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }  
                }
                TYA => {
                    self.a = self.y;
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }  
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
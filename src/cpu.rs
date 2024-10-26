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
    pub pc: Word,   // Program Counter
    pub sp: Byte,   // Stack Pointer
    pub a: Byte,    // Accumulator
    pub x: Byte,    // Index Register X
    pub y: Byte,    // Index Register Y
    pub p: Status,  // Processor Status
}

impl CPU {
    pub fn reset(&mut self, memory: &Memory) {
        self.pc = memory[0xFFFC] as u16 | ((memory[0xFFFD] as u16) << 8);
        self.sp = 0xFF; // goes between 0x0100 and 0x1FF in stack
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

    /// takes 1 cycle
    fn zero_page_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        self.fetch_byte(cycles, memory)
    }

    /// takes 2 cycles
    fn zero_page_x_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address= self.fetch_byte(cycles, memory);
        let effective_address = (self.x as u16 + address as u16) % 256; // % 256 wraps around so that the max is a byte
        *cycles -= 1;

        effective_address as u8
    }

    /// takes 2 cycles
    fn zero_page_y_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u8 {
        let address= self.fetch_byte(cycles, memory);
        let effective_address = (self.y as u16 + address as u16) % 256; // % 256 wraps around so that the max is a byte
        *cycles -= 1;

        effective_address as u8
    }

    /// takes 2 cycles
    fn absolute_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u16 {
        self.fetch_word(cycles, memory)
    }

    /// takes 2-3 cycles depending on if page was crossed
    fn absolute_x_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u16 {
        let address = self.fetch_word(cycles, memory);

        let effective_address = self.x as u16 + address;

        // checks if page was crossed (high byte of word are the same)
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        effective_address
    }

    /// takes 2-3 cycles depending on if page was crossed
    fn absolute_y_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u16 {
        let address = self.fetch_word(cycles, memory);

        let effective_address = self.y as u16 + address;

        // checks if page was crossed (high byte of word are the same)
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        effective_address
    }

    /// takes 4 cycles
    fn indirect_x_addressing(&mut self, cycles: &mut u32, memory: &mut Memory) -> u16 {
        let address = self.fetch_byte(cycles, memory);

        let effective_address = address.wrapping_add(self.x);
        *cycles -= 1;

        self.read_word_memory(cycles, memory, effective_address as u16)
    }

    /// takes 3-4 cycles depending on if page was crossed
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
                TSX => {
                    self.x = self.sp;
                    cycles -= 1;

                    if self.x == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.x & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }  
                }
                TXS => {
                    self.sp = self.x;
                    cycles -= 1;
                }
                PHA => {
                    // Discarded OP CODE (due to cpu design) that will be used on next cycle
                    cycles -= 1;

                    memory[self.sp as u16] = self.a;
                    self.sp -= 1;
                    cycles -= 1;
                }
                PHP => {
                    // Discarded OP CODE (due to cpu design) that will be used on next cycle
                    cycles -= 1;

                    memory[self.sp as u16] = self.p.bits();
                    self.sp -= 1;
                    cycles -= 1;
                }
                PLA => {
                    // Discarded OP CODE (due to cpu design) that will be used on next cycle
                    cycles -= 1;

                    // Discarded Stack Pointer Fetch (due to cpu design)
                    cycles -= 1;

                    self.sp += 1;
                    self.a = memory[self.sp as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }  
                }
                PLP => {
                    // Discarded OP CODE (due to cpu design) that will be used on next cycle
                    cycles -= 1;

                    // Discarded Stack Pointer Fetch (due to cpu design)
                    cycles -= 1;

                    self.sp += 1;
                    self.p = Status::from_bits(memory[self.sp as u16]).unwrap();
                    cycles -= 1;
                }
                AND_IM => {
                    self.a &= self.fetch_byte(&mut cycles, memory);
                
                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }  
                }
                AND_ZP => {
                    let effectve_address = self.zero_page_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                AND_ZPX => {
                    let effectve_address = self.zero_page_x_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                AND_ABS => {
                    let effectve_address = self.absolute_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                AND_ABSX => {
                    let effectve_address = self.absolute_x_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                AND_ABSY => {
                    let effectve_address = self.absolute_y_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                AND_INDX => {
                    let effectve_address = self.indirect_x_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                AND_INDY => {
                    let effectve_address = self.indirect_y_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                EOR_IM => {
                    self.a ^= self.fetch_byte(&mut cycles, memory);
                
                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }  
                }
                EOR_ZP => {
                    let effectve_address = self.zero_page_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                EOR_ZPX => {
                    let effectve_address = self.zero_page_x_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                EOR_ABS => {
                    let effectve_address = self.absolute_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                EOR_ABSX => {
                    let effectve_address = self.absolute_x_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                EOR_ABSY => {
                    let effectve_address = self.absolute_y_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                EOR_INDX => {
                    let effectve_address = self.indirect_x_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                EOR_INDY => {
                    let effectve_address = self.indirect_y_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                ORA_IM => {
                    self.a |= self.fetch_byte(&mut cycles, memory);
                
                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }  
                }
                ORA_ZP => {
                    let effectve_address = self.zero_page_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                ORA_ZPX => {
                    let effectve_address = self.zero_page_x_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                ORA_ABS => {
                    let effectve_address = self.absolute_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                ORA_ABSX => {
                    let effectve_address = self.absolute_x_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                ORA_ABSY => {
                    let effectve_address = self.absolute_y_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                ORA_INDX => {
                    let effectve_address = self.indirect_x_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                ORA_INDY => {
                    let effectve_address = self.indirect_y_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as u16];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                BIT_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let bit_test = self.a & memory[effective_address as u16];
                    cycles -= 1;
                    
                    if bit_test == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    self.p &= Status::from_bits(bit_test & 0b11000000).unwrap();
                }
                BIT_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let bit_test = self.a & memory[effective_address];
                    
                    if bit_test == 0 {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    self.p &= Status::from_bits(bit_test & 0b11000000).unwrap();
                }
                ADC_IM => {
                    let byte = self.fetch_byte(&mut cycles, memory);

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_ABSX => {
                    let effective_address = self.absolute_x_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_ABSY => {
                    let effective_address = self.absolute_y_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_INDX => {
                    let effective_address = self.indirect_x_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_INDY => {
                    let effective_address = self.indirect_y_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                // Same as ADC but with bit negation on the byte from memory
                SBC_IM => {
                    let byte = !self.fetch_byte(&mut cycles, memory);

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_ABSX => {
                    let effective_address = self.absolute_x_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_ABSY => {
                    let effective_address = self.absolute_y_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_INDX => {
                    let effective_address = self.indirect_x_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_INDY => {
                    let effective_address = self.indirect_y_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as u16];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if (self.p.bits() & 0b00000001) == 0b00000001 {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                CMP_IM => {
                    let byte = self.fetch_byte(&mut cycles, memory);

                    if self.a >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.a == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CMP_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.a >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.a == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CMP_ZPX => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.a >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.a == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CMP_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.a >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.a == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CMP_ABSX => {
                    let effective_address = self.absolute_x_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.a >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.a == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CMP_ABSY => {
                    let effective_address = self.absolute_y_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.a >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.a == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CMP_INDX => {
                    let effective_address = self.indirect_x_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.a >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.a == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CMP_INDY => {
                    let effective_address = self.indirect_y_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.a >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.a == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CPX_IM => {
                    let byte = self.fetch_byte(&mut cycles, memory);

                    if self.x >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.x == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.x >= byte && ((self.x - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CPX_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.x >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.x == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.x >= byte && ((self.x - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CPX_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.x >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.x == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.x >= byte && ((self.x - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CPY_IM => {
                    let byte = self.fetch_byte(&mut cycles, memory);

                    if self.y >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.y == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.y >= byte && ((self.y - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CPY_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.y >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.y == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.y >= byte && ((self.y - byte) & 0b10000000) == 0b10000000 {
                        self.p |= Status::from_bits(0b10000000).unwrap();
                    }
                }
                CPY_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as u16);

                    if self.y >= byte {
                        self.p |= Status::from_bits(0b00000001).unwrap();
                    }

                    if self.y == byte {
                        self.p |= Status::from_bits(0b00000010).unwrap();
                    }

                    if self.y >= byte && ((self.y - byte) & 0b10000000) == 0b10000000 {
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

    fn set_adc_sbc_flags(&mut self, overflow: bool, initial_value: u8) {
        if overflow {
            println!("found carry");
            // carry flag set
            self.p |= Status::from_bits(0b00000001).unwrap();
        }

        // on incorrect sign
        if (initial_value & 0b10000000) != (self.a & 0b10000000) {
            println!("found overflow");
            // overflow flag set
            self.p |= Status::from_bits(0b01000000).unwrap();
        }

        if self.a == 0 {
            // zero flag set
            self.p |= Status::from_bits(0b00000010).unwrap();
        }

        // if A has negative bit on
        if (self.a & 0b10000000) == 0b1000000 {
            // negative flag set
            self.p |= Status::from_bits(0b10000000).unwrap();

        }
    }
}
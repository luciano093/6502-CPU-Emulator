use bitflags::bitflags;
use crate::consts::LDA_INDY;
use crate::{Byte, Word};
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

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        Status::from_bits(value).unwrap_or_else(|| {
            // handle 5th bit being set
            Status::from_bits(value & 0b11011111).unwrap()
        })
    }
}

impl Status {
    pub fn set_carry(&mut self, state: bool) {
        self.set(0b00000001.into(), state);
    }

    pub const fn carry_flag(&self) -> bool {
        self.bits() & 0b00000001 == 0b00000001
    }

    pub fn set_zero(&mut self, state: bool) {
        self.set(0b00000010.into(), state);
    }

    pub const fn zero_flag(&self) -> bool {
        self.bits() & 0b00000010 == 0b00000010
    }

    pub fn set_interruptor(&mut self, state: bool) {
        self.set(0b00000100.into(), state);
    }

    pub const fn interruptor_flag(&self) -> bool {
        self.bits() & 0b00000100 == 0b00000100
    }

    pub fn set_decimal(&mut self, state: bool) {
        self.set(0b00001000.into(), state);
    }

    pub const fn decimal_flag(&self) -> bool {
        self.bits() & 0b00001000 == 0b00001000
    }

    pub fn set_break(&mut self, state: bool) {
        self.set(0b00010000.into(), state);
    }

    pub const fn break_flag(&self) -> bool {
        self.bits() & 0b00010000 == 0b00010000
    }

    pub fn set_overflow(&mut self, state: bool) {
        self.set(0b01000000.into(), state);
    }

    pub const fn overflow_flag(&self) -> bool {
        self.bits() & 0b01000000 == 0b01000000
    }

    pub fn set_negative(&mut self, state: bool) {
        self.set(0b10000000.into(), state);
    }

    pub const fn negative_flag(&self) -> bool {
        self.bits() & 0b10000000 == 0b10000000
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
    pub fn reset(&mut self, memory: &[u8]) {
        self.pc = memory[0xFFFC] as u16 | ((memory[0xFFFD] as u16) << 8);
        self.sp = 0xFF; // goes between 0x0100 and 0x1FF in stack
    }

    /// takes 1 cycle
    fn fetch_byte(&mut self, cycles: &mut u32, memory: &mut [u8]) -> Byte {
        let byte = memory[self.pc as usize];
        self.pc += 1;
        *cycles -= 1;
        
        byte
    }

    /// takes 2 cycles
    fn fetch_word(&mut self, cycles: &mut u32, memory: &mut [u8]) -> Word {
        let low_byte = memory[self.pc as usize];
        self.pc += 1;
        *cycles -= 1;

        let high_byte = memory[self.pc as usize];
        self.pc += 1;
        *cycles -= 1;

        // little endian
        let word = ((high_byte as u16) << 8) | low_byte as u16;
        
        word
    }

    /// `effective_address` refers to the physical memory location\
    /// takes 1 cycle
    fn read_memory(&mut self, cycles: &mut u32, memory: &mut [u8], effective_address: usize) -> Byte {
        let byte= memory[effective_address];
        *cycles -= 1;

        byte
    }

    /// `effective_address` refers to the physical memory location\
    /// takes 2 cycles
    fn read_word_memory(&mut self, cycles: &mut u32, memory: &mut [u8], effective_address: usize) -> Word {
        let low_byte = self.read_memory(cycles, memory, effective_address as usize);

        // todo: fix what happens if high byte is at effective address greater than allowed
        let high_byte = self.read_memory(cycles, memory, effective_address as usize + 1);

        (low_byte as u16) | ((high_byte as u16) << 8)
    }

    /// takes 1 cycle
    fn zero_page_addressing(&mut self, cycles: &mut u32, memory: &mut [u8]) -> u8 {
        self.fetch_byte(cycles, memory)
    }

    /// takes 2 cycles
    fn zero_page_x_addressing(&mut self, cycles: &mut u32, memory: &mut [u8]) -> u8 {
        let address= self.fetch_byte(cycles, memory);
        let effective_address = (self.x as u16 + address as u16) % 256; // % 256 wraps around so that the max is a byte
        *cycles -= 1;

        effective_address as u8
    }

    /// takes 2 cycles
    fn zero_page_y_addressing(&mut self, cycles: &mut u32, memory: &mut [u8]) -> u8 {
        let address= self.fetch_byte(cycles, memory);
        let effective_address = (self.y as u16 + address as u16) % 256; // % 256 wraps around so that the max is a byte
        *cycles -= 1;

        effective_address as u8
    }

    /// takes 2 cycles
    fn absolute_addressing(&mut self, cycles: &mut u32, memory: &mut [u8]) -> u16 {
        self.fetch_word(cycles, memory)
    }

    /// takes 2-3 cycles depending on if page was crossed
    fn absolute_x_addressing(&mut self, cycles: &mut u32, memory: &mut [u8]) -> u16 {
        let address = self.fetch_word(cycles, memory);

        let effective_address = self.x as u16 + address;

        // checks if page was crossed (high byte of word are the same)
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        effective_address
    }

    /// takes 2-3 cycles depending on if page was crossed
    fn absolute_y_addressing(&mut self, cycles: &mut u32, memory: &mut [u8]) -> u16 {
        let address = self.fetch_word(cycles, memory);

        let effective_address = self.y as u16 + address;

        // checks if page was crossed (high byte of word are the same)
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        effective_address
    }

    /// takes 4 cycles
    fn indirect_addressing(&mut self, cycles: &mut u32, memory: &mut [u8]) -> u16 {
        let effective_address = self.fetch_word(cycles, memory);

        let effective_address= self.read_word_memory(cycles, memory, effective_address as usize);

        effective_address
    }

    /// takes 4 cycles
    fn indirect_x_addressing(&mut self, cycles: &mut u32, memory: &mut [u8]) -> u16 {
        let address = self.fetch_byte(cycles, memory);

        let effective_address = address.wrapping_add(self.x);
        *cycles -= 1;

        self.read_word_memory(cycles, memory, effective_address as usize)
    }

    /// takes 3-4 cycles depending on if page was crossed
    fn indirect_y_addressing(&mut self, cycles: &mut u32, memory: &mut [u8]) -> u16 {
        let effective_address = self.fetch_byte(cycles, memory);

        let address = self.read_word_memory(cycles, memory, effective_address as usize);
        let effective_address = address + self.y as u16;
        
        // crosses a page
        if (address & 0xFF00) != (effective_address & 0xFF00) {
            *cycles -= 1;
        }

        effective_address
    }

    pub fn execute(&mut self, mut cycles: u32, memory: &mut [u8]) {
        while cycles > 0 {
            let instruction = self.fetch_byte(&mut cycles, memory);

            println!("instruction: {:02X}, cycles left: {}", instruction, cycles + 1);

            match instruction {
                LDA_IM => {
                    self.a = self.fetch_byte(&mut cycles, memory);
                    self.set_lda_flags();
                }
                lda_instruction @ (LDA_ZP | LDA_ZPX | LDA_ABS | LDA_ABSX | LDA_ABSY | LDA_INDX | LDA_INDY) => {
                    let effective_address = match lda_instruction {
                        LDA_ZP => self.zero_page_addressing(&mut cycles, memory) as usize,
                        LDA_ZPX => self.zero_page_x_addressing(&mut cycles, memory) as usize,
                        LDA_ABS => self.absolute_addressing(&mut cycles, memory) as usize,
                        LDA_ABSX => self.absolute_x_addressing(&mut cycles, memory) as usize,
                        LDA_ABSY => self.absolute_y_addressing(&mut cycles, memory) as usize,
                        LDA_INDX => self.indirect_x_addressing(&mut cycles, memory) as usize,
                        LDA_INDY => self.indirect_y_addressing(&mut cycles, memory) as usize,
                        _ => panic!("Unexpected LDA instruction"),
                    };

                    self.a = self.read_memory(&mut cycles, memory, effective_address);

                    self.set_lda_flags();
                }
                LDX_IM => {
                    self.x = self.fetch_byte(&mut cycles, memory);

                    self.set_ldx_flags();
                }
                LDX_ZP | LDX_ZPY | LDX_ABS | LDX_ABSY => {
                    let effective_address = match instruction {
                        LDX_ZP => self.zero_page_addressing(&mut cycles, memory) as usize,
                        LDX_ZPY => self.zero_page_y_addressing(&mut cycles, memory) as usize,
                        LDX_ABS => self.absolute_addressing(&mut cycles, memory) as usize,
                        LDX_ABSY => self.absolute_y_addressing(&mut cycles, memory) as usize,
                        _ => panic!("Unexpected LDX instruction"),
                    };
                    self.x = self.read_memory(&mut cycles, memory, effective_address);

                    self.set_ldx_flags();
                }
                LDY_IM => {
                    self.y = self.fetch_byte(&mut cycles, memory);

                    self.set_ldy_flags();
                }
                LDY_ZP | LDY_ZPX | LDY_ABS | LDY_ABSX => {
                    let effective_address = match instruction {
                        LDY_ZP => self.zero_page_addressing(&mut cycles, memory) as usize,
                        LDY_ZPX => self.zero_page_x_addressing(&mut cycles, memory) as usize,
                        LDY_ABS => self.absolute_addressing(&mut cycles, memory) as usize,
                        LDY_ABSX => self.absolute_x_addressing(&mut cycles, memory) as usize,
                        _ => panic!("Unexpected LDY instruction"),
                    };
                    self.y = self.read_memory(&mut cycles, memory, effective_address);

                    self.set_ldy_flags();
                }
                STA_ZP => {
                    let effective_address= self.zero_page_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.a;
                }
                STA_ZPX => {
                    let effective_address= self.zero_page_x_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.a;
                }
                STA_ABS => {
                    let effective_address= self.absolute_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.a;
                }
                STA_ABSX => {
                    let effective_address= self.absolute_x_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.a;
                }
                STA_ABSY => {
                    let effective_address= self.absolute_y_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.a;
                }
                STA_INDX => {
                    let effective_address= self.indirect_x_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.a;
                }
                STA_INDY => {
                    let effective_address= self.indirect_y_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.a;
                }
                STX_ZP => {
                    let effective_address= self.zero_page_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.x;
                }
                STX_ZPY => {
                    let effective_address= self.zero_page_y_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.x;
                }
                STX_ABS => {
                    let effective_address= self.absolute_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.x;
                }
                STY_ZP => {
                    let effective_address= self.zero_page_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.y;
                }
                STY_ZPX => {
                    let effective_address= self.zero_page_y_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.y;
                }
                STY_ABS => {
                    let effective_address= self.absolute_addressing(&mut cycles, memory);

                    memory[effective_address as usize] = self.y;
                }
                TAX => {
                    self.x = self.a;
                    cycles -= 1;

                    if self.x == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.x & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }  
                }
                TAY => {
                    self.y = self.a;
                    cycles -= 1;

                    if self.y == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.y & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }  
                }
                TXA => {
                    self.a = self.x;
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }  
                }
                TYA => {
                    self.a = self.y;
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }  
                }
                TSX => {
                    self.x = self.sp;
                    cycles -= 1;

                    if self.x == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.x & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }  
                }
                TXS => {
                    self.sp = self.x;
                    cycles -= 1;
                }
                PHA => {
                    // Discarded OP CODE (due to cpu design) that will be used on next cycle
                    cycles -= 1;

                    memory[self.sp as usize] = self.a;
                    self.sp -= 1;
                    cycles -= 1;
                }
                PHP => {
                    // Discarded OP CODE (due to cpu design) that will be used on next cycle
                    cycles -= 1;

                    memory[self.sp as usize] = self.p.bits();
                    self.sp -= 1;
                    cycles -= 1;
                }
                PLA => {
                    // Discarded OP CODE (due to cpu design) that will be used on next cycle
                    cycles -= 1;

                    // Discarded Stack Pointer Fetch (due to cpu design)
                    cycles -= 1;

                    self.sp += 1;
                    self.a = memory[self.sp as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }  
                }
                PLP => {
                    // Discarded OP CODE (due to cpu design) that will be used on next cycle
                    cycles -= 1;

                    // Discarded Stack Pointer Fetch (due to cpu design)
                    cycles -= 1;

                    self.sp += 1;
                    self.p = Status::from_bits(memory[self.sp as usize]).unwrap();
                    cycles -= 1;
                }
                AND_IM => {
                    self.a &= self.fetch_byte(&mut cycles, memory);
                
                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }  
                }
                AND_ZP => {
                    let effectve_address = self.zero_page_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                AND_ZPX => {
                    let effectve_address = self.zero_page_x_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                AND_ABS => {
                    let effectve_address = self.absolute_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                AND_ABSX => {
                    let effectve_address = self.absolute_x_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                AND_ABSY => {
                    let effectve_address = self.absolute_y_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                AND_INDX => {
                    let effectve_address = self.indirect_x_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                AND_INDY => {
                    let effectve_address = self.indirect_y_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                EOR_IM => {
                    self.a ^= self.fetch_byte(&mut cycles, memory);
                
                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }  
                }
                EOR_ZP => {
                    let effectve_address = self.zero_page_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                EOR_ZPX => {
                    let effectve_address = self.zero_page_x_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                EOR_ABS => {
                    let effectve_address = self.absolute_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                EOR_ABSX => {
                    let effectve_address = self.absolute_x_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                EOR_ABSY => {
                    let effectve_address = self.absolute_y_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                EOR_INDX => {
                    let effectve_address = self.indirect_x_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                EOR_INDY => {
                    let effectve_address = self.indirect_y_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ORA_IM => {
                    self.a |= self.fetch_byte(&mut cycles, memory);
                
                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }  
                }
                ORA_ZP => {
                    let effectve_address = self.zero_page_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ORA_ZPX => {
                    let effectve_address = self.zero_page_x_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ORA_ABS => {
                    let effectve_address = self.absolute_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ORA_ABSX => {
                    let effectve_address = self.absolute_x_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ORA_ABSY => {
                    let effectve_address = self.absolute_y_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ORA_INDX => {
                    let effectve_address = self.indirect_x_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ORA_INDY => {
                    let effectve_address = self.indirect_y_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }
            
                    if self.a & 0b10000000 == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                BIT_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let bit_test = self.a & memory[effective_address as usize];
                    cycles -= 1;
                    
                    if bit_test == 0 {
                        self.p.set_zero(true);
                    }

                    self.p &= Status::from_bits(bit_test & 0b11000000).unwrap();
                }
                BIT_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let bit_test = self.a & memory[effective_address as usize];
                    
                    if bit_test == 0 {
                        self.p.set_zero(true);
                    }

                    self.p &= Status::from_bits(bit_test & 0b11000000).unwrap();
                }
                ADC_IM => {
                    let byte = self.fetch_byte(&mut cycles, memory);

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_ABSX => {
                    let effective_address = self.absolute_x_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_ABSY => {
                    let effective_address = self.absolute_y_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_INDX => {
                    let effective_address = self.indirect_x_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                ADC_INDY => {
                    let effective_address = self.indirect_y_addressing(&mut cycles, memory);
                    let byte = memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
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

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_ABSX => {
                    let effective_address = self.absolute_x_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_ABSY => {
                    let effective_address = self.absolute_y_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_INDX => {
                    let effective_address = self.indirect_x_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
                        let (new_a, carry_overflow) = a.overflowing_add(1);

                        a = new_a;
                        a_overflow |= carry_overflow;
                    }

                    self.a = a;

                    self.set_adc_sbc_flags(a_overflow, byte);
                }
                SBC_INDY => {
                    let effective_address = self.indirect_y_addressing(&mut cycles, memory);
                    let byte = !memory[effective_address as usize];
                    cycles -= 1;

                    let (mut a, mut a_overflow) = self.a.overflowing_add(byte);

                    if self.p.carry_flag() {
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
                        self.p.set_carry(true);
                    }

                    if self.a == byte {
                        self.p.set_zero(true);
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CMP_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.a >= byte {
                        self.p.set_carry(true);
                    }

                    if self.a == byte {
                        self.p.set_zero(true);
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CMP_ZPX => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.a >= byte {
                        self.p.set_carry(true);
                    }

                    if self.a == byte {
                        self.p.set_zero(true);
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CMP_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.a >= byte {
                        self.p.set_carry(true);
                    }

                    if self.a == byte {
                        self.p.set_zero(true);
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CMP_ABSX => {
                    let effective_address = self.absolute_x_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.a >= byte {
                        self.p.set_carry(true);
                    }

                    if self.a == byte {
                        self.p.set_zero(true);
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CMP_ABSY => {
                    let effective_address = self.absolute_y_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.a >= byte {
                        self.p.set_carry(true);
                    }

                    if self.a == byte {
                        self.p.set_zero(true);
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CMP_INDX => {
                    let effective_address = self.indirect_x_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.a >= byte {
                        self.p.set_carry(true);
                    }

                    if self.a == byte {
                        self.p.set_zero(true);
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CMP_INDY => {
                    let effective_address = self.indirect_y_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.a >= byte {
                        self.p.set_carry(true);
                    }

                    if self.a == byte {
                        self.p.set_zero(true);
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CPX_IM => {
                    let byte = self.fetch_byte(&mut cycles, memory);

                    if self.x >= byte {
                        self.p.set_carry(true);
                    }

                    if self.x == byte {
                        self.p.set_zero(true);
                    }

                    if self.x >= byte && ((self.x - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CPX_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.x >= byte {
                        self.p.set_carry(true);
                    }

                    if self.x == byte {
                        self.p.set_zero(true);
                    }

                    if self.x >= byte && ((self.x - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CPX_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.x >= byte {
                        self.p.set_carry(true);
                    }

                    if self.x == byte {
                        self.p.set_zero(true);
                    }

                    if self.x >= byte && ((self.x - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CPY_IM => {
                    let byte = self.fetch_byte(&mut cycles, memory);

                    if self.y >= byte {
                        self.p.set_carry(true);
                    }

                    if self.y == byte {
                        self.p.set_zero(true);
                    }

                    if self.y >= byte && ((self.y - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CPY_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.y >= byte {
                        self.p.set_carry(true);
                    }

                    if self.y == byte {
                        self.p.set_zero(true);
                    }

                    if self.y >= byte && ((self.y - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CPY_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    if self.y >= byte {
                        self.p.set_carry(true);
                    }

                    if self.y == byte {
                        self.p.set_zero(true);
                    }

                    if self.y >= byte && ((self.y - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                INC_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);

                    // Fetch data
                    let mut data = memory[effective_address as usize];
                    cycles -= 1;

                    // Add
                    data += 1;
                    cycles -= 1;

                    // Write modified data back to memory cycle
                    memory[effective_address as usize] = data;
                    cycles -= 1;

                    if data == 0 {
                        self.p.set_zero(true);
                    }

                    if (data & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                INC_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);

                    // Fetch data
                    let mut data = memory[effective_address as usize];
                    cycles -= 1;

                    // Add
                    data += 1;
                    cycles -= 1;

                    // Write modified data back to memory cycle
                    memory[effective_address as usize] = data;
                    cycles -= 1;

                    if data == 0 {
                        self.p.set_zero(true);
                    }

                    if (data & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                INC_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);

                    // Fetch data
                    let mut data = memory[effective_address as usize];
                    cycles -= 1;

                    // Add
                    data += 1;
                    cycles -= 1;

                    // Write modified data back to memory cycle
                    memory[effective_address as usize] = data;
                    cycles -= 1;

                    if data == 0 {
                        self.p.set_zero(true);
                    }

                    if (data & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                INC_ABSX => {
                    let address = self.fetch_word(&mut cycles, memory);

                    let effective_address = self.x as u16 + address;

                    // Discarded Data
                    cycles -= 1;

                    // Fetch data
                    let mut data = memory[effective_address as usize];
                    cycles -= 1;

                    // Add
                    data += 1;
                    cycles -= 1;

                    // Write modified data back to memory cycle
                    memory[effective_address as usize] = data;
                    cycles -= 1;

                    if data == 0 {
                        self.p.set_zero(true);
                    }

                    if (data & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                INX => {
                    self.x += 1;
                    cycles -= 1;

                    if self.x == 0 {
                        self.p.set_zero(true);
                    }

                    if (self.x & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                INY => {
                    self.y += 1;
                    cycles -= 1;

                    if self.y == 0 {
                        self.p.set_zero(true);
                    }

                    if (self.y & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                DEC_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);

                    // Fetch data
                    let mut data = memory[effective_address as usize];
                    cycles -= 1;

                    // Subtract
                    data -= 1;
                    cycles -= 1;

                    // Write modified data back to memory cycle
                    memory[effective_address as usize] = data;
                    cycles -= 1;

                    if data == 0 {
                        self.p.set_zero(true);
                    }

                    if (data & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                DEC_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);

                    // Fetch data
                    let mut data = memory[effective_address as usize];
                    cycles -= 1;

                    // Subtract
                    data -= 1;
                    cycles -= 1;

                    // Write modified data back to memory cycle
                    memory[effective_address as usize] = data;
                    cycles -= 1;

                    if data == 0 {
                        self.p.set_zero(true);
                    }

                    if (data & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                DEC_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);

                    // Fetch data
                    let mut data = memory[effective_address as usize];
                    cycles -= 1;

                    // Subtract
                    data -= 1;
                    cycles -= 1;

                    // Write modified data back to memory cycle
                    memory[effective_address as usize] = data;
                    cycles -= 1;

                    if data == 0 {
                        self.p.set_zero(true);
                    }

                    if (data & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                DEC_ABSX => {
                    let address = self.fetch_word(&mut cycles, memory);

                    let effective_address = self.x as u16 + address;

                    // Discarded Data
                    cycles -= 1;

                    // Fetch data
                    let mut data = memory[effective_address as usize];
                    cycles -= 1;

                    // Subtract
                    data -= 1;
                    cycles -= 1;

                    // Write modified data back to memory cycle
                    memory[effective_address as usize] = data;
                    cycles -= 1;

                    if data == 0 {
                        self.p.set_zero(true);
                    }

                    if (data & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                DEX => {
                    self.x -= 1;

                    if self.x == 0 {
                        self.p.set_zero(true);
                    }

                    if (self.x & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                DEY => {
                    self.y -= 1;

                    if self.y == 0 {
                        self.p.set_zero(true);
                    }

                    if (self.y & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ASL_A => {
                    let old_a = self.a;
                    self.a <<= 1;
                    cycles -= 1;

                    if (old_a & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true);
                    } else {
                        self.p.set_negative(false);
                    }

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }

                    if (self.a & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ASL_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let new_byte = old_byte << 1;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true);
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ASL_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let new_byte = old_byte << 1;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true);
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ASL_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let new_byte = old_byte << 1;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true);
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ASL_ABSX => {
                    let address = self.fetch_word(&mut cycles, memory);

                    let effective_address = self.x as u16 + address;
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    // Discarded Data
                    cycles -= 1;

                    let new_byte = old_byte << 1;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true);
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                LSR_A => {
                    let old_a = self.a;
                    self.a >>= 1;
                    cycles -= 1;

                    if (old_a & 0b00000001) == 0b00000001 {
                        self.p.set_carry(true);
                    } else {
                        self.p.set_negative(false);
                    }

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }

                    if (self.a & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                LSR_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let new_byte = old_byte >> 1;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                LSR_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let new_byte = old_byte >> 1;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                LSR_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let new_byte = old_byte >> 1;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                LSR_ABSX => {
                    let address = self.fetch_word(&mut cycles, memory);

                    let effective_address = self.x as u16 + address;
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    // Discarded Data
                    cycles -= 1;

                    let new_byte = old_byte >> 1;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ROL_A => {
                    let old_a = self.a;
                    self.a <<= 1;
                    self.a |= self.p.bits() & 0b00000001;
                    cycles -= 1;

                    if (old_a & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }

                    if (self.a & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ROL_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let mut new_byte = old_byte << 1;
                    new_byte |= self.p.bits() & 0b00000001;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ROL_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let mut new_byte = old_byte << 1;
                    new_byte |= self.p.bits() & 0b00000001;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ROL_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let mut new_byte = old_byte << 1;
                    new_byte |= self.p.bits() & 0b00000001;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ROL_ABSX => {
                    let address = self.fetch_word(&mut cycles, memory);

                    let effective_address = self.x as u16 + address;
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    // Discarded Data
                    cycles -= 1;

                    let mut new_byte = old_byte << 1;
                    new_byte |= self.p.bits() & 0b00000001;
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ROR_A => {
                    let old_a = self.a;
                    self.a >>= 1;
                    if self.p.carry_flag() {
                        self.a |= 0b10000000;
                    } else {
                        self.a &= 0b01111111;
                    }
                    cycles -= 1;

                    if (old_a & 0b00000001) == 0b00000001 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if self.a == 0 {
                        self.p.set_zero(true);
                    }

                    if (self.a & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ROR_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let mut new_byte = old_byte >> 1;
                    if self.p.carry_flag() {
                        new_byte |= 0b10000000;
                    } else {
                        new_byte &= 0b01111111;
                    }
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ROR_ZPX => {
                    let effective_address = self.zero_page_x_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let mut new_byte = old_byte >> 1;
                    if self.p.carry_flag() {
                        new_byte |= 0b10000000;
                    } else {
                        new_byte &= 0b01111111;
                    }
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ROR_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    let mut new_byte = old_byte >> 1;
                    if self.p.carry_flag() {
                        new_byte |= 0b10000000;
                    } else {
                        new_byte &= 0b01111111;
                    }
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                ROR_ABSX => {
                    let address = self.fetch_word(&mut cycles, memory);

                    let effective_address = self.x as u16 + address;
                    let old_byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    // Discarded Data
                    cycles -= 1;

                    let mut new_byte = old_byte >> 1;
                    if self.p.carry_flag() {
                        new_byte |= 0b10000000;
                    } else {
                        new_byte &= 0b01111111;
                    }
                    cycles -= 1;

                    memory[effective_address as usize] = new_byte;
                    cycles -= 1;

                    if (old_byte & 0b10000000) == 0b10000000 {
                        self.p.set_carry(true)
                    } else {
                        self.p.set_negative(false);
                    }

                    if new_byte == 0 {
                        self.p.set_zero(true);
                    }

                    if (new_byte & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                JMP_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    self.pc = effective_address;
                }
                JMP_IND => {
                    let effective_address = self.indirect_addressing(&mut cycles, memory);
                    self.pc = effective_address;
                }
                JSR => {
                    let low_byte = self.fetch_byte(&mut cycles, memory);

                    // Discarded data
                    cycles -= 1;

                    memory[self.sp as usize] = (self.pc >> 8) as u8;
                    self.sp -= 1;
                    cycles -= 1;

                    memory[self.sp as usize] = self.pc as u8;
                    self.sp -= 1;
                    cycles -= 1;

                    let high_byte = self.fetch_byte(&mut cycles, memory);

                    self.pc = ((high_byte as u16) << 8) | low_byte as u16;
                }
                RTS => {
                    // Discarded data
                    cycles -= 1;

                    // Discarded data
                    cycles -= 1;

                    self.sp += 1;
                    let low_byte = memory[self.sp as usize];
                    cycles -= 1;

                    self.sp += 1;
                    let high_byte = memory[self.sp as usize];
                    cycles -= 1;

                    // Discarded data
                    cycles -= 1;

                    self.pc = ((high_byte as u16) << 8) | low_byte as u16;
                    self.pc += 1;
                }
                _ => panic!("Tried to execute unknown instruction"),
            }
        }
    }

    fn set_lda_flags(&mut self) {
        if self.a == 0 {
            self.p.set_zero(true);
        }

        if self.a & 0b10000000 == 0b10000000 {
            self.p.set_negative(true);
        }  
    }

    fn set_ldx_flags(&mut self) {
        if self.x == 0 {
            self.p.set_zero(true);
        }

        if self.x & 0b10000000 == 0b10000000 {
            self.p.set_negative(true);
        } 
    }

    fn set_ldy_flags(&mut self) {
        if self.y == 0 {
            self.p.set_zero(true);
        }

        if self.y & 0b10000000 == 0b10000000 {
            self.p.set_negative(true);
        } 
    }

    fn set_adc_sbc_flags(&mut self, overflow: bool, initial_value: u8) {
        if overflow {
            println!("found carry");
            // carry flag set
            self.p.set_carry(true)
        }

        // on incorrect sign
        if (initial_value & 0b10000000) != (self.a & 0b10000000) {
            println!("found overflow");
            // overflow flag set
            self.p.set_overflow(true);
        }

        if self.a == 0 {
            // zero flag set
            self.p.set_zero(true);
        }

        // if A has negative bit on
        if (self.a & 0b10000000) == 0b1000000 {
            // negative flag set
            self.p.set_negative(true);

        }
    }
}
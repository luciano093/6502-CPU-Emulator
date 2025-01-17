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
        const I = 0b00000100; // Interrupt Disable Flag
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

    pub fn set_interrupt(&mut self, state: bool) {
        self.set(0b00000100.into(), state);
    }

    pub const fn interrupt_flag(&self) -> bool {
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
            println!("A: {:04X}", self.a);
            println!("X: {:04X}", self.x);
            println!("Y: {:04X}", self.y);
            println!("flags: {:08b}", self.p.bits());

            match instruction {
                LDA_IM => {
                    self.a = self.fetch_byte(&mut cycles, memory);
                    self.set_lda_flags();
                }
                LDA_ZP | LDA_ZPX | LDA_ABS | LDA_ABSX | LDA_ABSY | LDA_INDX | LDA_INDY => {
                    let effective_address = match instruction {
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
                STA_ZP | STA_ZPX | STA_ABS | STA_ABSX | STA_ABSY | STA_INDX | STA_INDY => {
                    let effective_address= match instruction {
                        STA_ZP => self.zero_page_addressing(&mut cycles, memory) as usize,
                        STA_ZPX => self.zero_page_x_addressing(&mut cycles, memory) as usize,
                        STA_ABS => self.absolute_addressing(&mut cycles, memory) as usize,
                        STA_ABSX => self.absolute_x_addressing(&mut cycles, memory) as usize,
                        STA_ABSY => self.absolute_y_addressing(&mut cycles, memory) as usize,
                        STA_INDX => self.indirect_x_addressing(&mut cycles, memory) as usize,
                        STA_INDY => self.indirect_y_addressing(&mut cycles, memory) as usize,
                        _ => panic!("Unexpected STA instruction"),
                    };

                    memory[effective_address as usize] = self.a;
                }
                STX_ZP | STX_ZPY | STX_ABS => {
                    let effective_address= match instruction {
                        STX_ZP => self.zero_page_addressing(&mut cycles, memory) as usize,
                        STX_ZPY => self.zero_page_y_addressing(&mut cycles, memory) as usize,
                        STX_ABS => self.absolute_addressing(&mut cycles, memory) as usize,
                        _ => panic!("Unexpected STX instruction"),
                    };

                    memory[effective_address] = self.x;
                }
                STY_ZP | STY_ZPX | STY_ABS => {
                    let effective_address= match instruction {
                        STY_ZP => self.zero_page_addressing(&mut cycles, memory) as usize,
                        STY_ZPX => self.zero_page_x_addressing(&mut cycles, memory) as usize,
                        STY_ABS => self.absolute_addressing(&mut cycles, memory) as usize,
                        _ => panic!("Unexpected STY instruction"),
                    };

                    memory[effective_address] = self.x;
                }
                TAX => {
                    self.x = self.a;
                    cycles -= 1;

                    self.p.set_zero(self.x == 0);
            
                    self.p.set_negative(self.x & 0b10000000 == 0b10000000);
                }
                TAY => {
                    self.y = self.a;
                    cycles -= 1;

                    self.p.set_zero(self.y == 0);
            
                    self.p.set_negative(self.y & 0b10000000 == 0b10000000);
                }
                TXA => {
                    self.a = self.x;
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);  
                }
                TYA => {
                    self.a = self.y;
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                TSX => {
                    self.x = self.sp;
                    cycles -= 1;

                    self.p.set_zero(self.x == 0);
            
                    self.p.set_negative(self.x & 0b10000000 == 0b10000000);
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

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);  
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
                
                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);  
                }
                AND_ZP => {
                    let effectve_address = self.zero_page_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                AND_ZPX => {
                    let effectve_address = self.zero_page_x_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                AND_ABS => {
                    let effectve_address = self.absolute_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                AND_ABSX => {
                    let effectve_address = self.absolute_x_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                AND_ABSY => {
                    let effectve_address = self.absolute_y_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                AND_INDX => {
                    let effectve_address = self.indirect_x_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                AND_INDY => {
                    let effectve_address = self.indirect_y_addressing(&mut cycles, memory); 
                    self.a &= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                EOR_IM => {
                    self.a ^= self.fetch_byte(&mut cycles, memory);
                
                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);  
                }
                EOR_ZP => {
                    let effectve_address = self.zero_page_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                EOR_ZPX => {
                    let effectve_address = self.zero_page_x_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                EOR_ABS => {
                    let effectve_address = self.absolute_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                EOR_ABSX => {
                    let effectve_address = self.absolute_x_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                EOR_ABSY => {
                    let effectve_address = self.absolute_y_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                EOR_INDX => {
                    let effectve_address = self.indirect_x_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                EOR_INDY => {
                    let effectve_address = self.indirect_y_addressing(&mut cycles, memory); 
                    self.a ^= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                ORA_IM => {
                    self.a |= self.fetch_byte(&mut cycles, memory);
                
                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);  
                }
                ORA_ZP => {
                    let effectve_address = self.zero_page_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                ORA_ZPX => {
                    let effectve_address = self.zero_page_x_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                ORA_ABS => {
                    let effectve_address = self.absolute_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                ORA_ABSX => {
                    let effectve_address = self.absolute_x_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                ORA_ABSY => {
                    let effectve_address = self.absolute_y_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                ORA_INDX => {
                    let effectve_address = self.indirect_x_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                ORA_INDY => {
                    let effectve_address = self.indirect_y_addressing(&mut cycles, memory); 
                    self.a |= memory[effectve_address as usize];
                    cycles -= 1;

                    self.p.set_zero(self.a == 0);
            
                    self.p.set_negative(self.a & 0b10000000 == 0b10000000);
                }
                BIT_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let bit_test = self.a & memory[effective_address as usize];
                    cycles -= 1;
                    
                    self.p.set_zero(bit_test == 0);

                    self.p &= Status::from_bits(bit_test & 0b11000000).unwrap();
                }
                BIT_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let bit_test = self.a & memory[effective_address as usize];
                    
                    self.p.set_zero(bit_test == 0);

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

                    self.p.set_carry(self.a >= byte);
                    
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

                    self.p.set_carry(self.a >= byte);

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

                    self.p.set_carry(self.a >= byte);

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

                    self.p.set_carry(self.a >= byte);

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

                    self.p.set_carry(self.a >= byte);

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

                    self.p.set_carry(self.a >= byte);

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

                    self.p.set_carry(self.a >= byte);

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

                    self.p.set_carry(self.a >= byte);

                    if self.a == byte {
                        self.p.set_zero(true);
                    }

                    if self.a >= byte && ((self.a - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CPX_IM => {
                    let byte = self.fetch_byte(&mut cycles, memory);

                    println!("byte: {:04X}", byte);

                    self.p.set_carry(self.x >= byte);

                    if self.x == byte {
                        println!("setting zero to true");
                        self.p.set_zero(true);
                    }

                    if self.x >= byte && ((self.x - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CPX_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    self.p.set_carry(self.x >= byte);

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

                    self.p.set_carry(self.x >= byte);

                    if self.x == byte {
                        self.p.set_zero(true);
                    }

                    if self.x >= byte && ((self.x - byte) & 0b10000000) == 0b10000000 {
                        self.p.set_negative(true);
                    }
                }
                CPY_IM => {
                    let byte = self.fetch_byte(&mut cycles, memory);

                    self.p.set_carry(self.y >= byte);

                    self.p.set_zero(self.y == byte);

                    self.p.set_negative(self.y >= byte && ((self.y - byte) & 0b10000000) == 0b10000000);
                }
                CPY_ZP => {
                    let effective_address = self.zero_page_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    self.p.set_carry(self.y >= byte);

                    self.p.set_zero(self.y == byte);

                    self.p.set_negative(self.y >= byte && ((self.y - byte) & 0b10000000) == 0b10000000);
                }
                CPY_ABS => {
                    let effective_address = self.absolute_addressing(&mut cycles, memory);
                    let byte = self.read_memory(&mut cycles, memory, effective_address as usize);

                    self.p.set_carry(self.y >= byte);

                    self.p.set_zero(self.y == byte);

                    self.p.set_negative(self.y >= byte && ((self.y - byte) & 0b10000000) == 0b10000000);
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

                    self.p.set_zero(data == 0);

                    self.p.set_negative((data & 0b10000000) == 0b10000000);
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

                    self.p.set_zero(data == 0);

                    self.p.set_negative((data & 0b10000000) == 0b10000000);
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

                    self.p.set_zero(data == 0);

                    self.p.set_negative((data & 0b10000000) == 0b10000000);
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

                    self.p.set_zero(data == 0);

                    self.p.set_negative((data & 0b10000000) == 0b10000000);
                }
                INX => {
                    self.x += 1;
                    cycles -= 1;

                    self.p.set_zero(self.x == 0);
                    self.p.set_negative((self.x & 0b10000000) == 0b10000000);
                }
                INY => {
                    self.y += 1;
                    cycles -= 1;

                    self.p.set_zero(self.y == 0);

                    self.p.set_negative((self.y & 0b10000000) == 0b10000000);
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

                    self.p.set_zero(data == 0);

                    self.p.set_negative((data & 0b10000000) == 0b10000000);
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

                    self.p.set_zero(data == 0);

                    self.p.set_negative((data & 0b10000000) == 0b10000000);
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

                    self.p.set_zero(data == 0);

                    self.p.set_negative((data & 0b10000000) == 0b10000000);
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

                    self.p.set_zero(data == 0);

                    self.p.set_negative((data & 0b10000000) == 0b10000000);
                }
                DEX => {
                    self.x -= 1;

                    self.p.set_zero(self.x == 0);

                    self.p.set_negative((self.x & 0b10000000) == 0b10000000);
                }
                DEY => {
                    self.y -= 1;

                    self.p.set_zero(self.y == 0);

                    self.p.set_negative((self.y & 0b10000000) == 0b10000000);
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

                    self.p.set_zero(self.a == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(self.a == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(self.a == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(self.a == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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

                    self.p.set_zero(new_byte == 0);

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
                BCC => {
                    let offset = self.fetch_byte(&mut cycles, memory);
                    
                    if !self.p.carry_flag() {
                        cycles -= 1;

                        let new_location;

                        if offset >= 128 {
                            new_location = self.pc - (256u16.wrapping_sub(offset as u16)) as u16;
                        } else {
                            new_location = self.pc + offset as u16;
                        }

                        if self.pc & 0xFF00 != new_location & 0xFF00 {
                            cycles -= 1;
                            cycles -= 1;
                        }

                        self.pc = new_location;
                    }
                }
                BCS => {
                    let offset = self.fetch_byte(&mut cycles, memory);

                    if self.p.carry_flag() {
                        cycles -= 1;

                        let new_location;

                        if offset >= 128 {
                            new_location = self.pc - (256u16.wrapping_sub(offset as u16)) as u16;
                        } else {
                            new_location = self.pc + offset as u16;
                        }

                        if self.pc & 0xFF00 != new_location & 0xFF00 {
                            cycles -= 1;
                            cycles -= 1;
                        }

                        self.pc = new_location;
                    }
                }
                BEQ => {
                    let offset = self.fetch_byte(&mut cycles, memory);

                    if self.p.zero_flag() {
                        cycles -= 1;

                        let new_location;

                        if offset >= 128 {
                            new_location = self.pc - (256u16.wrapping_sub(offset as u16)) as u16;
                        } else {
                            new_location = self.pc + offset as u16;
                        }

                        if self.pc & 0xFF00 != new_location & 0xFF00 {
                            cycles -= 1;
                            cycles -= 1;
                        }

                        self.pc = new_location;
                    }
                }
                BMI => {
                    let offset = self.fetch_byte(&mut cycles, memory);

                    if self.p.negative_flag() {
                        cycles -= 1;

                        let new_location;

                        if offset >= 128 {
                            new_location = self.pc - (256u16.wrapping_sub(offset as u16)) as u16;
                        } else {
                            new_location = self.pc + offset as u16;
                        }

                        if self.pc & 0xFF00 != new_location & 0xFF00 {
                            cycles -= 1;
                            cycles -= 1;
                        }

                        self.pc = new_location;
                    }
                }
                BNE => {
                    let offset = self.fetch_byte(&mut cycles, memory);

                    if !self.p.zero_flag() {
                        cycles -= 1;

                        let new_location;

                        if offset >= 128 {
                            new_location = self.pc - (256u16.wrapping_sub(offset as u16)) as u16;
                        } else {
                            new_location = self.pc + offset as u16;
                        }

                        if self.pc & 0xFF00 != new_location & 0xFF00 {
                            cycles -= 1;
                            cycles -= 1;
                        }

                        self.pc = new_location;
                    }
                }
                BPL => {
                    let offset = self.fetch_byte(&mut cycles, memory);
                    
                    if !self.p.negative_flag() {
                        cycles -= 1;

                        let new_location;

                        if offset >= 128 {
                            new_location = self.pc - (256u16.wrapping_sub(offset as u16)) as u16;
                        } else {
                            new_location = self.pc + offset as u16;
                        }

                        if self.pc & 0xFF00 != new_location & 0xFF00 {
                            cycles -= 1;
                            cycles -= 1;
                        }

                        self.pc = new_location;
                    }
                }
                BVC => {
                    let offset = self.fetch_byte(&mut cycles, memory);

                    if !self.p.overflow_flag() {
                        cycles -= 1;

                        let new_location;

                        if offset >= 128 {
                            new_location = self.pc - (256u16.wrapping_sub(offset as u16)) as u16;
                        } else {
                            new_location = self.pc + offset as u16;
                        }

                        if self.pc & 0xFF00 != new_location & 0xFF00 {
                            cycles -= 1;
                            cycles -= 1;
                        }

                        self.pc = new_location;
                    }
                }
                BVS => {
                    let offset = self.fetch_byte(&mut cycles, memory);
                    cycles -= 1;

                    if self.p.overflow_flag() {
                        let new_location;

                        if offset >= 128 {
                            new_location = self.pc - (256u16.wrapping_sub(offset as u16)) as u16;
                        } else {
                            new_location = self.pc + offset as u16;
                        }

                        if self.pc & 0xFF00 != new_location & 0xFF00 {
                            cycles -= 1;
                            cycles -= 1;
                        }

                        self.pc = new_location;
                    }
                }
                CLC => {
                    self.p.set_carry(false);
                    cycles -= 1;
                }
                CLD => {
                    self.p.set_decimal(false);
                    cycles -= 1;
                }
                CLI => {
                    self.p.set_interrupt(false);
                    cycles -= 1;
                }
                CLV => {
                    self.p.set_overflow(false);
                    cycles -= 1;
                }
                SEC => {
                    self.p.set_carry(true);
                    cycles -= 1;
                }
                SED => {
                    self.p.set_decimal(true);
                    cycles -= 1;
                }
                SEI => {
                    self.p.set_interrupt(true);
                    cycles -= 1;
                }
                BRK => {
                    // Discarded data
                    cycles -= 1;

                    memory[self.sp as usize] = (self.pc >> 8) as u8;
                    self.sp -= 1;
                    cycles -= 1;

                    memory[self.sp as usize] = self.pc as u8;
                    self.sp -= 1;
                    cycles -= 1;

                    memory[self.sp as usize] = self.p.bits();
                    self.sp -= 1;
                    cycles -= 1;

                    let low_byte = memory[0xFFFE];
                    cycles -= 1;

                    let high_byte = memory[0xFFFF];
                    cycles -= 1;

                    self.pc = ((high_byte as u16) << 8) | low_byte as u16;

                    self.p.set_break(true);
                }
                // todo: research nop behavior
                NOP => (),
                RTI => {
                    // Discarded data
                    cycles -= 1;

                    // Discarded data
                    cycles -= 1;

                    self.sp += 1;
                    self.p = memory[self.sp as usize].into();
                    cycles -= 1;

                    self.sp += 1;
                    let low_byte = memory[self.sp as usize];
                    cycles -= 1;

                    self.sp += 1;
                    let high_byte = memory[self.sp as usize];
                    cycles -= 1;

                    self.pc = ((high_byte as u16) << 8) | low_byte as u16;
                }
                _ => panic!("Tried to execute unknown instruction"),
            }
        }
    }

    fn set_lda_flags(&mut self) {
        self.p.set_zero(self.a == 0);

        self.p.set_negative(self.a & 0b10000000 == 0b10000000);
    }

    fn set_ldx_flags(&mut self) {
        self.p.set_zero(self.x == 0);

        self.p.set_negative(self.x & 0b10000000 == 0b10000000);
    }

    fn set_ldy_flags(&mut self) {
        self.p.set_zero(self.y == 0);

        self.p.set_negative(self.y & 0b10000000 == 0b10000000);
    }

    fn set_adc_sbc_flags(&mut self, overflow: bool, initial_value: u8) {
        self.p.set_carry(overflow);

        // incorrect sign means there was an overflow
        self.p.set_overflow((initial_value & 0b10000000) != (self.a & 0b10000000));

        self.p.set_zero(self.a == 0);

        // if A has negative bit on
        self.p.set_negative((self.a & 0b10000000) == 0b1000000);
    }
}
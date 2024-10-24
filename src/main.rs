use std::ops;

use bitflags::bitflags;

type Byte = u8;
type Word = u16;

bitflags! {
    #[derive(Default, Debug)]
    struct Status: u8 {
        const C = 0b00000001; // Carry Flag
        const Z = 0b00000010; // Zero Flag
        const I = 0b00000100; // Interruptor Disable Flag
        const D = 0b00001000; // Decimal Mode Flag
        const B = 0b00010000; // Break Command Flag
        const V = 0b01000000; // Overflow Flag
        const N = 0b10000000; // Negative Flag
    }
}

struct Memory {
    bytes: [Byte; 65536], // 64kb
}

impl Memory {
    fn new() -> Self {
        Memory { bytes: [0; 65536] }
    }
}

impl ops::Index<Word> for Memory {
    type Output = Byte;

    fn index(&self, index: Word) -> &Self::Output {
        &self.bytes[index as usize]
    }
}

impl ops::IndexMut<Word> for Memory {
    fn index_mut(&mut self, index: Word) -> &mut Self::Output {
        &mut self.bytes[index as usize]
    }
}

const LDA_IM: Byte = 0xA9;
const LDA_ZP: Byte = 0xA5;
const LDA_ZPX: Byte = 0xB5;
const LDA_ABS: Byte = 0xAD;
const LDA_ABSX: Byte = 0xBD;
const LDA_ABSY: Byte = 0xB9;
const LDA_INDX: Byte = 0xA1;
const LDA_INDY: Byte = 0xB1;

#[derive(Default)]
struct CPU {
    pc: Word,    // Program Counter
    sp: Byte,   // Stack Pointer
    a: Byte,    // Accumulator
    x: Byte,    // Index Register X
    y: Byte,    // Index Register Y
    p: Status,  // Processor Status
}

impl CPU {
    fn reset(&mut self) {
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

    fn execute(&mut self, mut cycles: u32, memory: &mut Memory) {
        while cycles > 0 {
            let instruction = self.fetch_byte(&mut cycles, memory);

            match instruction {
                LDA_IM => {
                    self.a = self.fetch_byte(&mut cycles, memory);

                    self.set_lda_flags();
                }
                LDA_ZP => {
                    let address= self.fetch_byte(&mut cycles, memory);
                    self.a = self.read_memory(&mut cycles, memory, address as u16);

                    self.set_lda_flags();
                }
                LDA_ZPX => {
                    let address= self.fetch_byte(&mut cycles, memory);
                    let effective_address = (self.x as u16 + address as u16) % 256; // % 256 wraps around so that the max is a byte
                    cycles -= 1;

                    self.a = self.read_memory(&mut cycles, memory, effective_address);

                    self.set_lda_flags();
                }
                LDA_ABS => {
                    let address = self.fetch_word(&mut cycles, memory);
                    self.a = self.read_memory(&mut cycles, memory, address);

                    self.set_lda_flags();
                }
                LDA_ABSX => {
                    let address = self.fetch_word(&mut cycles, memory);

                    let effective_address = self.x as u16 + address;
                    cycles -= 1;

                    // checks if page was crossed (high byte of word are the same)
                    if (effective_address & 0xFF00) != (effective_address & 0xFF00) {
                        cycles -= 1;
                    }

                    self.a = self.read_memory(&mut cycles, memory, effective_address);

                    self.set_lda_flags();
                }
                LDA_ABSY => {
                    let address = self.fetch_word(&mut cycles, memory);

                    let effective_address = self.y as u16 + address;
                    cycles -= 1;

                    // checks if page was crossed (high byte of word are the same)
                    if (effective_address & 0xFF00) != (effective_address & 0xFF00) {
                        cycles -= 1;
                    }

                    self.a = self.read_memory(&mut cycles, memory, effective_address);

                    self.set_lda_flags();
                }
                LDA_INDX => {
                    let address = self.fetch_byte(&mut cycles, memory);

                    let effective_address = address.wrapping_add(self.x);
                    cycles -= 1;

                    let effective_address = self.read_word_memory(&mut cycles, memory, effective_address as u16);

                    self.a = self.read_memory(&mut cycles, memory, effective_address);

                    self.set_lda_flags();
                }
                LDA_INDY => {
                    let effective_address = self.fetch_byte(&mut cycles, memory);

                    let address = self.read_word_memory(&mut cycles, memory, effective_address as u16);
                    let effective_address = address + self.y as u16;
                    
                    // crosses a page
                    if (address & 0xFF00) != (effective_address & 0xFF00) {
                        cycles -= 1;
                    }

                    self.a = self.read_memory(&mut cycles, memory, effective_address);

                    self.set_lda_flags();
                }
                _ => (),
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
}
fn main() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.y = 0x04;
    mem[0xFFFC] = LDA_INDY;
    mem[0xFFFD] = 0x02;
    mem[0x02] = 0x00;
    mem[0x03] = 0x80;
    mem[0x8004] = 0x01;

    cpu.execute(6, &mut mem);

    println!("Accumulator: {:04x}", cpu.a);
}

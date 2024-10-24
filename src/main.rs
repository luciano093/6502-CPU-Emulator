use emulator_6502::consts::*;
use emulator_6502::memory::Memory;
use emulator_6502::cpu::CPU;

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

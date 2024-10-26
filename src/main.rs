use emulator_6502::consts::*;
use emulator_6502::memory::Memory;
use emulator_6502::cpu::CPU;

fn main() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset(&mem);

    // cpu.p |= Status::from_bits(0b00000001).unwrap();
    cpu.a = 0xFF;
    mem[0xFFFC] = CMP_IM;
    mem[0xFFFD] = 0xFF;

    cpu.execute(2, &mut mem);

    println!("final A: {}", cpu.a);
    println!("flags: {:08b}", cpu.p.bits());
}

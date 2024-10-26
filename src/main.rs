use emulator_6502::consts::*;
use emulator_6502::memory::Memory;
use emulator_6502::cpu::CPU;

fn main() {
    let mut mem = Memory::new();
    mem[0xFFFC] = 0x00;
    mem[0xFFFD] = 0xE0;

    let mut cpu = CPU::default();
    cpu.reset(&mem);

    // cpu.p |= Status::from_bits(0b00000001).unwrap();
    cpu.a = 0xFF;
    mem[0xE000] = ASL_A;

    cpu.execute(2, &mut mem);

    println!("final A: {}/{:02x}/{:08b}", cpu.a, cpu.a, cpu.a);
    println!("flags: {:08b}", cpu.p.bits());
}

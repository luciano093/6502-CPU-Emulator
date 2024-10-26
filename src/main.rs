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
    mem[0xE000] = JSR; // 6
    mem[0xE001] = 0x08;
    mem[0xE002] = 0xE0;
    mem[0xE003] = JSR; // 6
    mem[0xE004] = 0x0B;
    mem[0xE005] = 0xE0;
    mem[0xE006] = LDY_IM; // 2
    mem[0xE007] = 0x01;
    mem[0xE008] = LDX_IM; // 2
    mem[0xE009] = 0x00;
    mem[0xE00A] = RTS; // 6
    mem[0xE00B] = INX; // 2
    mem[0xE00C] = INX; // 2
    mem[0xE00D] = RTS; // 6

    cpu.execute(32, &mut mem);

    println!("final A: {}/{:02x}/{:08b}", cpu.a, cpu.a, cpu.a);
    println!("final X: {}/{:02x}/{:08b}", cpu.x, cpu.x, cpu.x);
    println!("final Y: {}/{:02x}/{:08b}", cpu.y, cpu.y, cpu.y);
    println!("pc: {}/{:02x}/{:08b}", cpu.pc, cpu.pc, cpu.pc);
    println!("sp: {}/{:02x}/{:08b}", cpu.sp, cpu.sp, cpu.sp);
    println!("flags: {:08b}", cpu.p.bits());
    println!("first 2 bytes of stack: {:2x} {:2x}", mem[0xff], mem[0xff - 1]);
}

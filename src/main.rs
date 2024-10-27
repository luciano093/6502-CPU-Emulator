use emulator_6502::consts::*;
use emulator_6502::memory::Memory;
use emulator_6502::cpu::CPU;

fn main() {
    let mut mem = Memory::new();
    mem[0xFFFC] = 0x00;
    mem[0xFFFD] = 0xE0;

    let mut cpu = CPU::default();
    cpu.reset(&mem);

    mem[0xE000] = JSR; // 6
    mem[0xE001] = 0x09;
    mem[0xE002] = 0xE0;
    mem[0xE003] = JSR; // 6
    mem[0xE004] = 0x0C;
    mem[0xE005] = 0xE0;
    mem[0xE006] = JSR; // 2
    mem[0xE007] = 0x12;
    mem[0xE008] = 0xE0; // 2
    mem[0xE009] = LDX_IM;
    mem[0xE00A] = 0x00; // 6
    mem[0xE00B] = RTS; // 2
    mem[0xE00C] = INX;
    mem[0xE00D] = CPX_IM;
    mem[0xE00E] = 0x05;
    mem[0xE00F] = BNE;
    mem[0xE010] = 0xFB; // 6
    mem[0xE011] = RTS; // 6
    mem[0xE012] = LDY_IM; // 6
    mem[0xE013] = 0x01; // 6

    cpu.execute(68, &mut mem);

    println!("final A: {}/{:02x}/{:08b}", cpu.a, cpu.a, cpu.a);
    println!("final X: {}/{:02x}/{:08b}", cpu.x, cpu.x, cpu.x);
    println!("final Y: {}/{:02x}/{:08b}", cpu.y, cpu.y, cpu.y);
    println!("pc: {}/{:02x}/{:08b}", cpu.pc, cpu.pc, cpu.pc);
    println!("sp: {}/{:02x}/{:08b}", cpu.sp, cpu.sp, cpu.sp);
    println!("flags: {:08b}", cpu.p.bits());
    println!("first 2 bytes of stack: {:2x} {:2x}", mem[0xff], mem[0xff - 1]);
}

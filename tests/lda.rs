use emulator_6502::cpu::CPU;
use emulator_6502::memory::Memory;
use emulator_6502::consts::*;

#[test]
fn lda_immediate_accum() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = LDA_IM;
    mem[0xFFFD] = 0x99;

    cpu.execute(2, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
#[should_panic]
fn lda_immediate_cycle_panic() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = LDA_IM;
    mem[0xFFFD] = 0x99;

    cpu.execute(3, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
fn lda_zero_page_accum() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = LDA_ZP;
    mem[0xFFFD] = 0xFF;
    mem[0xFF] = 0x99;

    cpu.execute(3, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
#[should_panic]
fn lda_zero_page_cycle_panic() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = LDA_ZP;
    mem[0xFFFD] = 0xFF;
    mem[0xFF] = 0x99;

    cpu.execute(4, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
fn lda_zero_page_x_accum() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.x = 0x60;
    mem[0xFFFC] = LDA_ZPX;
    mem[0xFFFD] = 0xC0;
    mem[0x20] = 0x99;

    cpu.execute(4, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
#[should_panic]
fn lda_zero_page_x_cycle_panic() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.x = 0x60;
    mem[0xFFFC] = LDA_ZPX;
    mem[0xFFFD] = 0xC0;
    mem[0x20] = 0x99;

    cpu.execute(5, &mut mem);
}

#[test]
fn lda_absolute_accum() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = LDA_ABSX;
    mem[0xFFFD] = 0x00;
    mem[0xFFFE] = 0x20;
    mem[0x2000] = 0x99;

    cpu.execute(4, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
#[should_panic]
fn lda_absolute_cycle_panic() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = LDA_ABSX;
    mem[0xFFFD] = 0x00;
    mem[0xFFFE] = 0x20;
    mem[0x2000] = 0x99;

    cpu.execute(5, &mut mem);
}

#[test]
fn lda_absolute_x_accum() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.x = 0x92;
    mem[0xFFFC] = LDA_ABSX;
    mem[0xFFFD] = 0x00;
    mem[0xFFFE] = 0x20;
    mem[0x2092] = 0x99;

    cpu.execute(4, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
fn lda_absolute_x_page_cross() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.x = 0x01;
    mem[0xFFFC] = LDA_ABSX;
    mem[0xFFFD] = 0xFF;
    mem[0xFFFE] = 0x1F;
    mem[0x2000] = 0x99;

    cpu.execute(5, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
#[should_panic]
fn lda_absolute_x_cycle_panic() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.x = 0x92;
    mem[0xFFFC] = LDA_ABSX;
    mem[0xFFFD] = 0x00;
    mem[0xFFFE] = 0x20;
    mem[0x2092] = 0x99;

    cpu.execute(5, &mut mem);
}

#[test]
fn lda_absolute_y_accum() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.y = 0x92;
    mem[0xFFFC] = LDA_ABSY;
    mem[0xFFFD] = 0x00;
    mem[0xFFFE] = 0x20;
    mem[0x2092] = 0x99;

    cpu.execute(4, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
fn lda_absolute_y_page_cross() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.y = 0x01;
    mem[0xFFFC] = LDA_ABSY;
    mem[0xFFFD] = 0xFF;
    mem[0xFFFE] = 0x1F;
    mem[0x2000] = 0x99;

    cpu.execute(5, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
#[should_panic]
fn lda_absolute_y_cycle_panic() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.y = 0x92;
    mem[0xFFFC] = LDA_ABSY;
    mem[0xFFFD] = 0x00;
    mem[0xFFFE] = 0x20;
    mem[0x2092] = 0x99;

    cpu.execute(5, &mut mem);
}

#[test]
fn lda_indexed_indirect_accum() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.x = 0x04;
    mem[0xFFFC] = LDA_INDX;
    mem[0xFFFD] = 0x20;
    mem[0x24] = 0x74;
    mem[0x25] = 0x20;
    mem[0x2074] = 0x99;

    cpu.execute(6, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
#[should_panic]
fn lda_indexed_indirect_cycle_panic() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.y = 0x04;
    mem[0xFFFC] = LDA_INDX;
    mem[0xFFFD] = 0x20;
    mem[0x24] = 0x74;
    mem[0x25] = 0x20;
    mem[0x2074] = 0x99;

    cpu.execute(7, &mut mem);
}

#[test]
fn lda_indirect_indexed_accum() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.y = 0x10;
    mem[0xFFFC] = LDA_INDY;
    mem[0xFFFD] = 0x86;
    mem[0x86] = 0x28;
    mem[0x87] = 0x40;
    mem[0x4038] = 0x99;

    cpu.execute(5, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
fn lda_indirect_indexed_page_cross() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.y = 0x01;
    mem[0xFFFC] = LDA_INDY;
    mem[0xFFFD] = 0x86;
    mem[0x86] = 0xFF;
    mem[0x87] = 0x1F;
    mem[0x2000] = 0x99;

    cpu.execute(6, &mut mem);

    assert_eq!(cpu.a, 0x99);
}

#[test]
#[should_panic]
fn lda_indirect_indexed_cycle_panic() {
    let mut mem = Memory::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.y = 0x10;
    mem[0xFFFC] = LDA_INDY;
    mem[0xFFFD] = 0x86;
    mem[0x86] = 0x28;
    mem[0x87] = 0x40;
    mem[0x4038] = 0x99;

    cpu.execute(7, &mut mem);
}
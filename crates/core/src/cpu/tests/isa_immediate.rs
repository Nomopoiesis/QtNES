use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_IMMEDIATE: u8 = 0x09;
const AND_IMMEDIATE: u8 = 0x29;
const EOR_IMMEDIATE: u8 = 0x49;
const ADC_IMMEDIATE: u8 = 0x69;
const LDY_IMMEDIATE: u8 = 0xA0;
const LDX_IMMEDIATE: u8 = 0xA2;
const LDA_IMMEDIATE: u8 = 0xA9;
const CPY_IMMEDIATE: u8 = 0xC0;
const CMP_IMMEDIATE: u8 = 0xC9;
const CPX_IMMEDIATE: u8 = 0xE0;
const SBC_IMMEDIATE: u8 = 0xE9;

#[test]
fn ora_immediate() {
    let (mut cpu, mut bus) = setup(&[ORA_IMMEDIATE, 0xF0]);
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ORA_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn ora_immediate_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[ORA_IMMEDIATE, 0x00]);
    cpu.registers.a = 0x00;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn and_immediate() {
    let (mut cpu, mut bus) = setup(&[AND_IMMEDIATE, 0x9F]);
    cpu.registers.a = 0xF0;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x90);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(AND_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn and_immediate_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[AND_IMMEDIATE, 0x0F]);
    cpu.registers.a = 0xF0;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn eor_immediate() {
    let (mut cpu, mut bus) = setup(&[EOR_IMMEDIATE, 0xFF]);
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xF0);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(EOR_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn eor_immediate_sets_zero_flag() {
    // a value XOR itself is zero
    let (mut cpu, mut bus) = setup(&[EOR_IMMEDIATE, 0xA5]);
    cpu.registers.a = 0xA5;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn adc_immediate() {
    let (mut cpu, mut bus) = setup(&[ADC_IMMEDIATE, 0x20]);
    cpu.registers.a = 0x10;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    // $10 + #$20 + carry
    assert_eq!(cpu.registers.a, 0x31);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ADC_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn adc_immediate_sets_carry_and_overflow() {
    // -48 + -112 = -160: carry out and signed overflow
    let (mut cpu, mut bus) = setup(&[ADC_IMMEDIATE, 0x90]);
    cpu.registers.a = 0xD0;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x60);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ldy_immediate() {
    let (mut cpu, mut bus) = setup(&[LDY_IMMEDIATE, 0x80]);
    cpu.registers.y = 0x11;
    cpu.registers.a = 0x22;
    cpu.registers.x = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.y, 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(LDY_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x22);
    assert_eq!(cpu.registers.x, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ldy_immediate_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDY_IMMEDIATE, 0x00]);
    cpu.registers.y = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.y, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ldx_immediate() {
    let (mut cpu, mut bus) = setup(&[LDX_IMMEDIATE, 0x80]);
    cpu.registers.x = 0x11;
    cpu.registers.a = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.x, 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(LDX_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ldx_immediate_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDX_IMMEDIATE, 0x00]);
    cpu.registers.x = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.x, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lda_immediate() {
    let (mut cpu, mut bus) = setup(&[LDA_IMMEDIATE, 0x80]);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(LDA_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn lda_immediate_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDA_IMMEDIATE, 0x00]);
    cpu.registers.a = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cpy_immediate() {
    // Y greater than the operand: C set, Z clear, N from $50 - $30 = $20
    let (mut cpu, mut bus) = setup(&[CPY_IMMEDIATE, 0x30]);
    cpu.registers.y = 0x50;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(CPY_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed: the compare does not store its result
    assert_eq!(cpu.registers.y, 0x50);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn cpy_immediate_equal_sets_zero_and_carry() {
    let (mut cpu, mut bus) = setup(&[CPY_IMMEDIATE, 0x42]);
    cpu.registers.y = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cpy_immediate_less_than_clears_carry() {
    // $30 - $50 borrows; the wrapped difference $E0 has bit 7 set
    let (mut cpu, mut bus) = setup(&[CPY_IMMEDIATE, 0x50]);
    cpu.registers.y = 0x30;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::ZERO, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cpy_immediate_negative_comes_from_the_difference_not_the_sign() {
    // unsigned $80 >= $01, so C is set, and $80 - $01 = $7F has bit 7 clear:
    // N is clear even though Y is "negative" and the operand is positive
    let (mut cpu, mut bus) = setup(&[CPY_IMMEDIATE, 0x01]);
    cpu.registers.y = 0x80;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cpy_immediate_does_not_touch_overflow() {
    // $80 - $01 overflows as a signed subtraction, but compares never set V
    let (mut cpu, mut bus) = setup(&[CPY_IMMEDIATE, 0x01]);
    cpu.registers.y = 0x80;
    cpu.status.set(StatusFlags::OVERFLOW, false);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
}

#[test]
fn cmp_immediate() {
    // A greater than the operand: C set, Z clear, N from $50 - $30 = $20
    let (mut cpu, mut bus) = setup(&[CMP_IMMEDIATE, 0x30]);
    cpu.registers.a = 0x50;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(CMP_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed: the compare does not store its result
    assert_eq!(cpu.registers.a, 0x50);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn cmp_immediate_equal_sets_zero_and_carry() {
    let (mut cpu, mut bus) = setup(&[CMP_IMMEDIATE, 0x42]);
    cpu.registers.a = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cmp_immediate_less_than_clears_carry() {
    // $30 - $50 borrows; the wrapped difference $E0 has bit 7 set
    let (mut cpu, mut bus) = setup(&[CMP_IMMEDIATE, 0x50]);
    cpu.registers.a = 0x30;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cpx_immediate() {
    // X greater than the operand: C set, Z clear, N from $50 - $30 = $20
    let (mut cpu, mut bus) = setup(&[CPX_IMMEDIATE, 0x30]);
    cpu.registers.x = 0x50;
    cpu.registers.a = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(CPX_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed: the compare does not store its result
    assert_eq!(cpu.registers.x, 0x50);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn cpx_immediate_equal_sets_zero_and_carry() {
    let (mut cpu, mut bus) = setup(&[CPX_IMMEDIATE, 0x42]);
    cpu.registers.x = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cpx_immediate_less_than_clears_carry() {
    // $30 - $50 borrows; the wrapped difference $E0 has bit 7 set
    let (mut cpu, mut bus) = setup(&[CPX_IMMEDIATE, 0x50]);
    cpu.registers.x = 0x30;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cpx_immediate_compares_x_not_a() {
    // A equals the operand but X does not: Z must stay clear
    let (mut cpu, mut bus) = setup(&[CPX_IMMEDIATE, 0x42]);
    cpu.registers.x = 0x10;
    cpu.registers.a = 0x42;

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn sbc_immediate() {
    // carry set means no borrow: $50 - $20 = $30
    let (mut cpu, mut bus) = setup(&[SBC_IMMEDIATE, 0x20]);
    cpu.registers.a = 0x50;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x30);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(SBC_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sbc_immediate_sets_overflow_positive_minus_negative() {
    // 80 - (-48) = 128, which does not fit in a signed byte
    let (mut cpu, mut bus) = setup(&[SBC_IMMEDIATE, 0xD0]);
    cpu.registers.a = 0x50;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x80);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

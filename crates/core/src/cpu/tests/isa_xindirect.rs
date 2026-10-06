use super::{Access, PROGRAM_START, TestBus, run_instruction, setup, table_cycles};
use crate::cpu::{CPU, StatusFlags};

const ORA_XINDIRECT: u8 = 0x01;
const AND_XINDIRECT: u8 = 0x21;
const EOR_XINDIRECT: u8 = 0x41;
const ADC_XINDIRECT: u8 = 0x61;
const STA_XINDIRECT: u8 = 0x81;
const LDA_XINDIRECT: u8 = 0xA1;
const CMP_XINDIRECT: u8 = 0xC1;
const SBC_XINDIRECT: u8 = 0xE1;

#[test]
fn ora_xindirect() {
    let (mut cpu, mut bus) = setup(&[ORA_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x01;
    // pointer at $20 + X = $24 holds the effective address $1234
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = 0x80;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x81);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ORA_XINDIRECT));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed pointer
            Access::Read(0x0024),
            Access::Read(0x0025),
            Access::Read(0x1234),
        ]
    );
}

#[test]
fn ora_xindirect_index_wraps_within_zero_page() {
    let (mut cpu, mut bus) = setup(&[ORA_XINDIRECT, 0xFE]);
    cpu.registers.x = 0x03;
    // $FE + $03 wraps to $01, not $0101
    bus.memory[0x0001] = 0x34;
    bus.memory[0x0002] = 0x12;
    bus.memory[0x1234] = 0x42;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
}

#[test]
fn ora_xindirect_pointer_high_byte_wraps_within_zero_page() {
    let (mut cpu, mut bus) = setup(&[ORA_XINDIRECT, 0xFF]);
    cpu.registers.x = 0x00;
    // pointer low byte at $FF, high byte wraps to $00 instead of $0100
    bus.memory[0x00FF] = 0x34;
    bus.memory[0x0000] = 0x12;
    bus.memory[0x0100] = 0x77;
    bus.memory[0x1234] = 0x42;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
}

#[test]
fn and_xindirect() {
    let (mut cpu, mut bus) = setup(&[AND_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = 0xF0;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    // pointer at $20 + X = $24 holds the effective address $1234
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = 0x9F;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x90);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(AND_XINDIRECT));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed pointer
            Access::Read(0x0024),
            Access::Read(0x0025),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0x9F);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn and_xindirect_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[AND_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = 0xF0;
    cpu.status.set(StatusFlags::NEGATIVE, true);
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = 0x0F;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn eor_xindirect() {
    let (mut cpu, mut bus) = setup(&[EOR_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x0F;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    // pointer at $20 + X = $24 holds the effective address $1234
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = 0xFF;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xF0);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(EOR_XINDIRECT));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed pointer
            Access::Read(0x0024),
            Access::Read(0x0025),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0xFF);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn eor_xindirect_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[EOR_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    // a value XOR itself is zero
    cpu.registers.a = 0xA5;
    cpu.status.set(StatusFlags::NEGATIVE, true);
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = 0xA5;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

/// ADC (indirect,X) with A = `a`, operand `operand` at $1234 (pointer at $24,
/// X = $04) and the carry flag set to `carry`.
fn setup_adc_xindirect(a: u8, operand: u8, carry: bool) -> (CPU, TestBus) {
    let (mut cpu, mut bus) = setup(&[ADC_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = a;
    cpu.status.set(StatusFlags::CARRY, carry);
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = operand;
    (cpu, bus)
}

#[test]
fn adc_xindirect() {
    let (mut cpu, mut bus) = setup_adc_xindirect(0x10, 0x20, false);
    cpu.registers.y = 0x22;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x30);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ADC_XINDIRECT));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed pointer
            Access::Read(0x0024),
            Access::Read(0x0025),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0x20);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn adc_xindirect_adds_carry_in() {
    let (mut cpu, mut bus) = setup_adc_xindirect(0x10, 0x20, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x31);
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn adc_xindirect_sets_carry_and_zero_on_unsigned_overflow() {
    // $FF + $01 = $100: carry out, zero result, no signed overflow (-1 + 1 = 0)
    let (mut cpu, mut bus) = setup_adc_xindirect(0xFF, 0x01, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn adc_xindirect_sets_overflow_adding_two_positives() {
    // 80 + 80 = 160, which does not fit in a signed byte
    let (mut cpu, mut bus) = setup_adc_xindirect(0x50, 0x50, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xA0);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn adc_xindirect_sets_overflow_adding_two_negatives() {
    // -48 + -112 = -160, which does not fit in a signed byte
    let (mut cpu, mut bus) = setup_adc_xindirect(0xD0, 0x90, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x60);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn adc_xindirect_clears_overflow_for_mixed_signs() {
    // 80 + -48 = 32: inputs of opposite sign can never overflow
    let (mut cpu, mut bus) = setup_adc_xindirect(0x50, 0xD0, false);
    cpu.status.set(StatusFlags::OVERFLOW, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x20);
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn adc_xindirect_ignores_decimal_flag() {
    // the 2A03 has no BCD mode: $09 + $01 is $0A, not $10
    let (mut cpu, mut bus) = setup_adc_xindirect(0x09, 0x01, false);
    cpu.status.set(StatusFlags::DECIMAL_MODE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x0A);
    assert!(cpu.status.get(StatusFlags::DECIMAL_MODE));
}

#[test]
fn sta_xindirect() {
    let (mut cpu, mut bus) = setup(&[STA_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x80;
    cpu.registers.y = 0x22;
    // every flag set except B: a store changes none of them
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;
    // pointer at $20 + X = $24 holds the effective address $1234
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = 0x55;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x80);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(STA_XINDIRECT));
    // the target is written without being read first
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed pointer
            Access::Read(0x0024),
            Access::Read(0x0025),
            Access::Write(0x1234, 0x80),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, 0xFF & !StatusFlags::BREAK_COMMAND);
    assert_eq!(cpu.registers.a, 0x80);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sta_xindirect_does_not_change_flags_for_zero() {
    let (mut cpu, mut bus) = setup(&[STA_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x00;
    cpu.status.0 = 0x00;
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = 0x55;

    run_instruction(&mut cpu, &mut bus);

    // storing zero does not set Z
    assert_eq!(bus.memory[0x1234], 0x00);
    assert_eq!(cpu.status.0, 0x00);
}

#[test]
fn sta_xindirect_index_wraps_within_zero_page() {
    let (mut cpu, mut bus) = setup(&[STA_XINDIRECT, 0xFE]);
    cpu.registers.x = 0x03;
    cpu.registers.a = 0x42;
    // $FE + $03 wraps to $01, not $0101
    bus.memory[0x0001] = 0x34;
    bus.memory[0x0002] = 0x12;
    bus.memory[0x0101] = 0x78;
    bus.memory[0x0102] = 0x56;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x42);
    assert_eq!(bus.memory[0x5678], 0x00);
}

#[test]
fn sta_xindirect_pointer_high_byte_wraps_within_zero_page() {
    let (mut cpu, mut bus) = setup(&[STA_XINDIRECT, 0xFF]);
    cpu.registers.x = 0x00;
    cpu.registers.a = 0x42;
    // pointer low byte at $FF, high byte wraps to $00 instead of $0100
    bus.memory[0x00FF] = 0x34;
    bus.memory[0x0000] = 0x12;
    bus.memory[0x0100] = 0x77;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x42);
    assert_eq!(bus.memory[0x7734], 0x00);
}

#[test]
fn lda_xindirect() {
    let (mut cpu, mut bus) = setup(&[LDA_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);
    // pointer at $20 + X = $24 holds the effective address $1234
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = 0x80;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(LDA_XINDIRECT));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed pointer
            Access::Read(0x0024),
            Access::Read(0x0025),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0x80);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn lda_xindirect_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDA_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = 0x00;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

/// CMP ($20,X) with X = $04: the pointer at $24 holds $1234, which holds `operand`.
fn setup_cmp_xindirect(a: u8, operand: u8) -> (CPU, TestBus) {
    let (mut cpu, mut bus) = setup(&[CMP_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = a;
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = operand;
    (cpu, bus)
}

#[test]
fn cmp_xindirect() {
    // A greater than the operand: C set, Z clear, N from $50 - $30 = $20
    let (mut cpu, mut bus) = setup_cmp_xindirect(0x50, 0x30);
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(CMP_XINDIRECT));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed pointer
            Access::Read(0x0024),
            Access::Read(0x0025),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed: the compare does not store its result
    assert_eq!(cpu.registers.a, 0x50);
    assert_eq!(bus.memory[0x1234], 0x30);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn cmp_xindirect_equal_sets_zero_and_carry() {
    let (mut cpu, mut bus) = setup_cmp_xindirect(0x42, 0x42);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cmp_xindirect_less_than_clears_carry() {
    // $30 - $50 borrows; the wrapped difference $E0 has bit 7 set
    let (mut cpu, mut bus) = setup_cmp_xindirect(0x30, 0x50);
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::ZERO, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cmp_xindirect_negative_comes_from_the_difference_not_the_sign() {
    // unsigned $80 >= $01, and $80 - $01 = $7F has bit 7 clear; V is never touched
    let (mut cpu, mut bus) = setup_cmp_xindirect(0x80, 0x01);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::OVERFLOW, false);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
}

/// SBC ($20,X) with X = $04: the pointer at $24 holds $1234, which holds `operand`.
fn setup_sbc_xindirect(a: u8, operand: u8, carry: bool) -> (CPU, TestBus) {
    let (mut cpu, mut bus) = setup(&[SBC_XINDIRECT, 0x20]);
    cpu.registers.x = 0x04;
    cpu.registers.a = a;
    cpu.status.set(StatusFlags::CARRY, carry);
    bus.memory[0x0024] = 0x34;
    bus.memory[0x0025] = 0x12;
    bus.memory[0x1234] = operand;
    (cpu, bus)
}

#[test]
fn sbc_xindirect() {
    // carry set means no borrow: $50 - $20 = $30
    let (mut cpu, mut bus) = setup_sbc_xindirect(0x50, 0x20, true);
    cpu.registers.y = 0x22;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x30);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(SBC_XINDIRECT));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed pointer
            Access::Read(0x0024),
            Access::Read(0x0025),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0x20);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sbc_xindirect_subtracts_borrow_when_carry_clear() {
    // carry clear means borrow: $50 - $20 - 1 = $2F
    let (mut cpu, mut bus) = setup_sbc_xindirect(0x50, 0x20, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x2F);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn sbc_xindirect_equal_operands_give_zero_and_keep_carry() {
    let (mut cpu, mut bus) = setup_sbc_xindirect(0x42, 0x42, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
}

#[test]
fn sbc_xindirect_clears_carry_on_borrow() {
    // $20 - $50 = -$30: the unsigned result wraps to $D0 and borrows
    let (mut cpu, mut bus) = setup_sbc_xindirect(0x20, 0x50, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xD0);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
}

#[test]
fn sbc_xindirect_sets_overflow_positive_minus_negative() {
    // 80 - (-48) = 128, which does not fit in a signed byte
    let (mut cpu, mut bus) = setup_sbc_xindirect(0x50, 0xD0, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x80);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn sbc_xindirect_sets_overflow_negative_minus_positive() {
    // -48 - 112 = -160, which does not fit in a signed byte
    let (mut cpu, mut bus) = setup_sbc_xindirect(0xD0, 0x70, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x60);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn sbc_xindirect_clears_overflow_for_same_signs() {
    // 80 - 48 = 32: subtracting a same-signed value can never overflow
    let (mut cpu, mut bus) = setup_sbc_xindirect(0x50, 0x30, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x20);
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
}

#[test]
fn sbc_xindirect_ignores_decimal_flag() {
    // the 2A03 has no BCD mode: $10 - $01 is $0F, not $09
    let (mut cpu, mut bus) = setup_sbc_xindirect(0x10, 0x01, true);
    cpu.status.set(StatusFlags::DECIMAL_MODE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x0F);
}

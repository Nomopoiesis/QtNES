use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_ZEROPAGE: u8 = 0x05;
const ASL_ZEROPAGE: u8 = 0x06;
const BIT_ZEROPAGE: u8 = 0x24;
const AND_ZEROPAGE: u8 = 0x25;
const ROL_ZEROPAGE: u8 = 0x26;
const EOR_ZEROPAGE: u8 = 0x45;
const LSR_ZEROPAGE: u8 = 0x46;
const ADC_ZEROPAGE: u8 = 0x65;
const ROR_ZEROPAGE: u8 = 0x66;
const STY_ZEROPAGE: u8 = 0x84;
const STA_ZEROPAGE: u8 = 0x85;
const STX_ZEROPAGE: u8 = 0x86;
const LDY_ZEROPAGE: u8 = 0xA4;
const LDA_ZEROPAGE: u8 = 0xA5;
const LDX_ZEROPAGE: u8 = 0xA6;
const CPY_ZEROPAGE: u8 = 0xC4;
const CMP_ZEROPAGE: u8 = 0xC5;
const DEC_ZEROPAGE: u8 = 0xC6;
const CPX_ZEROPAGE: u8 = 0xE4;
const SBC_ZEROPAGE: u8 = 0xE5;
const INC_ZEROPAGE: u8 = 0xE6;

#[test]
fn ora_zeropage() {
    let (mut cpu, mut bus) = setup(&[ORA_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0xF0;
    cpu.registers.a = 0x0F;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ORA_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0010),
        ]
    );
}

#[test]
fn ora_zeropage_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[ORA_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x00;
    cpu.registers.a = 0x00;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn asl_zeropage() {
    let (mut cpu, mut bus) = setup(&[ASL_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x41;
    cpu.registers.a = 0x55;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x82);
    assert_eq!(cpu.registers.a, 0x55, "accumulator must not be touched");
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ASL_ZEROPAGE));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0010),
            Access::Write(0x0010, 0x41),
            Access::Write(0x0010, 0x82),
        ]
    );
}

#[test]
fn asl_zeropage_shifts_bit_7_into_carry() {
    let (mut cpu, mut bus) = setup(&[ASL_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x80;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn bit_zeropage() {
    let (mut cpu, mut bus) = setup(&[BIT_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0xC0;
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    // $0F AND $C0 is zero; N and V are bits 7 and 6 of the operand
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(BIT_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0010),
        ]
    );

    // nothing else changed, the accumulator included
    assert_eq!(cpu.registers.a, 0x0F);
    assert_eq!(bus.memory[0x0010], 0xC0);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn bit_zeropage_clears_zero_negative_and_overflow() {
    let (mut cpu, mut bus) = setup(&[BIT_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x01;
    cpu.registers.a = 0xFF;
    cpu.status.set(StatusFlags::ZERO, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0xFF);
}

#[test]
fn bit_zeropage_negative_and_overflow_come_from_operand_not_result() {
    let (mut cpu, mut bus) = setup(&[BIT_ZEROPAGE, 0x10]);
    // operand has bit 6 set and bit 7 clear; the AND with A is zero
    bus.memory[0x0010] = 0x40;
    cpu.registers.a = 0x80;

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
}

#[test]
fn and_zeropage() {
    let (mut cpu, mut bus) = setup(&[AND_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x9F;
    cpu.registers.a = 0xF0;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x90);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(AND_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0010),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0010], 0x9F);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn and_zeropage_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[AND_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x0F;
    cpu.registers.a = 0xF0;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn rol_zeropage() {
    let (mut cpu, mut bus) = setup(&[ROL_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x41;
    cpu.registers.a = 0x55;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x82);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ROL_ZEROPAGE));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0010),
            Access::Write(0x0010, 0x41),
            Access::Write(0x0010, 0x82),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn rol_zeropage_rotates_carry_into_bit_0() {
    let (mut cpu, mut bus) = setup(&[ROL_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x01;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x03);
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn rol_zeropage_rotates_bit_7_into_carry() {
    let (mut cpu, mut bus) = setup(&[ROL_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x80;
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn rol_zeropage_carry_in_and_out_together() {
    let (mut cpu, mut bus) = setup(&[ROL_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x80;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    // bit 7 leaves into carry while the old carry enters bit 0
    assert_eq!(bus.memory[0x0010], 0x01);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
}

#[test]
fn eor_zeropage() {
    let (mut cpu, mut bus) = setup(&[EOR_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0xFF;
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xF0);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(EOR_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0010),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0010], 0xFF);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn eor_zeropage_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[EOR_ZEROPAGE, 0x10]);
    // a value XOR itself is zero
    bus.memory[0x0010] = 0xA5;
    cpu.registers.a = 0xA5;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lsr_zeropage() {
    let (mut cpu, mut bus) = setup(&[LSR_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x82;
    cpu.registers.a = 0x55;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x41);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(LSR_ZEROPAGE));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0010),
            Access::Write(0x0010, 0x82),
            Access::Write(0x0010, 0x41),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn lsr_zeropage_shifts_bit_0_into_carry() {
    let (mut cpu, mut bus) = setup(&[LSR_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x01;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lsr_zeropage_shifts_zero_into_bit_7() {
    let (mut cpu, mut bus) = setup(&[LSR_ZEROPAGE, 0x10]);
    // carry set beforehand must not be shifted in, unlike ROR
    bus.memory[0x0010] = 0xFF;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x7F);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn adc_zeropage() {
    let (mut cpu, mut bus) = setup(&[ADC_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x20;
    cpu.registers.a = 0x10;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    // $10 + $20 + carry
    assert_eq!(cpu.registers.a, 0x31);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ADC_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0010),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0010], 0x20);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn adc_zeropage_sets_carry_and_overflow() {
    // -48 + -112 = -160: carry out and signed overflow
    let (mut cpu, mut bus) = setup(&[ADC_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x90;
    cpu.registers.a = 0xD0;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x60);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ror_zeropage() {
    let (mut cpu, mut bus) = setup(&[ROR_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x82;
    cpu.registers.a = 0x55;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x41);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ROR_ZEROPAGE));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0010),
            Access::Write(0x0010, 0x82),
            Access::Write(0x0010, 0x41),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ror_zeropage_rotates_carry_into_bit_7() {
    let (mut cpu, mut bus) = setup(&[ROR_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x02;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x81);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ror_zeropage_rotates_bit_0_into_carry() {
    let (mut cpu, mut bus) = setup(&[ROR_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x01;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ror_zeropage_carry_in_and_out_together() {
    let (mut cpu, mut bus) = setup(&[ROR_ZEROPAGE, 0x10]);
    bus.memory[0x0010] = 0x01;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    // bit 0 leaves into carry while the old carry enters bit 7
    assert_eq!(bus.memory[0x0010], 0x80);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
}

#[test]
fn sty_zeropage() {
    let (mut cpu, mut bus) = setup(&[STY_ZEROPAGE, 0x42]);
    cpu.registers.y = 0x80;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    // every flag set except B: a store changes none of them
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;
    bus.memory[0x0042] = 0x55;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0042], 0x80);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(STY_ZEROPAGE));
    // the target is written without being read first
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Write(0x0042, 0x80),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, 0xFF & !StatusFlags::BREAK_COMMAND);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x80);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sty_zeropage_does_not_change_flags_for_zero() {
    let (mut cpu, mut bus) = setup(&[STY_ZEROPAGE, 0x42]);
    cpu.registers.y = 0x00;
    cpu.status.0 = 0x00;
    bus.memory[0x0042] = 0x55;

    run_instruction(&mut cpu, &mut bus);

    // storing zero does not set Z
    assert_eq!(bus.memory[0x0042], 0x00);
    assert_eq!(cpu.status.0, 0x00);
}

#[test]
fn sta_zeropage() {
    let (mut cpu, mut bus) = setup(&[STA_ZEROPAGE, 0x42]);
    cpu.registers.a = 0x80;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    // every flag set except B: a store changes none of them
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;
    bus.memory[0x0042] = 0x55;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0042], 0x80);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(STA_ZEROPAGE));
    // the target is written without being read first
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Write(0x0042, 0x80),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, 0xFF & !StatusFlags::BREAK_COMMAND);
    assert_eq!(cpu.registers.a, 0x80);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sta_zeropage_does_not_change_flags_for_zero() {
    let (mut cpu, mut bus) = setup(&[STA_ZEROPAGE, 0x42]);
    cpu.registers.a = 0x00;
    cpu.status.0 = 0x00;
    bus.memory[0x0042] = 0x55;

    run_instruction(&mut cpu, &mut bus);

    // storing zero does not set Z
    assert_eq!(bus.memory[0x0042], 0x00);
    assert_eq!(cpu.status.0, 0x00);
}

#[test]
fn stx_zeropage() {
    let (mut cpu, mut bus) = setup(&[STX_ZEROPAGE, 0x42]);
    cpu.registers.x = 0x80;
    cpu.registers.a = 0x11;
    cpu.registers.y = 0x33;
    // every flag set except B: a store changes none of them
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;
    bus.memory[0x0042] = 0x55;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0042], 0x80);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(STX_ZEROPAGE));
    // the target is written without being read first
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Write(0x0042, 0x80),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, 0xFF & !StatusFlags::BREAK_COMMAND);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x80);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn stx_zeropage_does_not_change_flags_for_zero() {
    let (mut cpu, mut bus) = setup(&[STX_ZEROPAGE, 0x42]);
    cpu.registers.x = 0x00;
    cpu.status.0 = 0x00;
    bus.memory[0x0042] = 0x55;

    run_instruction(&mut cpu, &mut bus);

    // storing zero does not set Z
    assert_eq!(bus.memory[0x0042], 0x00);
    assert_eq!(cpu.status.0, 0x00);
}

#[test]
fn ldy_zeropage() {
    let (mut cpu, mut bus) = setup(&[LDY_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x80;
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
    assert_eq!(cycles, table_cycles(LDY_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0042),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0042], 0x80);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x22);
    assert_eq!(cpu.registers.x, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ldy_zeropage_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDY_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x00;
    cpu.registers.y = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.y, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lda_zeropage() {
    let (mut cpu, mut bus) = setup(&[LDA_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x80;
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
    assert_eq!(cycles, table_cycles(LDA_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0042),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0042], 0x80);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn lda_zeropage_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDA_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x00;
    cpu.registers.a = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ldx_zeropage() {
    let (mut cpu, mut bus) = setup(&[LDX_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x80;
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
    assert_eq!(cycles, table_cycles(LDX_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0042),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0042], 0x80);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ldx_zeropage_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDX_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x00;
    cpu.registers.x = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.x, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cpy_zeropage() {
    // Y greater than the operand: C set, Z clear, N from $50 - $30 = $20
    let (mut cpu, mut bus) = setup(&[CPY_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x30;
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
    assert_eq!(cycles, table_cycles(CPY_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0042),
        ]
    );

    // nothing else changed: the compare does not store its result
    assert_eq!(cpu.registers.y, 0x50);
    assert_eq!(bus.memory[0x0042], 0x30);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn cpy_zeropage_equal_sets_zero_and_carry() {
    let (mut cpu, mut bus) = setup(&[CPY_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x42;
    cpu.registers.y = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cpy_zeropage_less_than_clears_carry() {
    // $30 - $50 borrows; the wrapped difference $E0 has bit 7 set
    let (mut cpu, mut bus) = setup(&[CPY_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x50;
    cpu.registers.y = 0x30;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cmp_zeropage() {
    // A greater than the operand: C set, Z clear, N from $50 - $30 = $20
    let (mut cpu, mut bus) = setup(&[CMP_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x30;
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
    assert_eq!(cycles, table_cycles(CMP_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0042),
        ]
    );

    // nothing else changed: the compare does not store its result
    assert_eq!(cpu.registers.a, 0x50);
    assert_eq!(bus.memory[0x0042], 0x30);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn cmp_zeropage_equal_sets_zero_and_carry() {
    let (mut cpu, mut bus) = setup(&[CMP_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x42;
    cpu.registers.a = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cmp_zeropage_less_than_clears_carry() {
    // $30 - $50 borrows; the wrapped difference $E0 has bit 7 set
    let (mut cpu, mut bus) = setup(&[CMP_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x50;
    cpu.registers.a = 0x30;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn dec_zeropage() {
    let (mut cpu, mut bus) = setup(&[DEC_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x81;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0042], 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(DEC_ZEROPAGE));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0042),
            Access::Write(0x0042, 0x81),
            Access::Write(0x0042, 0x80),
        ]
    );

    // nothing else changed
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn dec_zeropage_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[DEC_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x01;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0042], 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn dec_zeropage_wraps_from_zero_to_ff() {
    let (mut cpu, mut bus) = setup(&[DEC_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x00;
    cpu.status.set(StatusFlags::ZERO, true);
    // DEC does not touch carry, unlike a subtraction
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0042], 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn cpx_zeropage() {
    // X greater than the operand: C set, Z clear, N from $50 - $30 = $20
    let (mut cpu, mut bus) = setup(&[CPX_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x30;
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
    assert_eq!(cycles, table_cycles(CPX_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0042),
        ]
    );

    // nothing else changed: the compare does not store its result
    assert_eq!(cpu.registers.x, 0x50);
    assert_eq!(bus.memory[0x0042], 0x30);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn cpx_zeropage_equal_sets_zero_and_carry() {
    let (mut cpu, mut bus) = setup(&[CPX_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x42;
    cpu.registers.x = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cpx_zeropage_less_than_clears_carry() {
    // $30 - $50 borrows; the wrapped difference $E0 has bit 7 set
    let (mut cpu, mut bus) = setup(&[CPX_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x50;
    cpu.registers.x = 0x30;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn sbc_zeropage() {
    // carry set means no borrow: $50 - $20 = $30
    let (mut cpu, mut bus) = setup(&[SBC_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x20;
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
    assert_eq!(cycles, table_cycles(SBC_ZEROPAGE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0042),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0042], 0x20);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sbc_zeropage_clears_carry_on_borrow() {
    // $20 - $50 = -$30: the unsigned result wraps to $D0 and borrows
    let (mut cpu, mut bus) = setup(&[SBC_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x50;
    cpu.registers.a = 0x20;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xD0);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
}

#[test]
fn inc_zeropage() {
    let (mut cpu, mut bus) = setup(&[INC_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0x7F;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0042], 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(INC_ZEROPAGE));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0042),
            Access::Write(0x0042, 0x7F),
            Access::Write(0x0042, 0x80),
        ]
    );

    // nothing else changed
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn inc_zeropage_wraps_from_ff_to_zero() {
    let (mut cpu, mut bus) = setup(&[INC_ZEROPAGE, 0x42]);
    bus.memory[0x0042] = 0xFF;
    cpu.status.set(StatusFlags::NEGATIVE, true);
    // INC does not touch carry, unlike an addition
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0042], 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

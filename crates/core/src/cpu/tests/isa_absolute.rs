use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_ABSOLUTE: u8 = 0x0D;
const ASL_ABSOLUTE: u8 = 0x0E;
const JSR: u8 = 0x20;
const BIT_ABSOLUTE: u8 = 0x2C;
const AND_ABSOLUTE: u8 = 0x2D;
const ROL_ABSOLUTE: u8 = 0x2E;
const JMP_ABSOLUTE: u8 = 0x4C;
const EOR_ABSOLUTE: u8 = 0x4D;
const LSR_ABSOLUTE: u8 = 0x4E;

#[test]
fn ora_absolute() {
    // operand is little-endian: $1234
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0xF0;
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ORA_ABSOLUTE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0xF0);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn ora_absolute_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x00;
    cpu.registers.a = 0x00;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ora_absolute_clears_zero_and_negative_flags() {
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x01;
    cpu.registers.a = 0x40;
    cpu.status.set(StatusFlags::ZERO, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x41);
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn asl_absolute() {
    let (mut cpu, mut bus) = setup(&[ASL_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x41;
    cpu.registers.a = 0x55;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x82);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ASL_ABSOLUTE));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234),
            Access::Write(0x1234, 0x41),
            Access::Write(0x1234, 0x82),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn asl_absolute_shifts_bit_7_into_carry() {
    let (mut cpu, mut bus) = setup(&[ASL_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x80;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn jsr() {
    let (mut cpu, mut bus) = setup(&[JSR, 0x34, 0x12]);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.registers.sp, 0xFB);
    assert_eq!(cycles, table_cycles(JSR));
    // the pushed return address is the last byte of the JSR ($0202), not the
    // next instruction; RTS adds one when it pulls it
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x01FD), // dummy read of the stack
            Access::Write(0x01FD, 0x02),
            Access::Write(0x01FC, 0x02),
            Access::Read(PROGRAM_START + 2),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, status_before);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
}

#[test]
fn jsr_stack_pointer_wraps() {
    let (mut cpu, mut bus) = setup(&[JSR, 0x34, 0x12]);
    cpu.registers.sp = 0x00;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.registers.sp, 0xFE);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0100),
            Access::Write(0x0100, 0x02),
            Access::Write(0x01FF, 0x02),
            Access::Read(PROGRAM_START + 2),
        ]
    );
}

#[test]
fn bit_absolute() {
    let (mut cpu, mut bus) = setup(&[BIT_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0xC0;
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    // $0F AND $C0 is zero; N and V are bits 7 and 6 of the operand
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(BIT_ABSOLUTE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed, the accumulator included
    assert_eq!(cpu.registers.a, 0x0F);
    assert_eq!(bus.memory[0x1234], 0xC0);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn bit_absolute_clears_zero_negative_and_overflow() {
    let (mut cpu, mut bus) = setup(&[BIT_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x01;
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
fn bit_absolute_negative_and_overflow_come_from_operand_not_result() {
    let (mut cpu, mut bus) = setup(&[BIT_ABSOLUTE, 0x34, 0x12]);
    // operand has bit 6 set and bit 7 clear; the AND with A is zero
    bus.memory[0x1234] = 0x40;
    cpu.registers.a = 0x80;

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
}

#[test]
fn and_absolute() {
    let (mut cpu, mut bus) = setup(&[AND_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x9F;
    cpu.registers.a = 0xF0;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x90);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(AND_ABSOLUTE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0x9F);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn and_absolute_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[AND_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x0F;
    cpu.registers.a = 0xF0;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn rol_absolute() {
    let (mut cpu, mut bus) = setup(&[ROL_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x41;
    cpu.registers.a = 0x55;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x82);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ROL_ABSOLUTE));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234),
            Access::Write(0x1234, 0x41),
            Access::Write(0x1234, 0x82),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn rol_absolute_rotates_carry_into_bit_0() {
    let (mut cpu, mut bus) = setup(&[ROL_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x01;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x03);
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn rol_absolute_rotates_bit_7_into_carry() {
    let (mut cpu, mut bus) = setup(&[ROL_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x80;
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn jmp_absolute() {
    let (mut cpu, mut bus) = setup(&[JMP_ABSOLUTE, 0x34, 0x12]);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cycles, table_cycles(JMP_ABSOLUTE));
    // no access to the target itself: the next op-code fetch reads it
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, status_before);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn jmp_absolute_then_executes_at_target() {
    let (mut cpu, mut bus) = setup(&[JMP_ABSOLUTE, 0x34, 0x12]);
    // ORA #$42 at the target
    bus.memory[0x1234] = 0x09;
    bus.memory[0x1235] = 0x42;

    run_instruction(&mut cpu, &mut bus);
    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.pc, 0x1236);
}

#[test]
fn eor_absolute() {
    let (mut cpu, mut bus) = setup(&[EOR_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0xFF;
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xF0);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(EOR_ABSOLUTE));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0xFF);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn eor_absolute_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[EOR_ABSOLUTE, 0x34, 0x12]);
    // a value XOR itself is zero
    bus.memory[0x1234] = 0xA5;
    cpu.registers.a = 0xA5;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lsr_absolute() {
    let (mut cpu, mut bus) = setup(&[LSR_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x82;
    cpu.registers.a = 0x55;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x41);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(LSR_ABSOLUTE));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234),
            Access::Write(0x1234, 0x82),
            Access::Write(0x1234, 0x41),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn lsr_absolute_shifts_bit_0_into_carry() {
    let (mut cpu, mut bus) = setup(&[LSR_ABSOLUTE, 0x34, 0x12]);
    bus.memory[0x1234] = 0x01;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lsr_absolute_shifts_zero_into_bit_7() {
    let (mut cpu, mut bus) = setup(&[LSR_ABSOLUTE, 0x34, 0x12]);
    // carry set beforehand must not be shifted in, unlike ROR
    bus.memory[0x1234] = 0xFF;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x7F);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

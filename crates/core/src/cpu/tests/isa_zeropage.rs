use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_ZEROPAGE: u8 = 0x05;
const ASL_ZEROPAGE: u8 = 0x06;
const BIT_ZEROPAGE: u8 = 0x24;
const AND_ZEROPAGE: u8 = 0x25;
const ROL_ZEROPAGE: u8 = 0x26;
const EOR_ZEROPAGE: u8 = 0x45;
const LSR_ZEROPAGE: u8 = 0x46;

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

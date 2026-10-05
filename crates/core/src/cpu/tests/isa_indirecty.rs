use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_INDIRECTY: u8 = 0x11;
const AND_INDIRECTY: u8 = 0x31;
const EOR_INDIRECTY: u8 = 0x51;

#[test]
fn ora_indirecty() {
    let (mut cpu, mut bus) = setup(&[ORA_INDIRECTY, 0x20]);
    // pointer at $20 holds the base address $1200, Y is added after the indirection
    bus.memory[0x0020] = 0x00;
    bus.memory[0x0021] = 0x12;
    bus.memory[0x1234] = 0xF0;
    cpu.registers.y = 0x34;
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ORA_INDIRECTY));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020),
            Access::Read(0x0021),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0xF0);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x34);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn ora_indirecty_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[ORA_INDIRECTY, 0x20]);
    bus.memory[0x0020] = 0x00;
    bus.memory[0x0021] = 0x12;
    bus.memory[0x1234] = 0x00;
    cpu.registers.y = 0x34;
    cpu.registers.a = 0x00;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ora_indirecty_page_cross_takes_extra_cycle() {
    let (mut cpu, mut bus) = setup(&[ORA_INDIRECTY, 0x20]);
    // base $12F0 + Y $20 = $1310, crossing from page $12 to $13
    bus.memory[0x0020] = 0xF0;
    bus.memory[0x0021] = 0x12;
    bus.memory[0x1310] = 0x02;
    // the dummy read hits $1210; its value must not reach the accumulator
    bus.memory[0x1210] = 0x80;
    cpu.registers.y = 0x20;
    cpu.registers.a = 0x01;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x03);
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ORA_INDIRECTY) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020),
            Access::Read(0x0021),
            Access::Read(0x1210), // dummy read: high byte not fixed yet
            Access::Read(0x1310),
        ]
    );
}

#[test]
fn ora_indirecty_pointer_high_byte_wraps_within_zero_page() {
    let (mut cpu, mut bus) = setup(&[ORA_INDIRECTY, 0xFF]);
    // pointer low byte at $FF, high byte wraps to $00 instead of $0100
    bus.memory[0x00FF] = 0x34;
    bus.memory[0x0000] = 0x12;
    bus.memory[0x0100] = 0x77;
    bus.memory[0x1234] = 0x42;
    cpu.registers.y = 0x00;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cycles, table_cycles(ORA_INDIRECTY));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x00FF),
            Access::Read(0x0000),
            Access::Read(0x1234),
        ]
    );
}

#[test]
fn ora_indirecty_effective_address_wraps_past_ffff() {
    let (mut cpu, mut bus) = setup(&[ORA_INDIRECTY, 0x20]);
    // base $FFF0 + Y $20 wraps to $0010
    bus.memory[0x0020] = 0xF0;
    bus.memory[0x0021] = 0xFF;
    bus.memory[0x0010] = 0x42;
    cpu.registers.y = 0x20;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cycles, table_cycles(ORA_INDIRECTY) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020),
            Access::Read(0x0021),
            Access::Read(0xFF10), // dummy read: high byte not fixed yet
            Access::Read(0x0010),
        ]
    );
}

#[test]
fn and_indirecty() {
    let (mut cpu, mut bus) = setup(&[AND_INDIRECTY, 0x20]);
    // pointer at $20 holds the base address $1200, Y is added after the indirection
    bus.memory[0x0020] = 0x00;
    bus.memory[0x0021] = 0x12;
    bus.memory[0x1234] = 0x9F;
    cpu.registers.y = 0x34;
    cpu.registers.a = 0xF0;
    cpu.registers.x = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x90);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(AND_INDIRECTY));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020),
            Access::Read(0x0021),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0x9F);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x34);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn and_indirecty_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[AND_INDIRECTY, 0x20]);
    bus.memory[0x0020] = 0x00;
    bus.memory[0x0021] = 0x12;
    bus.memory[0x1234] = 0x0F;
    cpu.registers.y = 0x34;
    cpu.registers.a = 0xF0;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn and_indirecty_page_cross_takes_extra_cycle() {
    let (mut cpu, mut bus) = setup(&[AND_INDIRECTY, 0x20]);
    // base $12F0 + Y $20 = $1310, crossing from page $12 to $13
    bus.memory[0x0020] = 0xF0;
    bus.memory[0x0021] = 0x12;
    bus.memory[0x1310] = 0x0F;
    // the dummy read hits $1210; its value must not reach the accumulator
    bus.memory[0x1210] = 0xF0;
    cpu.registers.y = 0x20;
    cpu.registers.a = 0x3C;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x0C);
    assert_eq!(cycles, table_cycles(AND_INDIRECTY) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020),
            Access::Read(0x0021),
            Access::Read(0x1210), // dummy read: high byte not fixed yet
            Access::Read(0x1310),
        ]
    );
}

#[test]
fn eor_indirecty() {
    let (mut cpu, mut bus) = setup(&[EOR_INDIRECTY, 0x20]);
    // pointer at $20 holds the base address $1200, Y is added after the indirection
    bus.memory[0x0020] = 0x00;
    bus.memory[0x0021] = 0x12;
    bus.memory[0x1234] = 0xFF;
    cpu.registers.y = 0x34;
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xF0);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(EOR_INDIRECTY));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020),
            Access::Read(0x0021),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x1234], 0xFF);
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x34);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn eor_indirecty_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[EOR_INDIRECTY, 0x20]);
    bus.memory[0x0020] = 0x00;
    bus.memory[0x0021] = 0x12;
    // a value XOR itself is zero
    bus.memory[0x1234] = 0xA5;
    cpu.registers.y = 0x34;
    cpu.registers.a = 0xA5;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn eor_indirecty_page_cross_takes_extra_cycle() {
    let (mut cpu, mut bus) = setup(&[EOR_INDIRECTY, 0x20]);
    // base $12F0 + Y $20 = $1310, crossing from page $12 to $13
    bus.memory[0x0020] = 0xF0;
    bus.memory[0x0021] = 0x12;
    bus.memory[0x1310] = 0x0F;
    // the dummy read hits $1210; its value must not reach the accumulator
    bus.memory[0x1210] = 0xF0;
    cpu.registers.y = 0x20;
    cpu.registers.a = 0x3C;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x33);
    assert_eq!(cycles, table_cycles(EOR_INDIRECTY) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020),
            Access::Read(0x0021),
            Access::Read(0x1210), // dummy read: high byte not fixed yet
            Access::Read(0x1310),
        ]
    );
}

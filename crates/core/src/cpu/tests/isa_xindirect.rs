use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_XINDIRECT: u8 = 0x01;
const AND_XINDIRECT: u8 = 0x21;
const EOR_XINDIRECT: u8 = 0x41;

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

use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const STX_ZEROPAGEY: u8 = 0x96;
const LDX_ZEROPAGEY: u8 = 0xB6;

#[test]
fn stx_zeropagey() {
    let (mut cpu, mut bus) = setup(&[STX_ZEROPAGEY, 0x20]);
    // effective address is $20 + Y = $24
    bus.memory[0x0024] = 0x55;
    bus.memory[0x0020] = 0x99;
    cpu.registers.y = 0x04;
    cpu.registers.x = 0x80;
    cpu.registers.a = 0x11;
    // every flag set except B: a store changes none of them
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x80);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(STX_ZEROPAGEY));
    // the target is written without being read first
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed address
            Access::Write(0x0024, 0x80),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0020], 0x99);
    assert_eq!(cpu.status.0, 0xFF & !StatusFlags::BREAK_COMMAND);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x80);
    assert_eq!(cpu.registers.y, 0x04);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn stx_zeropagey_indexes_with_y_not_x() {
    let (mut cpu, mut bus) = setup(&[STX_ZEROPAGEY, 0x20]);
    cpu.registers.x = 0x42;
    cpu.registers.y = 0x04;

    run_instruction(&mut cpu, &mut bus);

    // $20 + Y = $24; indexing with X would hit $62
    assert_eq!(bus.memory[0x0024], 0x42);
    assert_eq!(bus.memory[0x0062], 0x00);
}

#[test]
fn stx_zeropagey_index_wraps_within_zero_page() {
    let (mut cpu, mut bus) = setup(&[STX_ZEROPAGEY, 0xF0]);
    cpu.registers.y = 0x20;
    cpu.registers.x = 0x42;

    run_instruction(&mut cpu, &mut bus);

    // $F0 + $20 wraps to $10, not $0110
    assert_eq!(bus.memory[0x0010], 0x42);
    assert_eq!(bus.memory[0x0110], 0x00);
}

#[test]
fn ldx_zeropagey() {
    let (mut cpu, mut bus) = setup(&[LDX_ZEROPAGEY, 0x20]);
    // effective address is $20 + Y = $24
    bus.memory[0x0024] = 0x80;
    // the dummy read hits $20; its value must not reach X
    bus.memory[0x0020] = 0x01;
    cpu.registers.y = 0x04;
    cpu.registers.x = 0x11;
    cpu.registers.a = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.x, 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(LDX_ZEROPAGEY));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed address
            Access::Read(0x0024),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0024], 0x80);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x22);
    assert_eq!(cpu.registers.y, 0x04);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ldx_zeropagey_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDX_ZEROPAGEY, 0x20]);
    bus.memory[0x0024] = 0x00;
    cpu.registers.y = 0x04;
    cpu.registers.x = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.x, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ldx_zeropagey_indexes_with_y_not_x() {
    let (mut cpu, mut bus) = setup(&[LDX_ZEROPAGEY, 0x20]);
    bus.memory[0x0024] = 0x42;
    bus.memory[0x0030] = 0x99;
    cpu.registers.y = 0x04;
    cpu.registers.x = 0x10;

    run_instruction(&mut cpu, &mut bus);

    // $20 + Y = $24; indexing with X would hit $30
    assert_eq!(cpu.registers.x, 0x42);
}

#[test]
fn ldx_zeropagey_index_wraps_within_zero_page() {
    let (mut cpu, mut bus) = setup(&[LDX_ZEROPAGEY, 0xF0]);
    bus.memory[0x0010] = 0x42;
    bus.memory[0x0110] = 0x99;
    cpu.registers.y = 0x20;

    run_instruction(&mut cpu, &mut bus);

    // $F0 + $20 wraps to $10, not $0110
    assert_eq!(cpu.registers.x, 0x42);
}

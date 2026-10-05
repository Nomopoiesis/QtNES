use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_ABSOLUTEX: u8 = 0x1D;
const ASL_ABSOLUTEX: u8 = 0x1E;

#[test]
fn ora_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0xF0;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x0F;
    cpu.registers.y = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ORA_ABSOLUTEX));
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
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x11);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn ora_absolutex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x00;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x00;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ora_absolutex_page_cross_takes_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x02;
    // the dummy read hits $1210; its value must not reach the accumulator
    bus.memory[0x1210] = 0x80;
    cpu.registers.x = 0x20;
    cpu.registers.a = 0x01;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x03);
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ORA_ABSOLUTEX) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1210), // dummy read: high byte not fixed yet
            Access::Read(0x1310),
        ]
    );
}

#[test]
fn ora_absolutex_effective_address_wraps_past_ffff() {
    // base $FFF0 + X $20 wraps to $0010
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTEX, 0xF0, 0xFF]);
    bus.memory[0x0010] = 0x42;
    cpu.registers.x = 0x20;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cycles, table_cycles(ORA_ABSOLUTEX) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0xFF10), // dummy read: high byte not fixed yet
            Access::Read(0x0010),
        ]
    );
}

#[test]
fn asl_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[ASL_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x41;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x82);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ASL_ABSOLUTEX));
    // the dummy read happens even without a page cross, then the unmodified
    // value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234), // dummy read
            Access::Read(0x1234),
            Access::Write(0x1234, 0x41),
            Access::Write(0x1234, 0x82),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x11);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn asl_absolutex_shifts_bit_7_into_carry() {
    let (mut cpu, mut bus) = setup(&[ASL_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x80;
    cpu.registers.x = 0x34;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn asl_absolutex_page_cross_takes_no_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[ASL_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x01;
    // the dummy read hits $1210, which must not be modified
    bus.memory[0x1210] = 0x40;
    cpu.registers.x = 0x20;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1310], 0x02);
    assert_eq!(bus.memory[0x1210], 0x40);
    assert_eq!(cycles, table_cycles(ASL_ABSOLUTEX));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1210), // dummy read: high byte not fixed yet
            Access::Read(0x1310),
            Access::Write(0x1310, 0x01),
            Access::Write(0x1310, 0x02),
        ]
    );
}

#[test]
fn asl_absolutex_effective_address_wraps_past_ffff() {
    // base $FFF0 + X $20 wraps to $0010
    let (mut cpu, mut bus) = setup(&[ASL_ABSOLUTEX, 0xF0, 0xFF]);
    bus.memory[0x0010] = 0x01;
    cpu.registers.x = 0x20;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x02);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0xFF10), // dummy read: high byte not fixed yet
            Access::Read(0x0010),
            Access::Write(0x0010, 0x01),
            Access::Write(0x0010, 0x02),
        ]
    );
}

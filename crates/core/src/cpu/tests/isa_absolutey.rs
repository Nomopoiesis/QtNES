use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_ABSOLUTEY: u8 = 0x19;

#[test]
fn ora_absolutey() {
    // base address $1200 + Y $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTEY, 0x00, 0x12]);
    bus.memory[0x1234] = 0xF0;
    cpu.registers.y = 0x34;
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ORA_ABSOLUTEY));
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
    assert_eq!(cpu.registers.y, 0x34);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn ora_absolutey_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTEY, 0x00, 0x12]);
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
fn ora_absolutey_page_cross_takes_extra_cycle() {
    // base $12F0 + Y $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTEY, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x02;
    // the dummy read hits $1210; its value must not reach the accumulator
    bus.memory[0x1210] = 0x80;
    cpu.registers.y = 0x20;
    cpu.registers.a = 0x01;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x03);
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ORA_ABSOLUTEY) + 1);
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
fn ora_absolutey_effective_address_wraps_past_ffff() {
    // base $FFF0 + Y $20 wraps to $0010
    let (mut cpu, mut bus) = setup(&[ORA_ABSOLUTEY, 0xF0, 0xFF]);
    bus.memory[0x0010] = 0x42;
    cpu.registers.y = 0x20;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cycles, table_cycles(ORA_ABSOLUTEY) + 1);
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

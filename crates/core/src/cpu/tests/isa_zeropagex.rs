use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_ZEROPAGEX: u8 = 0x15;
const ASL_ZEROPAGEX: u8 = 0x16;

#[test]
fn ora_zeropagex() {
    let (mut cpu, mut bus) = setup(&[ORA_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0xF0;
    // the dummy read hits $20; its value must not reach the accumulator
    bus.memory[0x0020] = 0x01;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x0F;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ORA_ZEROPAGEX));
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
    assert_eq!(bus.memory[0x0024], 0xF0);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn ora_zeropagex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[ORA_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x00;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x00;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ora_zeropagex_index_wraps_within_zero_page() {
    let (mut cpu, mut bus) = setup(&[ORA_ZEROPAGEX, 0xF0]);
    // $F0 + $20 wraps to $10, not $0110
    bus.memory[0x0010] = 0x42;
    bus.memory[0x0110] = 0x77;
    cpu.registers.x = 0x20;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
    // no extra cycle: zero-page indexing never crosses a page
    assert_eq!(cycles, table_cycles(ORA_ZEROPAGEX));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x00F0),
            Access::Read(0x0010),
        ]
    );
}

#[test]
fn asl_zeropagex() {
    let (mut cpu, mut bus) = setup(&[ASL_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x41;
    bus.memory[0x0020] = 0x99;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x82);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ASL_ZEROPAGEX));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed address
            Access::Read(0x0024),
            Access::Write(0x0024, 0x41),
            Access::Write(0x0024, 0x82),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0020], 0x99);
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn asl_zeropagex_shifts_bit_7_into_carry() {
    let (mut cpu, mut bus) = setup(&[ASL_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x80;
    cpu.registers.x = 0x04;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn asl_zeropagex_index_wraps_within_zero_page() {
    let (mut cpu, mut bus) = setup(&[ASL_ZEROPAGEX, 0xF0]);
    // $F0 + $20 wraps to $10, not $0110
    bus.memory[0x0010] = 0x01;
    bus.memory[0x0110] = 0x77;
    cpu.registers.x = 0x20;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x02);
    assert_eq!(bus.memory[0x0110], 0x77);
    assert_eq!(cycles, table_cycles(ASL_ZEROPAGEX));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x00F0),
            Access::Read(0x0010),
            Access::Write(0x0010, 0x01),
            Access::Write(0x0010, 0x02),
        ]
    );
}

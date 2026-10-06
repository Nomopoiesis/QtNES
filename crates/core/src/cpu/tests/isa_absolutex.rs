use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_ABSOLUTEX: u8 = 0x1D;
const ASL_ABSOLUTEX: u8 = 0x1E;
const AND_ABSOLUTEX: u8 = 0x3D;
const ROL_ABSOLUTEX: u8 = 0x3E;
const EOR_ABSOLUTEX: u8 = 0x5D;
const LSR_ABSOLUTEX: u8 = 0x5E;
const ADC_ABSOLUTEX: u8 = 0x7D;
const ROR_ABSOLUTEX: u8 = 0x7E;
const STA_ABSOLUTEX: u8 = 0x9D;
const LDY_ABSOLUTEX: u8 = 0xBC;
const LDA_ABSOLUTEX: u8 = 0xBD;
const CMP_ABSOLUTEX: u8 = 0xDD;
const DEC_ABSOLUTEX: u8 = 0xDE;
const SBC_ABSOLUTEX: u8 = 0xFD;
const INC_ABSOLUTEX: u8 = 0xFE;

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

#[test]
fn and_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[AND_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x9F;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0xF0;
    cpu.registers.y = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x90);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(AND_ABSOLUTEX));
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
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x11);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn and_absolutex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[AND_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x0F;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0xF0;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn and_absolutex_page_cross_takes_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[AND_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x0F;
    // the dummy read hits $1210; its value must not reach the accumulator
    bus.memory[0x1210] = 0xF0;
    cpu.registers.x = 0x20;
    cpu.registers.a = 0x3C;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x0C);
    assert_eq!(cycles, table_cycles(AND_ABSOLUTEX) + 1);
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
fn rol_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[ROL_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x41;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x11;
    cpu.status.set(StatusFlags::CARRY, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x82);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ROL_ABSOLUTEX));
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
fn rol_absolutex_rotates_carry_into_bit_0() {
    let (mut cpu, mut bus) = setup(&[ROL_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x01;
    cpu.registers.x = 0x34;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x03);
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn rol_absolutex_rotates_bit_7_into_carry() {
    let (mut cpu, mut bus) = setup(&[ROL_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x80;
    cpu.registers.x = 0x34;
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn rol_absolutex_page_cross_takes_no_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[ROL_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x01;
    // the dummy read hits $1210, which must not be modified
    bus.memory[0x1210] = 0x40;
    cpu.registers.x = 0x20;
    cpu.status.set(StatusFlags::CARRY, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1310], 0x02);
    assert_eq!(bus.memory[0x1210], 0x40);
    assert_eq!(cycles, table_cycles(ROL_ABSOLUTEX));
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
fn eor_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[EOR_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0xFF;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x0F;
    cpu.registers.y = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xF0);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(EOR_ABSOLUTEX));
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
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x11);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn eor_absolutex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[EOR_ABSOLUTEX, 0x00, 0x12]);
    // a value XOR itself is zero
    bus.memory[0x1234] = 0xA5;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0xA5;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn eor_absolutex_page_cross_takes_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[EOR_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x0F;
    // the dummy read hits $1210; its value must not reach the accumulator
    bus.memory[0x1210] = 0xF0;
    cpu.registers.x = 0x20;
    cpu.registers.a = 0x3C;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x33);
    assert_eq!(cycles, table_cycles(EOR_ABSOLUTEX) + 1);
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
fn lsr_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[LSR_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x82;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x41);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(LSR_ABSOLUTEX));
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
            Access::Write(0x1234, 0x82),
            Access::Write(0x1234, 0x41),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x11);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn lsr_absolutex_shifts_bit_0_into_carry() {
    let (mut cpu, mut bus) = setup(&[LSR_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x01;
    cpu.registers.x = 0x34;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lsr_absolutex_shifts_zero_into_bit_7() {
    let (mut cpu, mut bus) = setup(&[LSR_ABSOLUTEX, 0x00, 0x12]);
    // carry set beforehand must not be shifted in, unlike ROR
    bus.memory[0x1234] = 0xFF;
    cpu.registers.x = 0x34;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x7F);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lsr_absolutex_page_cross_takes_no_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[LSR_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x02;
    // the dummy read hits $1210, which must not be modified
    bus.memory[0x1210] = 0x40;
    cpu.registers.x = 0x20;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1310], 0x01);
    assert_eq!(bus.memory[0x1210], 0x40);
    assert_eq!(cycles, table_cycles(LSR_ABSOLUTEX));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1210), // dummy read: high byte not fixed yet
            Access::Read(0x1310),
            Access::Write(0x1310, 0x02),
            Access::Write(0x1310, 0x01),
        ]
    );
}

#[test]
fn sta_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[STA_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x55;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x80;
    cpu.registers.y = 0x22;
    // every flag set except B: a store changes none of them
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x80);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    // a store always takes the extra cycle, even without a page cross
    assert_eq!(cycles, table_cycles(STA_ABSOLUTEX));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234), // dummy read before the write
            Access::Write(0x1234, 0x80),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, 0xFF & !StatusFlags::BREAK_COMMAND);
    assert_eq!(cpu.registers.a, 0x80);
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sta_absolutex_page_cross_takes_no_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[STA_ABSOLUTEX, 0xF0, 0x12]);
    cpu.registers.x = 0x20;
    cpu.registers.a = 0x42;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1310], 0x42);
    // the un-fixed address is only read, never written
    assert_eq!(bus.memory[0x1210], 0x00);
    assert_eq!(cycles, table_cycles(STA_ABSOLUTEX));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1210), // dummy read: high byte not fixed yet
            Access::Write(0x1310, 0x42),
        ]
    );
}

#[test]
fn sta_absolutex_effective_address_wraps_past_ffff() {
    // base $FFF0 + X $20 wraps to $0010
    let (mut cpu, mut bus) = setup(&[STA_ABSOLUTEX, 0xF0, 0xFF]);
    cpu.registers.x = 0x20;
    cpu.registers.a = 0x42;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0010], 0x42);
    assert_eq!(
        bus.accesses[3..],
        [Access::Read(0xFF10), Access::Write(0x0010, 0x42)]
    );
}

#[test]
fn ldy_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[LDY_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x80;
    cpu.registers.x = 0x34;
    cpu.registers.y = 0x11;
    cpu.registers.a = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.y, 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(LDY_ABSOLUTEX));
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
    assert_eq!(bus.memory[0x1234], 0x80);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x22);
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ldy_absolutex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDY_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x00;
    cpu.registers.x = 0x34;
    cpu.registers.y = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.y, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ldy_absolutex_page_cross_takes_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[LDY_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x42;
    // the dummy read hits $1210; its value must not reach Y
    bus.memory[0x1210] = 0x99;
    cpu.registers.x = 0x20;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.y, 0x42);
    assert_eq!(cycles, table_cycles(LDY_ABSOLUTEX) + 1);
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
fn lda_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[LDA_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x80;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(LDA_ABSOLUTEX));
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
    assert_eq!(bus.memory[0x1234], 0x80);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn lda_absolutex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDA_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x00;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lda_absolutex_page_cross_takes_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[LDA_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x42;
    // the dummy read hits $1210; its value must not reach the accumulator
    bus.memory[0x1210] = 0x99;
    cpu.registers.x = 0x20;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cycles, table_cycles(LDA_ABSOLUTEX) + 1);
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
fn cmp_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    // A greater than the operand: C set, Z clear, N from $50 - $30 = $20
    let (mut cpu, mut bus) = setup(&[CMP_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x30;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x50;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(CMP_ABSOLUTEX));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234),
        ]
    );

    // nothing else changed: the compare does not store its result
    assert_eq!(cpu.registers.a, 0x50);
    assert_eq!(bus.memory[0x1234], 0x30);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn cmp_absolutex_equal_sets_zero_and_carry() {
    let (mut cpu, mut bus) = setup(&[CMP_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x42;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cmp_absolutex_less_than_clears_carry() {
    // $30 - $50 borrows; the wrapped difference $E0 has bit 7 set
    let (mut cpu, mut bus) = setup(&[CMP_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x50;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x30;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn dec_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[DEC_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x81;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(DEC_ABSOLUTEX));
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
            Access::Write(0x1234, 0x81),
            Access::Write(0x1234, 0x80),
        ]
    );

    // nothing else changed
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x11);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn dec_absolutex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[DEC_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x01;
    cpu.registers.x = 0x34;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn dec_absolutex_wraps_from_zero_to_ff() {
    let (mut cpu, mut bus) = setup(&[DEC_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x00;
    cpu.registers.x = 0x34;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn adc_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[ADC_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x20;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x10;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    // $10 + $20 + carry
    assert_eq!(cpu.registers.a, 0x31);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ADC_ABSOLUTEX));
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
    assert_eq!(bus.memory[0x1234], 0x20);
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn adc_absolutex_sets_carry_and_overflow() {
    let (mut cpu, mut bus) = setup(&[ADC_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x90;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0xD0;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    // $D0 + $90 = $160: two negatives giving a positive is a signed overflow
    assert_eq!(cpu.registers.a, 0x60);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn adc_absolutex_page_cross_takes_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[ADC_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x02;
    // the dummy read hits $1210; its value must not reach the accumulator
    bus.memory[0x1210] = 0x80;
    cpu.registers.x = 0x20;
    cpu.registers.a = 0x01;
    cpu.status.set(StatusFlags::CARRY, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x03);
    assert_eq!(cycles, table_cycles(ADC_ABSOLUTEX) + 1);
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
fn ror_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    // carry in goes to bit 7, bit 0 goes out to carry
    let (mut cpu, mut bus) = setup(&[ROR_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x02;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x81);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(ROR_ABSOLUTEX));
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
            Access::Write(0x1234, 0x02),
            Access::Write(0x1234, 0x81),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x11);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ror_absolutex_shifts_bit_0_into_carry() {
    let (mut cpu, mut bus) = setup(&[ROR_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x01;
    cpu.registers.x = 0x34;
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn sbc_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    // carry set means no borrow: $50 - $20 = $30
    let (mut cpu, mut bus) = setup(&[SBC_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x20;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x50;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x30);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(SBC_ABSOLUTEX));
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
    assert_eq!(bus.memory[0x1234], 0x20);
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sbc_absolutex_clears_carry_on_borrow() {
    // $20 - $50 = -$30: the unsigned result wraps to $D0 and borrows
    let (mut cpu, mut bus) = setup(&[SBC_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x50;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x20;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xD0);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn sbc_absolutex_page_cross_takes_extra_cycle() {
    // base $12F0 + X $20 = $1310, crossing from page $12 to $13
    let (mut cpu, mut bus) = setup(&[SBC_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x01;
    // the dummy read hits $1210; its value must not reach the accumulator
    bus.memory[0x1210] = 0x40;
    cpu.registers.x = 0x20;
    cpu.registers.a = 0x10;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x0F);
    assert_eq!(cycles, table_cycles(SBC_ABSOLUTEX) + 1);
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
fn inc_absolutex() {
    // base address $1200 + X $34 = $1234, no page cross
    let (mut cpu, mut bus) = setup(&[INC_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0x7F;
    cpu.registers.x = 0x34;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x11;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cycles, table_cycles(INC_ABSOLUTEX));
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
            Access::Write(0x1234, 0x7F),
            Access::Write(0x1234, 0x80),
        ]
    );

    // nothing else changed
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x34);
    assert_eq!(cpu.registers.y, 0x11);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn inc_absolutex_wraps_from_ff_to_zero() {
    let (mut cpu, mut bus) = setup(&[INC_ABSOLUTEX, 0x00, 0x12]);
    bus.memory[0x1234] = 0xFF;
    cpu.registers.x = 0x34;
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1234], 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn inc_absolutex_page_cross_writes_to_fixed_address() {
    // base $12F0 + X $20 = $1310: the dummy read hits $1210, but the
    // read-modify-write cycles all use the fixed address
    let (mut cpu, mut bus) = setup(&[INC_ABSOLUTEX, 0xF0, 0x12]);
    bus.memory[0x1310] = 0x41;
    bus.memory[0x1210] = 0x99;
    cpu.registers.x = 0x20;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x1310], 0x42);
    assert_eq!(bus.memory[0x1210], 0x99);
    // read-modify-write always takes 7 cycles, page cross or not
    assert_eq!(cycles, table_cycles(INC_ABSOLUTEX));
    assert_eq!(
        bus.accesses[3..],
        [
            Access::Read(0x1210), // dummy read: high byte not fixed yet
            Access::Read(0x1310),
            Access::Write(0x1310, 0x41),
            Access::Write(0x1310, 0x42),
        ]
    );
}

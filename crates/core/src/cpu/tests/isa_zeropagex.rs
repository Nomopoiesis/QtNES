use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_ZEROPAGEX: u8 = 0x15;
const ASL_ZEROPAGEX: u8 = 0x16;
const AND_ZEROPAGEX: u8 = 0x35;
const ROL_ZEROPAGEX: u8 = 0x36;
const EOR_ZEROPAGEX: u8 = 0x55;
const LSR_ZEROPAGEX: u8 = 0x56;
const ADC_ZEROPAGEX: u8 = 0x75;
const ROR_ZEROPAGEX: u8 = 0x76;
const STY_ZEROPAGEX: u8 = 0x94;
const STA_ZEROPAGEX: u8 = 0x95;
const LDY_ZEROPAGEX: u8 = 0xB4;
const LDA_ZEROPAGEX: u8 = 0xB5;
const CMP_ZEROPAGEX: u8 = 0xD5;
const DEC_ZEROPAGEX: u8 = 0xD6;
const SBC_ZEROPAGEX: u8 = 0xF5;
const INC_ZEROPAGEX: u8 = 0xF6;

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

#[test]
fn and_zeropagex() {
    let (mut cpu, mut bus) = setup(&[AND_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x9F;
    // the dummy read hits $20; its value must not reach the accumulator
    bus.memory[0x0020] = 0x0F;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0xF0;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x90);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(AND_ZEROPAGEX));
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
    assert_eq!(bus.memory[0x0024], 0x9F);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn and_zeropagex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[AND_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x0F;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0xF0;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn rol_zeropagex() {
    let (mut cpu, mut bus) = setup(&[ROL_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x41;
    bus.memory[0x0020] = 0x99;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x82);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ROL_ZEROPAGEX));
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
fn rol_zeropagex_rotates_carry_into_bit_0() {
    let (mut cpu, mut bus) = setup(&[ROL_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x01;
    cpu.registers.x = 0x04;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x03);
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn rol_zeropagex_rotates_bit_7_into_carry() {
    let (mut cpu, mut bus) = setup(&[ROL_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x80;
    cpu.registers.x = 0x04;
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn eor_zeropagex() {
    let (mut cpu, mut bus) = setup(&[EOR_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0xFF;
    // the dummy read hits $20; its value must not reach the accumulator
    bus.memory[0x0020] = 0x0F;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x0F;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xF0);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(EOR_ZEROPAGEX));
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
    assert_eq!(bus.memory[0x0024], 0xFF);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn eor_zeropagex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[EOR_ZEROPAGEX, 0x20]);
    // a value XOR itself is zero
    bus.memory[0x0024] = 0xA5;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0xA5;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lsr_zeropagex() {
    let (mut cpu, mut bus) = setup(&[LSR_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x82;
    bus.memory[0x0020] = 0x99;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x41);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(LSR_ZEROPAGEX));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed address
            Access::Read(0x0024),
            Access::Write(0x0024, 0x82),
            Access::Write(0x0024, 0x41),
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
fn lsr_zeropagex_shifts_bit_0_into_carry() {
    let (mut cpu, mut bus) = setup(&[LSR_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x01;
    cpu.registers.x = 0x04;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lsr_zeropagex_shifts_zero_into_bit_7() {
    let (mut cpu, mut bus) = setup(&[LSR_ZEROPAGEX, 0x20]);
    // carry set beforehand must not be shifted in, unlike ROR
    bus.memory[0x0024] = 0xFF;
    cpu.registers.x = 0x04;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x7F);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn adc_zeropagex() {
    let (mut cpu, mut bus) = setup(&[ADC_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x20;
    // the dummy read hits $20; its value must not reach the accumulator
    bus.memory[0x0020] = 0x0F;
    cpu.registers.x = 0x04;
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
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ADC_ZEROPAGEX));
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
    assert_eq!(bus.memory[0x0024], 0x20);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn adc_zeropagex_negative_overflow_sets_carry_and_overflow() {
    let (mut cpu, mut bus) = setup(&[ADC_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x80;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x80;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    // two negatives giving a positive result is a signed overflow
    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn ror_zeropagex() {
    let (mut cpu, mut bus) = setup(&[ROR_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24; carry in goes to bit 7, bit 0 out to carry
    bus.memory[0x0024] = 0x02;
    bus.memory[0x0020] = 0x99;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x81);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ROR_ZEROPAGEX));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed address
            Access::Read(0x0024),
            Access::Write(0x0024, 0x02),
            Access::Write(0x0024, 0x81),
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
fn ror_zeropagex_shifts_bit_0_into_carry() {
    let (mut cpu, mut bus) = setup(&[ROR_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x01;
    cpu.registers.x = 0x04;
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn sty_zeropagex() {
    let (mut cpu, mut bus) = setup(&[STY_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x55;
    bus.memory[0x0020] = 0x99;
    cpu.registers.x = 0x04;
    cpu.registers.y = 0x80;
    cpu.registers.a = 0x11;
    // every flag set except B: a store changes none of them
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x80);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(STY_ZEROPAGEX));
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
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x80);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sty_zeropagex_index_wraps_within_zero_page() {
    let (mut cpu, mut bus) = setup(&[STY_ZEROPAGEX, 0xF0]);
    cpu.registers.x = 0x20;
    cpu.registers.y = 0x42;

    run_instruction(&mut cpu, &mut bus);

    // $F0 + $20 wraps to $10, not $0110
    assert_eq!(bus.memory[0x0010], 0x42);
    assert_eq!(bus.memory[0x0110], 0x00);
}

#[test]
fn sta_zeropagex() {
    let (mut cpu, mut bus) = setup(&[STA_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x55;
    bus.memory[0x0020] = 0x99;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x80;
    cpu.registers.y = 0x22;
    // every flag set except B: a store changes none of them
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x80);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(STA_ZEROPAGEX));
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
    assert_eq!(cpu.registers.a, 0x80);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ldy_zeropagex() {
    let (mut cpu, mut bus) = setup(&[LDY_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x80;
    // the dummy read hits $20; its value must not reach Y
    bus.memory[0x0020] = 0x01;
    cpu.registers.x = 0x04;
    cpu.registers.y = 0x11;
    cpu.registers.a = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.y, 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(LDY_ZEROPAGEX));
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
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ldy_zeropagex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDY_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x00;
    cpu.registers.x = 0x04;
    cpu.registers.y = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.y, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lda_zeropagex() {
    let (mut cpu, mut bus) = setup(&[LDA_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x80;
    // the dummy read hits $20; its value must not reach the accumulator
    bus.memory[0x0020] = 0x01;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(LDA_ZEROPAGEX));
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
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn lda_zeropagex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[LDA_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x00;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cmp_zeropagex() {
    // A greater than the operand: C set, Z clear, N from $50 - $30 = $20
    let (mut cpu, mut bus) = setup(&[CMP_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x30;
    // the dummy read hits $20; comparing against it would give Z
    bus.memory[0x0020] = 0x50;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x50;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(CMP_ZEROPAGEX));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed address
            Access::Read(0x0024),
        ]
    );

    // nothing else changed: the compare does not store its result
    assert_eq!(cpu.registers.a, 0x50);
    assert_eq!(bus.memory[0x0024], 0x30);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn cmp_zeropagex_equal_sets_zero_and_carry() {
    let (mut cpu, mut bus) = setup(&[CMP_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x42;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cmp_zeropagex_less_than_clears_carry() {
    // $30 - $50 borrows; the wrapped difference $E0 has bit 7 set
    let (mut cpu, mut bus) = setup(&[CMP_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x50;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x30;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn dec_zeropagex() {
    let (mut cpu, mut bus) = setup(&[DEC_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x81;
    bus.memory[0x0020] = 0x99;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(DEC_ZEROPAGEX));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed address
            Access::Read(0x0024),
            Access::Write(0x0024, 0x81),
            Access::Write(0x0024, 0x80),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0020], 0x99);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn dec_zeropagex_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[DEC_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x01;
    cpu.registers.x = 0x04;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn dec_zeropagex_wraps_from_zero_to_ff() {
    let (mut cpu, mut bus) = setup(&[DEC_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x00;
    cpu.registers.x = 0x04;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn sbc_zeropagex() {
    // carry set means no borrow: $50 - $20 = $30
    let (mut cpu, mut bus) = setup(&[SBC_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x20;
    // the dummy read hits $20; its value must not reach the accumulator
    bus.memory[0x0020] = 0x01;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x50;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x30);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(SBC_ZEROPAGEX));
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
    assert_eq!(bus.memory[0x0024], 0x20);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sbc_zeropagex_sets_overflow_negative_minus_positive() {
    // -48 - 112 = -160, which does not fit in a signed byte
    let (mut cpu, mut bus) = setup(&[SBC_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0x70;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0xD0;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x60);
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn inc_zeropagex() {
    let (mut cpu, mut bus) = setup(&[INC_ZEROPAGEX, 0x20]);
    // effective address is $20 + X = $24
    bus.memory[0x0024] = 0x7F;
    bus.memory[0x0020] = 0x99;
    cpu.registers.x = 0x04;
    cpu.registers.a = 0x55;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x80);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(INC_ZEROPAGEX));
    // read-modify-write: the unmodified value is written back before the result
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(0x0020), // dummy read of the un-indexed address
            Access::Read(0x0024),
            Access::Write(0x0024, 0x7F),
            Access::Write(0x0024, 0x80),
        ]
    );

    // nothing else changed
    assert_eq!(bus.memory[0x0020], 0x99);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert_eq!(cpu.registers.a, 0x55);
    assert_eq!(cpu.registers.x, 0x04);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn inc_zeropagex_wraps_from_ff_to_zero() {
    let (mut cpu, mut bus) = setup(&[INC_ZEROPAGEX, 0x20]);
    bus.memory[0x0024] = 0xFF;
    cpu.registers.x = 0x04;
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(bus.memory[0x0024], 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

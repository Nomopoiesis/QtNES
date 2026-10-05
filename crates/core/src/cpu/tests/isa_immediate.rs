use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const ORA_IMMEDIATE: u8 = 0x09;
const AND_IMMEDIATE: u8 = 0x29;
const EOR_IMMEDIATE: u8 = 0x49;
const ADC_IMMEDIATE: u8 = 0x69;

#[test]
fn ora_immediate() {
    let (mut cpu, mut bus) = setup(&[ORA_IMMEDIATE, 0xF0]);
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xFF);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ORA_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn ora_immediate_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[ORA_IMMEDIATE, 0x00]);
    cpu.registers.a = 0x00;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn and_immediate() {
    let (mut cpu, mut bus) = setup(&[AND_IMMEDIATE, 0x9F]);
    cpu.registers.a = 0xF0;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x90);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(AND_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn and_immediate_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[AND_IMMEDIATE, 0x0F]);
    cpu.registers.a = 0xF0;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn eor_immediate() {
    let (mut cpu, mut bus) = setup(&[EOR_IMMEDIATE, 0xFF]);
    cpu.registers.a = 0x0F;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0xF0);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(EOR_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn eor_immediate_sets_zero_flag() {
    // a value XOR itself is zero
    let (mut cpu, mut bus) = setup(&[EOR_IMMEDIATE, 0xA5]);
    cpu.registers.a = 0xA5;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn adc_immediate() {
    let (mut cpu, mut bus) = setup(&[ADC_IMMEDIATE, 0x20]);
    cpu.registers.a = 0x10;
    cpu.registers.x = 0x11;
    cpu.registers.y = 0x22;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    // $10 + #$20 + carry
    assert_eq!(cpu.registers.a, 0x31);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(ADC_IMMEDIATE));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x11);
    assert_eq!(cpu.registers.y, 0x22);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn adc_immediate_sets_carry_and_overflow() {
    // -48 + -112 = -160: carry out and signed overflow
    let (mut cpu, mut bus) = setup(&[ADC_IMMEDIATE, 0x90]);
    cpu.registers.a = 0xD0;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x60);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::OVERFLOW));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

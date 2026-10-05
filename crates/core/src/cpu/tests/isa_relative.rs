use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const BPL: u8 = 0x10;
const BMI: u8 = 0x30;
const BVC: u8 = 0x50;

#[test]
fn bpl_not_taken() {
    let (mut cpu, mut bus) = setup(&[BPL, 0x10]);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(BPL));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, status_before);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn bpl_taken_forward() {
    let (mut cpu, mut bus) = setup(&[BPL, 0x10]);
    cpu.status.set(StatusFlags::NEGATIVE, false);
    cpu.status.set(StatusFlags::CARRY, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    // offset is relative to the address after the operand
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2 + 0x10);
    assert_eq!(cycles, table_cycles(BPL) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2), // dummy read of the next op-code
        ]
    );
    assert_eq!(cpu.status.0, status_before);
}

#[test]
fn bpl_taken_backward() {
    // offset $FE is -2: branch back to the BPL itself
    let (mut cpu, mut bus) = setup(&[BPL, 0xFE]);
    cpu.status.set(StatusFlags::NEGATIVE, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START);
    assert_eq!(cycles, table_cycles(BPL) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
        ]
    );
}

#[test]
fn bpl_taken_forward_across_page() {
    // BPL at $02F0: PC after the operand is $02F2, target is $02F2 + $20 = $0312
    let (mut cpu, mut bus) = setup(&[]);
    bus.memory[0x02F0..0x02F2].copy_from_slice(&[BPL, 0x20]);
    cpu.registers.pc = 0x02F0;
    cpu.status.set(StatusFlags::NEGATIVE, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x0312);
    assert_eq!(cycles, table_cycles(BPL) + 2);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(0x02F0),
            Access::Read(0x02F1),
            Access::Read(0x02F2), // dummy read of the next op-code
            Access::Read(0x0212), // dummy read: new low byte, old high byte
        ]
    );
}

#[test]
fn bpl_taken_backward_across_page() {
    // PC after the operand is $0202, offset $FC is -4, target is $01FE
    let (mut cpu, mut bus) = setup(&[BPL, 0xFC]);
    cpu.status.set(StatusFlags::NEGATIVE, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x01FE);
    assert_eq!(cycles, table_cycles(BPL) + 2);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2), // dummy read of the next op-code
            Access::Read(0x02FE),            // dummy read: new low byte, old high byte
        ]
    );
}

#[test]
fn bmi_not_taken() {
    let (mut cpu, mut bus) = setup(&[BMI, 0x10]);
    cpu.status.set(StatusFlags::NEGATIVE, false);
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(BMI));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, status_before);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn bmi_taken_forward() {
    let (mut cpu, mut bus) = setup(&[BMI, 0x10]);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    cpu.status.set(StatusFlags::CARRY, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    // offset is relative to the address after the operand
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2 + 0x10);
    assert_eq!(cycles, table_cycles(BMI) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2), // dummy read of the next op-code
        ]
    );
    assert_eq!(cpu.status.0, status_before);
}

#[test]
fn bmi_taken_backward() {
    // offset $FE is -2: branch back to the BMI itself
    let (mut cpu, mut bus) = setup(&[BMI, 0xFE]);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START);
    assert_eq!(cycles, table_cycles(BMI) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
        ]
    );
}

#[test]
fn bmi_taken_forward_across_page() {
    // BMI at $02F0: PC after the operand is $02F2, target is $02F2 + $20 = $0312
    let (mut cpu, mut bus) = setup(&[]);
    bus.memory[0x02F0..0x02F2].copy_from_slice(&[BMI, 0x20]);
    cpu.registers.pc = 0x02F0;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x0312);
    assert_eq!(cycles, table_cycles(BMI) + 2);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(0x02F0),
            Access::Read(0x02F1),
            Access::Read(0x02F2), // dummy read of the next op-code
            Access::Read(0x0212), // dummy read: new low byte, old high byte
        ]
    );
}

#[test]
fn bmi_taken_backward_across_page() {
    // PC after the operand is $0202, offset $FC is -4, target is $01FE
    let (mut cpu, mut bus) = setup(&[BMI, 0xFC]);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x01FE);
    assert_eq!(cycles, table_cycles(BMI) + 2);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2), // dummy read of the next op-code
            Access::Read(0x02FE),            // dummy read: new low byte, old high byte
        ]
    );
}

#[test]
fn bvc_not_taken() {
    let (mut cpu, mut bus) = setup(&[BVC, 0x10]);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(BVC));
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, status_before);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn bvc_taken_forward() {
    let (mut cpu, mut bus) = setup(&[BVC, 0x10]);
    cpu.status.set(StatusFlags::OVERFLOW, false);
    cpu.status.set(StatusFlags::CARRY, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    // offset is relative to the address after the operand
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2 + 0x10);
    assert_eq!(cycles, table_cycles(BVC) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2), // dummy read of the next op-code
        ]
    );
    assert_eq!(cpu.status.0, status_before);
}

#[test]
fn bvc_taken_backward() {
    // offset $FE is -2: branch back to the BVC itself
    let (mut cpu, mut bus) = setup(&[BVC, 0xFE]);
    cpu.status.set(StatusFlags::OVERFLOW, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START);
    assert_eq!(cycles, table_cycles(BVC) + 1);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
        ]
    );
}

#[test]
fn bvc_taken_forward_across_page() {
    // BVC at $02F0: PC after the operand is $02F2, target is $02F2 + $20 = $0312
    let (mut cpu, mut bus) = setup(&[]);
    bus.memory[0x02F0..0x02F2].copy_from_slice(&[BVC, 0x20]);
    cpu.registers.pc = 0x02F0;
    cpu.status.set(StatusFlags::OVERFLOW, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x0312);
    assert_eq!(cycles, table_cycles(BVC) + 2);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(0x02F0),
            Access::Read(0x02F1),
            Access::Read(0x02F2), // dummy read of the next op-code
            Access::Read(0x0212), // dummy read: new low byte, old high byte
        ]
    );
}

#[test]
fn bvc_taken_backward_across_page() {
    // PC after the operand is $0202, offset $FC is -4, target is $01FE
    let (mut cpu, mut bus) = setup(&[BVC, 0xFC]);
    cpu.status.set(StatusFlags::OVERFLOW, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x01FE);
    assert_eq!(cycles, table_cycles(BVC) + 2);
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2), // dummy read of the next op-code
            Access::Read(0x02FE),            // dummy read: new low byte, old high byte
        ]
    );
}

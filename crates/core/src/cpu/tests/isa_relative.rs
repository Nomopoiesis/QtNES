use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const BPL: u8 = 0x10;
const BMI: u8 = 0x30;
const BVC: u8 = 0x50;
const BVS: u8 = 0x70;
const BCC: u8 = 0x90;
const BCS: u8 = 0xB0;
const BNE: u8 = 0xD0;
const BEQ: u8 = 0xF0;

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

#[test]
fn bvs_not_taken() {
    let (mut cpu, mut bus) = setup(&[BVS, 0x10]);
    cpu.status.set(StatusFlags::OVERFLOW, false);
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(BVS));
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
fn bvs_taken_forward() {
    let (mut cpu, mut bus) = setup(&[BVS, 0x10]);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.status.set(StatusFlags::CARRY, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    // offset is relative to the address after the operand
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2 + 0x10);
    assert_eq!(cycles, table_cycles(BVS) + 1);
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
fn bvs_taken_backward() {
    // offset $FE is -2: branch back to the BVS itself
    let (mut cpu, mut bus) = setup(&[BVS, 0xFE]);
    cpu.status.set(StatusFlags::OVERFLOW, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START);
    assert_eq!(cycles, table_cycles(BVS) + 1);
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
fn bvs_taken_forward_across_page() {
    // BVS at $02F0: PC after the operand is $02F2, target is $02F2 + $20 = $0312
    let (mut cpu, mut bus) = setup(&[]);
    bus.memory[0x02F0..0x02F2].copy_from_slice(&[BVS, 0x20]);
    cpu.registers.pc = 0x02F0;
    cpu.status.set(StatusFlags::OVERFLOW, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x0312);
    assert_eq!(cycles, table_cycles(BVS) + 2);
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
fn bvs_taken_backward_across_page() {
    // PC after the operand is $0202, offset $FC is -4, target is $01FE
    let (mut cpu, mut bus) = setup(&[BVS, 0xFC]);
    cpu.status.set(StatusFlags::OVERFLOW, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x01FE);
    assert_eq!(cycles, table_cycles(BVS) + 2);
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
fn bcc_not_taken() {
    let (mut cpu, mut bus) = setup(&[BCC, 0x10]);
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(BCC));
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
fn bcc_taken_forward() {
    let (mut cpu, mut bus) = setup(&[BCC, 0x10]);
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    // offset is relative to the address after the operand
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2 + 0x10);
    assert_eq!(cycles, table_cycles(BCC) + 1);
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
fn bcc_taken_backward() {
    // offset $FE is -2: branch back to the BCC itself
    let (mut cpu, mut bus) = setup(&[BCC, 0xFE]);
    cpu.status.set(StatusFlags::CARRY, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START);
    assert_eq!(cycles, table_cycles(BCC) + 1);
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
fn bcc_taken_forward_across_page() {
    // BCC at $02F0: PC after the operand is $02F2, target is $02F2 + $20 = $0312
    let (mut cpu, mut bus) = setup(&[]);
    bus.memory[0x02F0..0x02F2].copy_from_slice(&[BCC, 0x20]);
    cpu.registers.pc = 0x02F0;
    cpu.status.set(StatusFlags::CARRY, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x0312);
    assert_eq!(cycles, table_cycles(BCC) + 2);
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
fn bcc_taken_backward_across_page() {
    // PC after the operand is $0202, offset $FC is -4, target is $01FE
    let (mut cpu, mut bus) = setup(&[BCC, 0xFC]);
    cpu.status.set(StatusFlags::CARRY, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x01FE);
    assert_eq!(cycles, table_cycles(BCC) + 2);
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
fn bcs_not_taken() {
    let (mut cpu, mut bus) = setup(&[BCS, 0x10]);
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(BCS));
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
fn bcs_taken_forward() {
    let (mut cpu, mut bus) = setup(&[BCS, 0x10]);
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    // offset is relative to the address after the operand
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2 + 0x10);
    assert_eq!(cycles, table_cycles(BCS) + 1);
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
fn bcs_taken_backward() {
    // offset $FE is -2: branch back to the BCS itself
    let (mut cpu, mut bus) = setup(&[BCS, 0xFE]);
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START);
    assert_eq!(cycles, table_cycles(BCS) + 1);
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
fn bcs_taken_forward_across_page() {
    // BCS at $02F0: PC after the operand is $02F2, target is $02F2 + $20 = $0312
    let (mut cpu, mut bus) = setup(&[]);
    bus.memory[0x02F0..0x02F2].copy_from_slice(&[BCS, 0x20]);
    cpu.registers.pc = 0x02F0;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x0312);
    assert_eq!(cycles, table_cycles(BCS) + 2);
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
fn bcs_taken_backward_across_page() {
    // PC after the operand is $0202, offset $FC is -4, target is $01FE
    let (mut cpu, mut bus) = setup(&[BCS, 0xFC]);
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x01FE);
    assert_eq!(cycles, table_cycles(BCS) + 2);
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
fn bne_not_taken() {
    let (mut cpu, mut bus) = setup(&[BNE, 0x10]);
    cpu.status.set(StatusFlags::ZERO, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(BNE));
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
fn bne_taken_forward() {
    let (mut cpu, mut bus) = setup(&[BNE, 0x10]);
    cpu.status.set(StatusFlags::ZERO, false);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    // offset is relative to the address after the operand
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2 + 0x10);
    assert_eq!(cycles, table_cycles(BNE) + 1);
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
fn bne_taken_backward() {
    // offset $FE is -2: branch back to the BNE itself
    let (mut cpu, mut bus) = setup(&[BNE, 0xFE]);
    cpu.status.set(StatusFlags::ZERO, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START);
    assert_eq!(cycles, table_cycles(BNE) + 1);
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
fn bne_taken_forward_across_page() {
    // BNE at $02F0: PC after the operand is $02F2, target is $02F2 + $20 = $0312
    let (mut cpu, mut bus) = setup(&[]);
    bus.memory[0x02F0..0x02F2].copy_from_slice(&[BNE, 0x20]);
    cpu.registers.pc = 0x02F0;
    cpu.status.set(StatusFlags::ZERO, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x0312);
    assert_eq!(cycles, table_cycles(BNE) + 2);
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
fn bne_taken_backward_across_page() {
    // PC after the operand is $0202, offset $FC is -4, target is $01FE
    let (mut cpu, mut bus) = setup(&[BNE, 0xFC]);
    cpu.status.set(StatusFlags::ZERO, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x01FE);
    assert_eq!(cycles, table_cycles(BNE) + 2);
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
fn beq_not_taken() {
    let (mut cpu, mut bus) = setup(&[BEQ, 0x10]);
    cpu.status.set(StatusFlags::ZERO, false);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cycles, table_cycles(BEQ));
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
fn beq_taken_forward() {
    let (mut cpu, mut bus) = setup(&[BEQ, 0x10]);
    cpu.status.set(StatusFlags::ZERO, true);
    cpu.status.set(StatusFlags::OVERFLOW, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    // offset is relative to the address after the operand
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2 + 0x10);
    assert_eq!(cycles, table_cycles(BEQ) + 1);
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
fn beq_taken_backward() {
    // offset $FE is -2: branch back to the BEQ itself
    let (mut cpu, mut bus) = setup(&[BEQ, 0xFE]);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, PROGRAM_START);
    assert_eq!(cycles, table_cycles(BEQ) + 1);
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
fn beq_taken_forward_across_page() {
    // BEQ at $02F0: PC after the operand is $02F2, target is $02F2 + $20 = $0312
    let (mut cpu, mut bus) = setup(&[]);
    bus.memory[0x02F0..0x02F2].copy_from_slice(&[BEQ, 0x20]);
    cpu.registers.pc = 0x02F0;
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x0312);
    assert_eq!(cycles, table_cycles(BEQ) + 2);
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
fn beq_taken_backward_across_page() {
    // PC after the operand is $0202, offset $FC is -4, target is $01FE
    let (mut cpu, mut bus) = setup(&[BEQ, 0xFC]);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x01FE);
    assert_eq!(cycles, table_cycles(BEQ) + 2);
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

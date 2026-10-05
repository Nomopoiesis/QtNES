use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};
use crate::cpu::StatusFlags;

const BRK: u8 = 0x00;
const PHP: u8 = 0x08;
const ASL_IMPLIED: u8 = 0x0A;
const CLC: u8 = 0x18;
const PLP: u8 = 0x28;
const ROL_IMPLIED: u8 = 0x2A;
const SEC: u8 = 0x38;
const RTI: u8 = 0x40;
const PHA: u8 = 0x48;
const LSR_IMPLIED: u8 = 0x4A;
const CLI: u8 = 0x58;
const RTS: u8 = 0x60;
const JSR: u8 = 0x20;
const PLA: u8 = 0x68;
const ROR_IMPLIED: u8 = 0x6A;
const SEI: u8 = 0x78;

#[test]
fn asl_implied() {
    let (mut cpu, mut bus) = setup(&[ASL_IMPLIED]);
    cpu.registers.a = 0x41;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x82);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(ASL_IMPLIED));
    // second cycle is a dummy read of the next byte
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );
}

#[test]
fn asl_implied_shifts_bit_7_into_carry() {
    let (mut cpu, mut bus) = setup(&[ASL_IMPLIED]);
    cpu.registers.a = 0x80;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn brk() {
    let (mut cpu, mut bus) = setup(&[BRK, 0xEA]);
    bus.memory[0xFFFE] = 0x34;
    bus.memory[0xFFFF] = 0x12;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.registers.sp, 0xFA);
    assert_eq!(cycles, table_cycles(BRK));

    // the return address skips the padding byte after BRK
    let pushed_status = StatusFlags::CARRY | StatusFlags::BREAK_COMMAND | StatusFlags::UNUSED;
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Write(0x01FD, 0x02),
            Access::Write(0x01FC, 0x02),
            Access::Write(0x01FB, pushed_status),
            Access::Read(0xFFFE),
            Access::Read(0xFFFF),
        ]
    );

    // B only exists in the pushed byte, never in the live register
    assert!(!cpu.status.get(StatusFlags::BREAK_COMMAND));
    assert!(cpu.status.get(StatusFlags::INTERRUPT_DISABLE));
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn php() {
    let (mut cpu, mut bus) = setup(&[PHP]);
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.sp, 0xFC);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(PHP));

    // B and the unused bit are set in the pushed byte only
    let pushed_status = status_before | StatusFlags::BREAK_COMMAND | StatusFlags::UNUSED;
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1), // dummy read of the next byte
            Access::Write(0x01FD, pushed_status),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, status_before);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
}

#[test]
fn php_stack_pointer_wraps() {
    let (mut cpu, mut bus) = setup(&[PHP]);
    cpu.registers.sp = 0x00;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.sp, 0xFF);
    assert_eq!(
        bus.memory[0x0100],
        StatusFlags::BREAK_COMMAND | StatusFlags::UNUSED
    );
}

#[test]
fn clc() {
    let (mut cpu, mut bus) = setup(&[CLC]);
    // every flag set, so the test also shows only carry changes
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(CLC));
    // second cycle is a dummy read of the next byte
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(
        cpu.status.0,
        0xFF & !StatusFlags::BREAK_COMMAND & !StatusFlags::CARRY
    );
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn clc_when_carry_already_clear() {
    let (mut cpu, mut bus) = setup(&[CLC]);
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn plp() {
    let (mut cpu, mut bus) = setup(&[PLP]);
    // C, Z, I, D, V and N all set on the stack
    let pulled = StatusFlags::CARRY
        | StatusFlags::ZERO
        | StatusFlags::INTERRUPT_DISABLE
        | StatusFlags::DECIMAL_MODE
        | StatusFlags::OVERFLOW
        | StatusFlags::NEGATIVE;
    bus.memory[0x01FE] = pulled;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.status.0, pulled);
    assert_eq!(cpu.registers.sp, 0xFE);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(PLP));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1), // dummy read of the next byte
            Access::Read(0x01FD),            // dummy read of the stack
            Access::Read(0x01FE),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
}

#[test]
fn plp_clears_flags() {
    let (mut cpu, mut bus) = setup(&[PLP]);
    bus.memory[0x01FE] = 0x00;
    cpu.status.0 = StatusFlags::CARRY
        | StatusFlags::ZERO
        | StatusFlags::INTERRUPT_DISABLE
        | StatusFlags::DECIMAL_MODE
        | StatusFlags::OVERFLOW
        | StatusFlags::NEGATIVE;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.status.0, 0x00);
}

#[test]
fn plp_ignores_break_and_unused_bits() {
    let (mut cpu, mut bus) = setup(&[PLP]);
    // pulled byte has B and unused set, the live register has neither
    bus.memory[0x01FE] = StatusFlags::BREAK_COMMAND | StatusFlags::UNUSED | StatusFlags::CARRY;
    cpu.status.0 = 0x00;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.status.0, StatusFlags::CARRY);

    // and the other way round: live register keeps its unused bit
    let (mut cpu, mut bus) = setup(&[PLP]);
    bus.memory[0x01FE] = StatusFlags::CARRY;
    cpu.status.0 = StatusFlags::UNUSED;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.status.0, StatusFlags::UNUSED | StatusFlags::CARRY);
}

#[test]
fn plp_restores_what_php_pushed() {
    let (mut cpu, mut bus) = setup(&[PHP, PLP]);
    cpu.status.0 = StatusFlags::CARRY | StatusFlags::NEGATIVE | StatusFlags::UNUSED;
    let status_before = cpu.status.0;

    run_instruction(&mut cpu, &mut bus);
    cpu.status.0 = StatusFlags::UNUSED;
    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.status.0, status_before);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn plp_stack_pointer_wraps() {
    let (mut cpu, mut bus) = setup(&[PLP]);
    cpu.registers.sp = 0xFF;
    bus.memory[0x0100] = StatusFlags::CARRY;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.sp, 0x00);
    assert_eq!(cpu.status.0, StatusFlags::CARRY);
    assert_eq!(
        bus.accesses[2..],
        [Access::Read(0x01FF), Access::Read(0x0100)]
    );
}

#[test]
fn rol_implied() {
    let (mut cpu, mut bus) = setup(&[ROL_IMPLIED]);
    cpu.registers.a = 0x41;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, false);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x82);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(ROL_IMPLIED));
    // second cycle is a dummy read of the next byte
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn rol_implied_rotates_carry_into_bit_0() {
    let (mut cpu, mut bus) = setup(&[ROL_IMPLIED]);
    cpu.registers.a = 0x01;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x03);
    assert!(!cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn rol_implied_rotates_bit_7_into_carry() {
    let (mut cpu, mut bus) = setup(&[ROL_IMPLIED]);
    cpu.registers.a = 0x80;
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn sec() {
    let (mut cpu, mut bus) = setup(&[SEC]);
    // every flag clear, so the test also shows only carry changes
    cpu.status.0 = 0x00;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::CARRY));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(SEC));
    // second cycle is a dummy read of the next byte
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, StatusFlags::CARRY);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sec_when_carry_already_set() {
    let (mut cpu, mut bus) = setup(&[SEC]);
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;

    run_instruction(&mut cpu, &mut bus);

    // carry stays set and every other flag is untouched
    assert_eq!(cpu.status.0, 0xFF & !StatusFlags::BREAK_COMMAND);
}

#[test]
fn rti() {
    let (mut cpu, mut bus) = setup(&[RTI]);
    // stack as an interrupt leaves it: status, then the return address
    cpu.registers.sp = 0xFA;
    let pulled = StatusFlags::CARRY | StatusFlags::NEGATIVE;
    bus.memory[0x01FB] = pulled;
    bus.memory[0x01FC] = 0x34; // PCL
    bus.memory[0x01FD] = 0x12; // PCH
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;

    let cycles = run_instruction(&mut cpu, &mut bus);

    // unlike RTS, the pulled address is used as is
    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.status.0, pulled);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert_eq!(cycles, table_cycles(RTI));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1), // dummy read of the next byte
            Access::Read(0x01FA),            // dummy read of the stack
            Access::Read(0x01FB),            // status
            Access::Read(0x01FC),            // PCL
            Access::Read(0x01FD),            // PCH
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
}

#[test]
fn rti_ignores_break_and_unused_bits() {
    let (mut cpu, mut bus) = setup(&[RTI]);
    cpu.registers.sp = 0xFA;
    // BRK pushes the status with B and unused set
    bus.memory[0x01FB] = StatusFlags::BREAK_COMMAND | StatusFlags::UNUSED | StatusFlags::CARRY;
    bus.memory[0x01FC] = 0x34;
    bus.memory[0x01FD] = 0x12;
    cpu.status.0 = 0x00;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.status.0, StatusFlags::CARRY);
}

#[test]
fn rti_returns_from_brk() {
    let (mut cpu, mut bus) = setup(&[BRK, 0xEA]);
    // BRK vector points at an RTI
    bus.memory[0xFFFE] = 0x00;
    bus.memory[0xFFFF] = 0x03;
    bus.memory[0x0300] = RTI;
    cpu.status.0 = StatusFlags::CARRY | StatusFlags::ZERO;
    let status_before = cpu.status.0;

    run_instruction(&mut cpu, &mut bus);
    assert_eq!(cpu.registers.pc, 0x0300);
    run_instruction(&mut cpu, &mut bus);

    // back after BRK's padding byte, with the status and stack restored
    assert_eq!(cpu.registers.pc, PROGRAM_START + 2);
    assert_eq!(cpu.status.0, status_before);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn rti_stack_pointer_wraps() {
    let (mut cpu, mut bus) = setup(&[RTI]);
    cpu.registers.sp = 0xFE;
    bus.memory[0x01FF] = StatusFlags::CARRY;
    bus.memory[0x0100] = 0x34;
    bus.memory[0x0101] = 0x12;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.registers.sp, 0x01);
    assert_eq!(cpu.status.0, StatusFlags::CARRY);
    assert_eq!(
        bus.accesses[2..],
        [
            Access::Read(0x01FE),
            Access::Read(0x01FF),
            Access::Read(0x0100),
            Access::Read(0x0101),
        ]
    );
}

#[test]
fn pha() {
    let (mut cpu, mut bus) = setup(&[PHA]);
    cpu.registers.a = 0x42;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.sp, 0xFC);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(PHA));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1), // dummy read of the next byte
            Access::Write(0x01FD, 0x42),
        ]
    );

    // nothing else changed, flags included
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.status.0, status_before);
}

#[test]
fn pha_stack_pointer_wraps() {
    let (mut cpu, mut bus) = setup(&[PHA]);
    cpu.registers.a = 0x42;
    cpu.registers.sp = 0x00;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.sp, 0xFF);
    assert_eq!(bus.memory[0x0100], 0x42);
}

#[test]
fn lsr_implied() {
    let (mut cpu, mut bus) = setup(&[LSR_IMPLIED]);
    cpu.registers.a = 0x82;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x41);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(LSR_IMPLIED));
    // second cycle is a dummy read of the next byte
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn lsr_implied_shifts_bit_0_into_carry() {
    let (mut cpu, mut bus) = setup(&[LSR_IMPLIED]);
    cpu.registers.a = 0x01;
    cpu.status.set(StatusFlags::CARRY, false);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn lsr_implied_shifts_zero_into_bit_7() {
    let (mut cpu, mut bus) = setup(&[LSR_IMPLIED]);
    // carry set beforehand must not be shifted in, unlike ROR
    cpu.registers.a = 0xFF;
    cpu.status.set(StatusFlags::CARRY, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x7F);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn cli() {
    let (mut cpu, mut bus) = setup(&[CLI]);
    // every flag set, so the test also shows only interrupt disable changes
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert!(!cpu.status.get(StatusFlags::INTERRUPT_DISABLE));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(CLI));
    // second cycle is a dummy read of the next byte
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(
        cpu.status.0,
        0xFF & !StatusFlags::BREAK_COMMAND & !StatusFlags::INTERRUPT_DISABLE
    );
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn cli_when_interrupt_disable_already_clear() {
    let (mut cpu, mut bus) = setup(&[CLI]);
    cpu.status.0 = 0x00;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.status.0, 0x00);
}

#[test]
fn rts() {
    let (mut cpu, mut bus) = setup(&[RTS]);
    // stack as JSR leaves it: the address of the JSR's last byte, $1233
    cpu.registers.sp = 0xFB;
    bus.memory[0x01FC] = 0x33; // PCL
    bus.memory[0x01FD] = 0x12; // PCH
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::NEGATIVE, true);
    let status_before = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    // the pulled address plus one
    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert_eq!(cycles, table_cycles(RTS));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1), // dummy read of the next byte
            Access::Read(0x01FB),            // dummy read of the stack
            Access::Read(0x01FC),            // PCL
            Access::Read(0x01FD),            // PCH
            Access::Read(0x1233),            // dummy read at the pulled address
        ]
    );

    // nothing else changed, flags included
    assert_eq!(cpu.status.0, status_before);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
}

#[test]
fn rts_returns_from_jsr() {
    // JSR $0300 at $0200, an RTS at $0300
    let (mut cpu, mut bus) = setup(&[JSR, 0x00, 0x03]);
    bus.memory[0x0300] = RTS;

    run_instruction(&mut cpu, &mut bus);
    assert_eq!(cpu.registers.pc, 0x0300);
    run_instruction(&mut cpu, &mut bus);

    // back at the instruction after the 3-byte JSR, with the stack restored
    assert_eq!(cpu.registers.pc, PROGRAM_START + 3);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn rts_pulled_address_increment_crosses_page() {
    let (mut cpu, mut bus) = setup(&[RTS]);
    // pulled $12FF, return address is $1300
    cpu.registers.sp = 0xFB;
    bus.memory[0x01FC] = 0xFF;
    bus.memory[0x01FD] = 0x12;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x1300);
}

#[test]
fn rts_stack_pointer_wraps() {
    let (mut cpu, mut bus) = setup(&[RTS]);
    cpu.registers.sp = 0xFF;
    bus.memory[0x0100] = 0x33;
    bus.memory[0x0101] = 0x12;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x1234);
    assert_eq!(cpu.registers.sp, 0x01);
    assert_eq!(
        bus.accesses[2..5],
        [
            Access::Read(0x01FF),
            Access::Read(0x0100),
            Access::Read(0x0101),
        ]
    );
}

#[test]
fn pla() {
    let (mut cpu, mut bus) = setup(&[PLA]);
    cpu.registers.sp = 0xFC;
    bus.memory[0x01FD] = 0x82;
    cpu.registers.a = 0x00;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);
    cpu.status.set(StatusFlags::ZERO, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x82);
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.sp, 0xFD);
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(PLA));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1), // dummy read of the next byte
            Access::Read(0x01FC),            // dummy read of the stack
            Access::Read(0x01FD),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert!(cpu.status.get(StatusFlags::CARRY));
}

#[test]
fn pla_sets_zero_flag() {
    let (mut cpu, mut bus) = setup(&[PLA]);
    cpu.registers.sp = 0xFC;
    bus.memory[0x01FD] = 0x00;
    cpu.registers.a = 0x42;
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn pla_restores_what_pha_pushed() {
    let (mut cpu, mut bus) = setup(&[PHA, PLA]);
    cpu.registers.a = 0x42;

    run_instruction(&mut cpu, &mut bus);
    cpu.registers.a = 0x00;
    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn pla_stack_pointer_wraps() {
    let (mut cpu, mut bus) = setup(&[PLA]);
    cpu.registers.sp = 0xFF;
    bus.memory[0x0100] = 0x42;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.sp, 0x00);
    assert_eq!(
        bus.accesses[2..],
        [Access::Read(0x01FF), Access::Read(0x0100)]
    );
}

#[test]
fn ror_implied() {
    let (mut cpu, mut bus) = setup(&[ROR_IMPLIED]);
    // carry in goes to bit 7, bit 0 goes out to carry
    cpu.registers.a = 0x02;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    cpu.status.set(StatusFlags::CARRY, true);

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x81);
    assert!(!cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::ZERO));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(ROR_IMPLIED));
    // second cycle is a dummy read of the next byte
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn ror_implied_shifts_bit_0_into_carry() {
    let (mut cpu, mut bus) = setup(&[ROR_IMPLIED]);
    cpu.registers.a = 0x01;
    cpu.status.set(StatusFlags::CARRY, false);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::ZERO));
    assert!(!cpu.status.get(StatusFlags::NEGATIVE));
}

#[test]
fn sei() {
    let (mut cpu, mut bus) = setup(&[SEI]);
    // every flag clear, so the test also shows only interrupt disable changes
    cpu.status.0 = 0x00;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert!(cpu.status.get(StatusFlags::INTERRUPT_DISABLE));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);
    assert_eq!(cycles, table_cycles(SEI));
    // second cycle is a dummy read of the next byte
    assert_eq!(
        bus.accesses,
        [Access::Read(PROGRAM_START), Access::Read(PROGRAM_START + 1)]
    );

    // nothing else changed
    assert_eq!(cpu.status.0, StatusFlags::INTERRUPT_DISABLE);
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
}

#[test]
fn sei_when_interrupt_disable_already_set() {
    let (mut cpu, mut bus) = setup(&[SEI]);
    cpu.status.0 = 0xFF & !StatusFlags::BREAK_COMMAND;

    run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.status.0, 0xFF & !StatusFlags::BREAK_COMMAND);
}

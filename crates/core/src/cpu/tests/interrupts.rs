use super::{Access, PROGRAM_START, run_instruction, run_interrupt, setup};
use crate::cpu::{CPUState, StatusFlags};

const CLC: u8 = 0x18;

/// Puts the CPU straight into the NMI sequence, as if the instruction before
/// `PROGRAM_START` had just finished with an NMI pending.
fn setup_nmi() -> (crate::cpu::CPU, super::TestBus) {
    let (mut cpu, mut bus) = setup(&[]);
    cpu.state = CPUState::NMI;
    cpu.nmi_pending = true;
    // NMI vector points at $8000
    bus.memory[0xFFFA] = 0x00;
    bus.memory[0xFFFB] = 0x80;
    (cpu, bus)
}

#[test]
fn nmi() {
    let (mut cpu, mut bus) = setup_nmi();
    cpu.status.0 = StatusFlags::CARRY | StatusFlags::NEGATIVE;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;

    let cycles = run_interrupt(&mut cpu, &mut bus);

    assert!(matches!(cpu.state, CPUState::Run));
    assert_eq!(cpu.registers.pc, 0x8000);
    assert_eq!(cpu.registers.sp, 0xFA);
    assert!(cpu.status.get(StatusFlags::INTERRUPT_DISABLE));
    assert!(!cpu.nmi_pending);
    assert_eq!(cycles, 7);
    // two dummy reads at PC (which is not incremented), three pushes, then
    // the vector; the pushed status has B clear and the unused bit set
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START),
            Access::Write(0x01FD, 0x02),
            Access::Write(0x01FC, 0x00),
            Access::Write(
                0x01FB,
                StatusFlags::CARRY | StatusFlags::NEGATIVE | StatusFlags::UNUSED
            ),
            Access::Read(0xFFFA),
            Access::Read(0xFFFB),
        ]
    );

    // nothing else changed
    assert!(cpu.status.get(StatusFlags::CARRY));
    assert!(cpu.status.get(StatusFlags::NEGATIVE));
    assert!(!cpu.status.get(StatusFlags::BREAK_COMMAND));
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
}

#[test]
fn nmi_is_taken_after_the_current_instruction() {
    let (mut cpu, mut bus) = setup(&[CLC]);
    bus.memory[0xFFFA] = 0x00;
    bus.memory[0xFFFB] = 0x80;
    cpu.nmi_pending = true;

    run_instruction(&mut cpu, &mut bus);

    // CLC completes, then the CPU enters the NMI sequence instead of fetching
    assert!(matches!(cpu.state, CPUState::NMI));
    assert_eq!(cpu.registers.pc, PROGRAM_START + 1);

    run_interrupt(&mut cpu, &mut bus);

    // the pushed return address is the instruction after CLC
    assert_eq!(bus.memory[0x01FD], 0x02);
    assert_eq!(bus.memory[0x01FC], 0x01);
    assert_eq!(cpu.registers.pc, 0x8000);
}

#[test]
fn nmi_ignores_interrupt_disable() {
    let (mut cpu, mut bus) = setup(&[CLC]);
    bus.memory[0xFFFA] = 0x00;
    bus.memory[0xFFFB] = 0x80;
    cpu.status.set(StatusFlags::INTERRUPT_DISABLE, true);
    cpu.nmi_pending = true;

    run_instruction(&mut cpu, &mut bus);

    assert!(matches!(cpu.state, CPUState::NMI));
}

#[test]
fn nmi_takes_priority_over_irq() {
    let (mut cpu, mut bus) = setup(&[CLC]);
    cpu.nmi_pending = true;
    cpu.irq_pending = true;

    run_instruction(&mut cpu, &mut bus);

    assert!(matches!(cpu.state, CPUState::NMI));
}

#[test]
fn nmi_stack_pointer_wraps() {
    let (mut cpu, mut bus) = setup_nmi();
    cpu.registers.sp = 0x01;
    cpu.status.0 = 0x00;

    run_interrupt(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.sp, 0xFE);
    assert_eq!(bus.memory[0x0101], 0x02);
    assert_eq!(bus.memory[0x0100], 0x00);
    assert_eq!(bus.memory[0x01FF], StatusFlags::UNUSED);
}

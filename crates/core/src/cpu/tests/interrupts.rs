use super::{Access, PROGRAM_START, run_instruction, run_interrupt, setup};
use crate::cpu::{CPUState, StatusFlags};

const BPL: u8 = 0x10;
const CLC: u8 = 0x18;
const JMP_ABSOLUTE: u8 = 0x4C;

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

/// Steps the CPU `cycles` times without any checks, to stop part-way through
/// an instruction.
fn step_cycles(cpu: &mut crate::cpu::CPU, bus: &mut super::TestBus, cycles: u8) {
    for _ in 0..cycles {
        cpu.step(bus);
    }
}

#[test]
fn nmi_during_next_to_last_cycle_is_taken_after_the_instruction() {
    // reference case for the branch tests below: a normal 3-cycle instruction
    // polls at the end of cycle 1, so an NMI arriving then is taken right after
    let (mut cpu, mut bus) = setup(&[JMP_ABSOLUTE, 0x00, 0x03]);

    step_cycles(&mut cpu, &mut bus, 2); // op-code fetch, low byte
    cpu.nmi_pending = true;
    step_cycles(&mut cpu, &mut bus, 1); // high byte, instruction ends

    assert_eq!(cpu.instruction_cycle, 0);
    assert!(matches!(cpu.state, CPUState::NMI));
}

#[test]
fn branch_not_taken_polls_before_operand_fetch() {
    // a not-taken branch is 2 cycles and polls at the end of cycle 0
    let (mut cpu, mut bus) = setup(&[BPL, 0x10]);
    cpu.status.set(StatusFlags::NEGATIVE, true);

    step_cycles(&mut cpu, &mut bus, 1); // op-code fetch
    cpu.nmi_pending = true;
    step_cycles(&mut cpu, &mut bus, 1); // operand fetch, branch not taken

    assert_eq!(cpu.instruction_cycle, 0);
    assert!(matches!(cpu.state, CPUState::NMI));
}

#[test]
fn branch_taken_same_page_delays_nmi_by_one_instruction() {
    // a taken branch without a page cross does not poll at the end of cycle 1
    // BPL at $0200 jumps to $0212, where a CLC waits
    let (mut cpu, mut bus) = setup(&[BPL, 0x10]);
    bus.memory[0x0212] = CLC;
    cpu.status.set(StatusFlags::NEGATIVE, false);

    step_cycles(&mut cpu, &mut bus, 2); // op-code fetch, operand fetch
    cpu.nmi_pending = true;
    step_cycles(&mut cpu, &mut bus, 1); // add offset, branch ends

    // the NMI is not taken after the branch...
    assert_eq!(cpu.instruction_cycle, 0);
    assert_eq!(cpu.registers.pc, 0x0212);
    assert!(matches!(cpu.state, CPUState::Run));

    // ...but stays latched and is taken after the next instruction
    run_instruction(&mut cpu, &mut bus);
    assert_eq!(cpu.registers.pc, 0x0213);
    assert!(matches!(cpu.state, CPUState::NMI));
}

#[test]
fn branch_taken_same_page_misses_irq_released_before_next_poll() {
    // IRQ is a level: if the line drops again before the next poll it is lost
    let (mut cpu, mut bus) = setup(&[BPL, 0x10]);
    bus.memory[0x0212] = CLC;
    cpu.status.set(StatusFlags::NEGATIVE, false);
    cpu.status.set(StatusFlags::INTERRUPT_DISABLE, false);

    step_cycles(&mut cpu, &mut bus, 2); // op-code fetch, operand fetch
    cpu.irq_pending = true;
    step_cycles(&mut cpu, &mut bus, 1); // add offset, branch ends
    cpu.irq_pending = false;

    assert!(matches!(cpu.state, CPUState::Run));
    run_instruction(&mut cpu, &mut bus);
    assert!(matches!(cpu.state, CPUState::Run));
}

#[test]
fn branch_taken_across_page_polls_before_fixup_cycle() {
    // a taken branch that crosses a page polls at the end of cycle 2, as usual
    // BPL at $02F0: target $02F2 + $20 = $0312
    let (mut cpu, mut bus) = setup(&[]);
    bus.memory[0x02F0..0x02F2].copy_from_slice(&[BPL, 0x20]);
    cpu.registers.pc = 0x02F0;
    cpu.status.set(StatusFlags::NEGATIVE, false);

    step_cycles(&mut cpu, &mut bus, 3); // op-code fetch, operand fetch, add offset
    cpu.nmi_pending = true;
    step_cycles(&mut cpu, &mut bus, 1); // fix high byte, branch ends

    assert_eq!(cpu.instruction_cycle, 0);
    assert_eq!(cpu.registers.pc, 0x0312);
    assert!(matches!(cpu.state, CPUState::NMI));
}

#[test]
fn branch_taken_across_page_also_misses_nmi_during_cycle_1() {
    // the poll at the end of cycle 1 is skipped whether or not the page
    // crosses, but here the poll before the fix-up cycle still catches it
    let (mut cpu, mut bus) = setup(&[]);
    bus.memory[0x02F0..0x02F2].copy_from_slice(&[BPL, 0x20]);
    cpu.registers.pc = 0x02F0;
    cpu.status.set(StatusFlags::NEGATIVE, false);

    step_cycles(&mut cpu, &mut bus, 2); // op-code fetch, operand fetch
    cpu.nmi_pending = true;
    step_cycles(&mut cpu, &mut bus, 2); // add offset, fix high byte

    assert_eq!(cpu.instruction_cycle, 0);
    assert!(matches!(cpu.state, CPUState::NMI));
}

use super::{Access, PROGRAM_START, run_instruction, setup, table_cycles};

const JMP_INDIRECT: u8 = 0x6C;

#[test]
fn jmp_indirect() {
    // pointer $1234 holds the target $5678
    let (mut cpu, mut bus) = setup(&[JMP_INDIRECT, 0x34, 0x12]);
    bus.memory[0x1234] = 0x78;
    bus.memory[0x1235] = 0x56;
    cpu.registers.a = 0x11;
    cpu.registers.x = 0x22;
    cpu.registers.y = 0x33;
    let status = cpu.status.0;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x5678);
    assert_eq!(cycles, table_cycles(JMP_INDIRECT));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x1234),
            Access::Read(0x1235),
        ]
    );

    // nothing else changed
    assert_eq!(cpu.registers.a, 0x11);
    assert_eq!(cpu.registers.x, 0x22);
    assert_eq!(cpu.registers.y, 0x33);
    assert_eq!(cpu.registers.sp, 0xFD);
    assert_eq!(cpu.status.0, status);
}

#[test]
fn jmp_indirect_pointer_does_not_cross_page() {
    // hardware bug: the pointer's low byte wraps without carrying into the
    // high byte, so the target's high byte comes from $1200, not $1300
    let (mut cpu, mut bus) = setup(&[JMP_INDIRECT, 0xFF, 0x12]);
    bus.memory[0x12FF] = 0x78;
    bus.memory[0x1200] = 0x56;
    bus.memory[0x1300] = 0x99;

    let cycles = run_instruction(&mut cpu, &mut bus);

    assert_eq!(cpu.registers.pc, 0x5678);
    assert_eq!(cycles, table_cycles(JMP_INDIRECT));
    assert_eq!(
        bus.accesses,
        [
            Access::Read(PROGRAM_START),
            Access::Read(PROGRAM_START + 1),
            Access::Read(PROGRAM_START + 2),
            Access::Read(0x12FF),
            Access::Read(0x1200),
        ]
    );
}

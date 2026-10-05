//! CPU tests: instruction tests, one file per addressing mode, and interrupt
//! sequence tests.
//!
//! These are a child module of `cpu` so they can set up and inspect private
//! CPU state directly.

mod interrupts;
mod isa_absolute;
mod isa_absolutex;
mod isa_absolutey;
mod isa_immediate;
mod isa_implied;
mod isa_indirect;
mod isa_indirecty;
mod isa_relative;
mod isa_xindirect;
mod isa_zeropage;
mod isa_zeropagex;

use super::isa::INFO;
use super::{CPU, CPUState};
use crate::bus::Bus;

/// Where test programs are loaded: clear of the zero page and the stack.
const PROGRAM_START: u16 = 0x0200;

/// One bus access, as seen from the CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Access {
    Read(u16),
    Write(u16, u8),
}

/// Flat 64 KB of RAM that records every access made through the bus.
///
/// Tests read and write `memory` directly to prepare and check state; only
/// accesses made by the CPU end up in `accesses`.
struct TestBus {
    memory:   Vec<u8>,
    accesses: Vec<Access>,
}

impl TestBus {
    fn new() -> Self {
        Self {
            memory:   vec![0; 0x10000],
            accesses: Vec::new(),
        }
    }
}

impl Bus for TestBus {
    fn read(&mut self, address: u16) -> u8 {
        self.accesses.push(Access::Read(address));
        self.memory[usize::from(address)]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.accesses.push(Access::Write(address, value));
        self.memory[usize::from(address)] = value;
    }
}

/// Returns a CPU ready to fetch its first op-code from `PROGRAM_START`, and a
/// bus with `program` loaded there.
fn setup(program: &[u8]) -> (CPU, TestBus) {
    let mut cpu = CPU::new();
    cpu.state = CPUState::Run;
    cpu.registers.pc = PROGRAM_START;
    cpu.registers.sp = 0xFD;

    let mut bus = TestBus::new();
    let start = usize::from(PROGRAM_START);
    bus.memory[start..start + program.len()].copy_from_slice(program);

    (cpu, bus)
}

/// No 6502 instruction takes more cycles than this.
const MAX_INSTRUCTION_CYCLES: u8 = 8;

/// Steps the CPU through one complete instruction and returns the number of
/// cycles it took.
///
/// Every cycle is also checked to make exactly one bus access, and that access
/// must be a write exactly when `CPU::is_write_cycle` predicted one before the
/// cycle ran. This verifies the instruction table's write-cycle lists for
/// every instruction that has a test.
///
/// Panics if the instruction has not finished after `MAX_INSTRUCTION_CYCLES`,
/// so an instruction that never ends fails the test instead of hanging it.
fn run_instruction(cpu: &mut CPU, bus: &mut TestBus) -> u8 {
    for cycles in 1..=MAX_INSTRUCTION_CYCLES {
        let cycle = cpu.instruction_cycle;
        let predicted_write = cpu.is_write_cycle();
        let accesses_before = bus.accesses.len();

        cpu.step(bus);

        let accesses = &bus.accesses[accesses_before..];
        assert_eq!(
            accesses.len(),
            1,
            "op-code {:02X} cycle {cycle}: expected exactly one bus access, got {accesses:?}",
            cpu.op_code
        );
        let was_write = matches!(accesses[0], Access::Write(..));
        assert_eq!(
            was_write,
            predicted_write,
            "op-code {:02X} cycle {cycle}: is_write_cycle predicted {}, the cycle did {:?}",
            cpu.op_code,
            if predicted_write { "a write" } else { "a read" },
            accesses[0]
        );

        if cpu.instruction_cycle == 0 {
            return cycles;
        }
    }
    panic!(
        "instruction did not finish within {MAX_INSTRUCTION_CYCLES} cycles, bus accesses: {:?}",
        bus.accesses
    );
}

/// The NMI and IRQ sequences take 7 cycles.
const MAX_INTERRUPT_CYCLES: u8 = 7;

/// Steps the CPU through one complete interrupt sequence (NMI or IRQ) and
/// returns the number of cycles it took.
///
/// Makes the same per-cycle checks as `run_instruction`: exactly one bus access
/// per cycle, and a write exactly when `CPU::is_write_cycle` predicted one.
///
/// Panics if the CPU is not in an interrupt state when called, or has not
/// returned to `Run` after `MAX_INTERRUPT_CYCLES`.
fn run_interrupt(cpu: &mut CPU, bus: &mut TestBus) -> u8 {
    assert!(
        matches!(cpu.state, CPUState::NMI | CPUState::IRQ),
        "run_interrupt called while the CPU is not in an interrupt state"
    );
    for cycles in 1..=MAX_INTERRUPT_CYCLES {
        let cycle = cpu.interrupt_cycle;
        let predicted_write = cpu.is_write_cycle();
        let accesses_before = bus.accesses.len();

        cpu.step(bus);

        let accesses = &bus.accesses[accesses_before..];
        assert_eq!(
            accesses.len(),
            1,
            "interrupt cycle {cycle}: expected exactly one bus access, got {accesses:?}"
        );
        let was_write = matches!(accesses[0], Access::Write(..));
        assert_eq!(
            was_write,
            predicted_write,
            "interrupt cycle {cycle}: is_write_cycle predicted {}, the cycle did {:?}",
            if predicted_write { "a write" } else { "a read" },
            accesses[0]
        );

        if matches!(cpu.state, CPUState::Run) {
            return cycles;
        }
    }
    panic!(
        "interrupt sequence did not finish within {MAX_INTERRUPT_CYCLES} cycles, bus accesses: {:?}",
        bus.accesses
    );
}

/// Cycle count the instruction table declares for `op_code`.
fn table_cycles(op_code: u8) -> u8 {
    INFO[usize::from(op_code)]
        .expect("op-code is missing from the instruction table")
        .cycles
}

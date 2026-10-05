pub mod isa;
#[cfg(test)]
mod tests;

use crate::bus::Bus;

enum CPUState {
    Reset = 0,
    Run,
    NMI,
    IRQ,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct StatusFlags(u8);

struct Registers {
    pc: u16, // Program Counter
    sp: u8,  // Stack Pointer
    a:  u8,  // Accumulator
    x:  u8,  // X Register
    y:  u8,  // Y Register
}

pub struct CPU {
    state:             CPUState,    // CPU state
    registers:         Registers,   // CPU registers
    status:            StatusFlags, // Status flags
    op_code:           u8,          // current op-code
    instruction_cycle: u8,          // current instruction cycle
    interrupt_cycle:   u8,          // current interrupt cycle

    adl_latch: u8, // low address latch
    adh_latch: u8, // high address latch
    op_latch:  u8, // operand/data latch

    page_crossed: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            state:             CPUState::Reset,
            registers:         Registers::new(),
            status:            StatusFlags(0),
            op_code:           0,
            instruction_cycle: 0,
            interrupt_cycle:   0,
            adl_latch:         0,
            adh_latch:         0,
            op_latch:          0,
            page_crossed:      false,
        }
    }

    pub fn reset(&mut self) {
        self.state = CPUState::Reset;
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
        match self.state {
            CPUState::Reset => self.run_reset(bus),
            CPUState::Run => self.run_step(bus),
            _ => todo!(),
        }
    }

    pub fn push_stack(&mut self, bus: &mut impl Bus, value: u8) {
        bus.write(u16::from_le_bytes([self.registers.sp, 0x01]), value);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
    }

    pub fn pop_stack(&mut self, bus: &mut impl Bus) -> u8 {
        self.registers.sp = self.registers.sp.wrapping_add(1);
        bus.read(u16::from_le_bytes([self.registers.sp, 0x01]))
    }

    fn run_reset(&mut self, bus: &mut impl Bus) {
        match self.interrupt_cycle {
            0 => {
                // clear internal state
                self.status = StatusFlags(0);
                self.registers.pc = 0;
                self.instruction_cycle = 0;
                self.op_code = 0;
                self.interrupt_cycle += 1;
            }
            1 => {
                // SP and flags initalization
                self.registers.sp = 0xFD;
                self.status.set(StatusFlags::INTERRUPT_DISABLE, true);
                self.status.set(StatusFlags::UNUSED, true);
                self.interrupt_cycle += 1;
            }
            2 => {
                // fetch low byte of reset vector
                let addr = u16::from_le_bytes([0xFC, 0xFF]);
                self.adl_latch = bus.read(addr);
                self.interrupt_cycle += 1;
            }
            3 => {
                // set high byte of reset vector
                let addr = u16::from_le_bytes([0xFD, 0xFF]);
                self.adh_latch = bus.read(addr);
                self.interrupt_cycle += 1;
            }
            4 => {
                // set PC
                self.registers.pc = u16::from_le_bytes([self.adl_latch, self.adh_latch]);
                self.state = CPUState::Run;
                self.interrupt_cycle = 0;
            }
            _ => unreachable!(),
        }
    }

    fn run_step(&mut self, bus: &mut impl Bus) {
        if self.instruction_cycle == 0 {
            // instruciton cycle zero means new instruction - check for pending interrupts

            // NMI had priority and cannot be disabled, check its status
            if false {
                todo!("implement NMI handling")
            }

            // IRQ can be disbaled, check if it is enabled and its status
            if false {
                todo!("implement IRQ handling")
            }

            // no interrupt, fetch next op-code
            self.op_code = bus.read(self.registers.pc);
            self.increment_pc();
            self.instruction_cycle += 1;
        } else {
            // Execute instruction
            isa::execute(self, bus);
        }
    }

    fn increment_pc(&mut self) {
        self.registers.pc = self.registers.pc.wrapping_add(1)
    }

    fn is_write_cycle(&self) -> bool {
        match self.state {
            CPUState::Run if self.instruction_cycle > 0 => {
                isa::is_write_cycle(self.op_code, self.instruction_cycle)
            }
            CPUState::NMI | CPUState::IRQ => matches!(self.interrupt_cycle, 2..=4),
            _ => false,
        }
    }
}

impl Registers {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0,
            a:  0,
            x:  0,
            y:  0,
        }
    }
}

impl StatusFlags {
    pub const CARRY: u8 = 1 << 0;
    pub const ZERO: u8 = 1 << 1;
    pub const INTERRUPT_DISABLE: u8 = 1 << 2;
    pub const DECIMAL_MODE: u8 = 1 << 3;
    pub const BREAK_COMMAND: u8 = 1 << 4;
    pub const UNUSED: u8 = 1 << 5;
    pub const OVERFLOW: u8 = 1 << 6;
    pub const NEGATIVE: u8 = 1 << 7;

    fn get(&self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }

    fn set(&mut self, flag: u8, on: bool) {
        if on {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }
}

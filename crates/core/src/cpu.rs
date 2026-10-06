pub mod isa;
#[cfg(test)]
mod tests;

use crate::{
    bus::Bus,
    util::{u16_high_byte, u16_low_byte},
};

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

    page_crossed:   bool,
    nmi_pending:    bool,
    irq_pending:    bool,
    interrupt_poll: bool,
    hold_interrupt: bool, // set by an instruction to skip one poll
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
            nmi_pending:       false,
            irq_pending:       false,
            interrupt_poll:    false,
            hold_interrupt:    false,
        }
    }

    pub fn reset(&mut self) {
        self.state = CPUState::Reset;
    }

    pub fn step(&mut self, bus: &mut impl Bus) {
        match self.state {
            CPUState::Reset => self.run_reset(bus),
            CPUState::Run => self.run_step(bus),
            CPUState::NMI => self.run_nmi(bus),
            CPUState::IRQ => self.run_irq(bus),
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
        if !self.hold_interrupt {
            self.interrupt_poll = (self.irq_pending
                && !self.status.get(StatusFlags::INTERRUPT_DISABLE))
                || self.nmi_pending;
        }
        self.hold_interrupt = false;

        if self.instruction_cycle == 0 {
            // instruciton cycle zero means new instruction
            self.op_code = bus.read(self.registers.pc);
            self.increment_pc();
            self.instruction_cycle += 1;
        } else {
            // Execute instruction
            isa::execute(self, bus);

            if self.instruction_cycle == 0 && self.interrupt_poll {
                if self.nmi_pending {
                    // NMI interrupt pending - takes priority
                    self.state = CPUState::NMI;
                } else {
                    // IRQ interrupt pending
                    self.state = CPUState::IRQ;
                }
            }
        }
    }

    fn increment_pc(&mut self) {
        self.registers.pc = self.registers.pc.wrapping_add(1)
    }

    fn run_nmi(&mut self, bus: &mut impl Bus) {
        match self.interrupt_cycle {
            0 => {
                // dummy read - clear NMI pending
                let _ = bus.read(self.registers.pc);
                self.nmi_pending = false;
                self.interrupt_cycle += 1;
            }
            1 => {
                // dummy read
                let _ = bus.read(self.registers.pc);
                self.interrupt_cycle += 1;
            }
            2 => {
                // push PCH to stack
                self.push_stack(bus, u16_high_byte(self.registers.pc));
                self.interrupt_cycle += 1;
            }
            3 => {
                // push PCL to stack
                self.push_stack(bus, u16_low_byte(self.registers.pc));
                self.interrupt_cycle += 1;
            }
            4 => {
                // push status to stack - set unused but NOT break flag
                self.push_stack(bus, self.status.0 | StatusFlags::UNUSED);
                self.interrupt_cycle += 1;
            }
            5 => {
                self.status.set(StatusFlags::INTERRUPT_DISABLE, true);
                // fetch low byte of NMI vector from 0xFFFA
                let low = bus.read(0xFFFA);
                self.registers.pc = u16::from_le_bytes([low, u16_high_byte(self.registers.pc)]);
                self.interrupt_cycle += 1;
            }
            6 => {
                // fetch high byte of NMI vector from 0xFFFB
                let high = bus.read(0xFFFB);
                self.registers.pc = u16::from_le_bytes([u16_low_byte(self.registers.pc), high]);
                self.interrupt_cycle = 0;
                self.state = CPUState::Run;
            }
            _ => unreachable!(),
        }
    }

    fn run_irq(&mut self, bus: &mut impl Bus) {
        match self.interrupt_cycle {
            0 => {
                // dummy read
                let _ = bus.read(self.registers.pc);
                self.interrupt_cycle += 1;
            }
            1 => {
                // dummy read
                let _ = bus.read(self.registers.pc);
                self.interrupt_cycle += 1;
            }
            2 => {
                // push PCH to stack
                self.push_stack(bus, u16_high_byte(self.registers.pc));
                self.interrupt_cycle += 1;
            }
            3 => {
                // push PCL to stack
                self.push_stack(bus, u16_low_byte(self.registers.pc));
                self.interrupt_cycle += 1;
            }
            4 => {
                // push status to stack - set unused but not break flag
                self.push_stack(bus, self.status.0 | StatusFlags::UNUSED);
                self.interrupt_cycle += 1;
            }
            5 => {
                self.status.set(StatusFlags::INTERRUPT_DISABLE, true);
                // fetch low byte of IRQ vector from 0xFFFE
                let low = bus.read(0xFFFE);
                self.registers.pc = u16::from_le_bytes([low, u16_high_byte(self.registers.pc)]);
                self.interrupt_cycle += 1;
            }
            6 => {
                // fetch high byte of IRQ vector from 0xFFFF
                let high = bus.read(0xFFFF);
                self.registers.pc = u16::from_le_bytes([u16_low_byte(self.registers.pc), high]);
                self.state = CPUState::Run;
                self.interrupt_cycle = 0;
            }
            _ => unreachable!(),
        }
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

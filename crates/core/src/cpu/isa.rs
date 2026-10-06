use std::marker::PhantomData;

use crate::bus;
use crate::cpu::StatusFlags;
use crate::util::{u16_high_byte, u16_low_byte};
use crate::{bus::Bus, cpu::CPU};

trait Operation {
    fn execute(cpu: &mut CPU);
}

trait ReadOp: Operation {} // only reads op_latch as its operand
trait ModifyOp: Operation {} // changes op_latch which may need write back
trait ImpliedOp: Operation {} // needs no op_latch operand, works only on registers
trait WriteOp: Operation {} // puts the value to store into op_latch

trait Condition {
    fn evaluate(cpu: &mut CPU) -> bool;
}

struct Bcc;
impl Condition for Bcc {
    #[inline(always)]
    fn evaluate(cpu: &mut CPU) -> bool {
        !cpu.status.get(StatusFlags::CARRY)
    }
}
struct Bcs;
impl Condition for Bcs {
    #[inline(always)]
    fn evaluate(cpu: &mut CPU) -> bool {
        cpu.status.get(StatusFlags::CARRY)
    }
}
struct Beq;
impl Condition for Beq {
    #[inline(always)]
    fn evaluate(cpu: &mut CPU) -> bool {
        cpu.status.get(StatusFlags::ZERO)
    }
}
struct Bmi;
impl Condition for Bmi {
    #[inline(always)]
    fn evaluate(cpu: &mut CPU) -> bool {
        cpu.status.get(StatusFlags::NEGATIVE)
    }
}
struct Bne;
impl Condition for Bne {
    #[inline(always)]
    fn evaluate(cpu: &mut CPU) -> bool {
        !cpu.status.get(StatusFlags::ZERO)
    }
}
struct Bpl;
impl Condition for Bpl {
    #[inline(always)]
    fn evaluate(cpu: &mut CPU) -> bool {
        !cpu.status.get(StatusFlags::NEGATIVE)
    }
}
struct Bvc;
impl Condition for Bvc {
    #[inline(always)]
    fn evaluate(cpu: &mut CPU) -> bool {
        !cpu.status.get(StatusFlags::OVERFLOW)
    }
}
struct Bvs;
impl Condition for Bvs {
    #[inline(always)]
    fn evaluate(cpu: &mut CPU) -> bool {
        cpu.status.get(StatusFlags::OVERFLOW)
    }
}

#[derive(Clone, Copy)]
pub enum Mnemonic {
    BRK,
    ORA,
    ASL,
    PHP,
    PHA,
    BPL,
    CLC,
    JSR,
    AND,
    BIT,
    ROL,
    PLP,
    BMI,
    SEC,
    RTI,
    EOR,
    LSR,
    JMP,
    BVC,
    CLI,
    CLV,
    RTS,
    ADC,
    ROR,
    BVS,
    STA,
    STY,
    STX,
    DEY,
    TXA,
    TYA,
    TXS,
    LDY,
    LDA,
    LDX,
    TAY,
    TAX,
    TSX,
    BCS,
    BCC,
    PLA,
    SEI,
    CPY,
    CMP,
    DEC,
    INY,
    DEX,
    BNE,
    CLD,
    CPX,
    SBC,
    INC,
    INX,
    NOP,
    BEQ,
    SED,
}

#[derive(Clone, Copy)]
pub enum Mode {
    Implied,
    Immediate,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Indirect,
    XIndirect,
    IndirectY,
}

#[derive(Clone, Copy)]
pub struct Info {
    pub mnemonic:     Mnemonic,
    pub mode:         Mode,
    pub cycles:       u8,
    pub write_cycles: u8,
}

macro_rules! end_instruction {
    ($cpu:ident) => {
        $cpu.instruction_cycle = 0;
    };
}

macro_rules! step_instruction {
    ($cpu:ident) => {
        $cpu.instruction_cycle += 1;
    };
}

macro_rules! instructions {
    ($( $opcode:literal => $mnemonic:ident, $mode:ident, $cycles:literal, [$($write:literal),*], $handler:ident; )*) => {
        pub const INFO: [Option<Info>; 256] = {
            let mut table = [None; 256];
            $(
                table[$opcode] = Some(Info {
                    mnemonic: Mnemonic::$mnemonic,
                    mode: Mode::$mode,
                    cycles: $cycles,
                    write_cycles: 0 $( | (1 << $write) )*,
                });
            )*
            table
        };

        pub(super) fn execute(cpu: &mut CPU, bus: &mut impl Bus) {
            match cpu.op_code {
                $( $opcode => $handler(cpu, bus), )*
                _ => unimplemented!("op-code {:02X}", cpu.op_code),
            }
        }
    };
}

#[inline(always)]
fn set_zn_flags(cpu: &mut CPU, value: u8) {
    cpu.status.set(StatusFlags::ZERO, value == 0);
    cpu.status.set(StatusFlags::NEGATIVE, (value & 0x80) != 0);
}

#[inline(always)]
fn operation_xindirect<Op: ReadOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of address
            cpu.op_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // perform dummy read and add X to pointer (zero page wraparound)
            let _ = bus.read(u16::from_le_bytes([cpu.op_latch, 0x00]));
            cpu.op_latch = cpu.op_latch.wrapping_add(cpu.registers.x);
            step_instruction!(cpu);
        }
        3 => {
            // read low byte of the effective address
            cpu.adl_latch = bus.read(u16::from_le_bytes([cpu.op_latch, 0x00]));
            step_instruction!(cpu);
        }
        4 => {
            // read high byte of the effective address
            cpu.adh_latch = bus.read(u16::from_le_bytes([cpu.op_latch.wrapping_add(1), 0x00]));
            step_instruction!(cpu);
        }
        5 => {
            // read value from effective address and execute op
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            Op::execute(cpu);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_xindirect_write<Op: WriteOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of address
            cpu.op_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // perform dummy read and add X to pointer (zero page wraparound)
            let _ = bus.read(u16::from_le_bytes([cpu.op_latch, 0x00]));
            cpu.op_latch = cpu.op_latch.wrapping_add(cpu.registers.x);
            step_instruction!(cpu);
        }
        3 => {
            // read low byte of the effective address
            cpu.adl_latch = bus.read(u16::from_le_bytes([cpu.op_latch, 0x00]));
            step_instruction!(cpu);
        }
        4 => {
            // read high byte of the effective address
            cpu.adh_latch = bus.read(u16::from_le_bytes([cpu.op_latch.wrapping_add(1), 0x00]));
            step_instruction!(cpu);
        }
        5 => {
            // execute op and write its value to the effective address
            Op::execute(cpu);
            bus.write(
                u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]),
                cpu.op_latch,
            );
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_zeropage<Op: ReadOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, 0x00]));
            Op::execute(cpu);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_zeropage_write<Op: WriteOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch zero page address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // execute op and write its value to the zero page address
            Op::execute(cpu);
            bus.write(u16::from_le_bytes([cpu.adl_latch, 0x00]), cpu.op_latch);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_zeropage_rmw<Op: ModifyOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, 0x00]));
            step_instruction!(cpu);
        }
        3 => {
            // dummy write
            bus.write(u16::from_le_bytes([cpu.adl_latch, 0x00]), cpu.op_latch);
            Op::execute(cpu);
            step_instruction!(cpu);
        }
        4 => {
            // actual write
            bus.write(u16::from_le_bytes([cpu.adl_latch, 0x00]), cpu.op_latch);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_implied<Op: ImpliedOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // dummy read
            let _ = bus.read(cpu.registers.pc);
            Op::execute(cpu);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_immediate<Op: ReadOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch immediate value and execute op
            cpu.op_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            Op::execute(cpu);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_absolute<Op: ReadOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        3 => {
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            Op::execute(cpu);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_absolute_write<Op: WriteOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // fetch high byte of address
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        3 => {
            // execute op and write its value to the effective address
            Op::execute(cpu);
            bus.write(
                u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]),
                cpu.op_latch,
            );
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_absolute_rmw<Op: ModifyOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        3 => {
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            step_instruction!(cpu);
        }
        4 => {
            // dummy write
            bus.write(
                u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]),
                cpu.op_latch,
            );
            Op::execute(cpu);
            step_instruction!(cpu);
        }
        5 => {
            // actual write
            bus.write(
                u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]),
                cpu.op_latch,
            );
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_absolutex<Op: ReadOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // fetch high byte of address
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            // add X to low byte to, if needed, trigger page crossing
            let tmp = (cpu.adl_latch as u16) + (cpu.registers.x as u16);
            cpu.adl_latch = u16_low_byte(tmp);
            cpu.page_crossed = u16_high_byte(tmp) != 0;
            step_instruction!(cpu);
        }
        3 => {
            // read value from the effective address, if page was crossed this is a dummy read
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            if cpu.page_crossed {
                // page was crossed, fix high byte and read again on the next cycle
                cpu.adh_latch = cpu.adh_latch.wrapping_add(1);
                step_instruction!(cpu);
            } else {
                // page was not crossed, execute operation and end instruction
                Op::execute(cpu);
                end_instruction!(cpu);
            }
        }
        4 => {
            debug_assert!(
                cpu.page_crossed,
                "Unexpected cycle for Absolute X read (page was not crossed)"
            );
            cpu.page_crossed = false; // reset page cross flag

            // read value from effective address
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            Op::execute(cpu);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_absolutex_write<Op: WriteOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // fetch high byte of address
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            // add X to low byte to, if needed, trigger page crossing
            let tmp = (cpu.adl_latch as u16) + (cpu.registers.x as u16);
            cpu.adl_latch = u16_low_byte(tmp);
            cpu.page_crossed = u16_high_byte(tmp) != 0;
            step_instruction!(cpu);
        }
        3 => {
            // dummy read from the effective address, always taken by a write: the high byte
            // may not be fixed yet, so the write cannot happen on this cycle
            let _ = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            if cpu.page_crossed {
                cpu.adh_latch = cpu.adh_latch.wrapping_add(1);
                cpu.page_crossed = false; // reset page cross flag
            }
            step_instruction!(cpu);
        }
        4 => {
            // execute op and write its value to the effective address
            Op::execute(cpu);
            bus.write(
                u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]),
                cpu.op_latch,
            );
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_absolutex_rmw<Op: ModifyOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // fetch high byte of address
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            // add X to low byte to, if needed, trigger page crossing
            let tmp = (cpu.adl_latch as u16) + (cpu.registers.x as u16);
            cpu.adl_latch = u16_low_byte(tmp);
            cpu.page_crossed = u16_high_byte(tmp) != 0;
            step_instruction!(cpu);
        }
        3 => {
            // dummy read from the effective address with the high byte not yet fixed,
            // read-modify-write always takes this cycle, page crossed or not
            let _ = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            if cpu.page_crossed {
                cpu.adh_latch = cpu.adh_latch.wrapping_add(1);
                cpu.page_crossed = false; // reset page cross flag
            }
            step_instruction!(cpu);
        }
        4 => {
            // read value from effective address
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            step_instruction!(cpu);
        }
        5 => {
            // dummy write
            bus.write(
                u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]),
                cpu.op_latch,
            );
            Op::execute(cpu);
            step_instruction!(cpu);
        }
        6 => {
            // actual write
            bus.write(
                u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]),
                cpu.op_latch,
            );
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_absolutey<Op: ReadOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // fetch high byte of address
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            // add Y to low byte to, if needed, trigger page crossing
            let tmp = (cpu.adl_latch as u16) + (cpu.registers.y as u16);
            cpu.adl_latch = u16_low_byte(tmp);
            cpu.page_crossed = u16_high_byte(tmp) != 0;
            step_instruction!(cpu);
        }
        3 => {
            // read value from the effective address, if page was crossed this is a dummy read
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            if cpu.page_crossed {
                // page was crossed, fix high byte and read again on the next cycle
                cpu.adh_latch = cpu.adh_latch.wrapping_add(1);
                step_instruction!(cpu);
            } else {
                // page was not crossed, execute operation and end instruction
                Op::execute(cpu);
                end_instruction!(cpu);
            }
        }
        4 => {
            debug_assert!(
                cpu.page_crossed,
                "Unexpected cycle for Absolute Y read (page was not crossed)"
            );
            cpu.page_crossed = false; // reset page cross flag

            // read value from effective address
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            Op::execute(cpu);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_absolutey_write<Op: WriteOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // fetch high byte of address
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            // add Y to low byte to, if needed, trigger page crossing
            let tmp = (cpu.adl_latch as u16) + (cpu.registers.y as u16);
            cpu.adl_latch = u16_low_byte(tmp);
            cpu.page_crossed = u16_high_byte(tmp) != 0;
            step_instruction!(cpu);
        }
        3 => {
            // dummy read from the effective address, always taken by a write: the high byte
            // may not be fixed yet, so the write cannot happen on this cycle
            let _ = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            if cpu.page_crossed {
                cpu.adh_latch = cpu.adh_latch.wrapping_add(1);
                cpu.page_crossed = false; // reset page cross flag
            }
            step_instruction!(cpu);
        }
        4 => {
            // execute op and write its value to the effective address
            Op::execute(cpu);
            bus.write(
                u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]),
                cpu.op_latch,
            );
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_indirecty<Op: ReadOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch pointer address
            cpu.op_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // fetch low byte address
            cpu.adl_latch = bus.read(u16::from_le_bytes([cpu.op_latch, 0x00]));
            step_instruction!(cpu);
        }
        3 => {
            // fetch high byte of address
            let high_byte = cpu.op_latch.wrapping_add(1);
            cpu.adh_latch = bus.read(u16::from_le_bytes([high_byte, 0x00]));
            // add Y to low byte to, if needed, trigger page crossing
            let tmp = (cpu.adl_latch as u16) + (cpu.registers.y as u16);
            cpu.adl_latch = u16_low_byte(tmp);
            cpu.page_crossed = u16_high_byte(tmp) != 0;
            step_instruction!(cpu);
        }
        4 => {
            // read value from the effective address into the register, if page was crossed this is a dummy read
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            if cpu.page_crossed {
                // page was crossed, increment high byte and set op_latch to 0
                cpu.adh_latch = cpu.adh_latch.wrapping_add(1);
                cpu.op_latch = 0;
                step_instruction!(cpu);
            } else {
                // page was not crossed, execute logical operation and end instruction
                Op::execute(cpu);
                end_instruction!(cpu);
            }
        }
        5 => {
            debug_assert!(
                cpu.page_crossed,
                "Unexpected cycle for Logical Operation Indirect Y (page was not corossed)"
            );
            cpu.page_crossed = false; // reset page cross flag

            // read value from effective address
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            Op::execute(cpu);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_indirecty_write<Op: WriteOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch pointer address
            cpu.op_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // fetch low byte of address
            cpu.adl_latch = bus.read(u16::from_le_bytes([cpu.op_latch, 0x00]));
            step_instruction!(cpu);
        }
        3 => {
            // fetch high byte of address
            let high_byte = cpu.op_latch.wrapping_add(1);
            cpu.adh_latch = bus.read(u16::from_le_bytes([high_byte, 0x00]));
            // add Y to low byte to, if needed, trigger page crossing
            let tmp = (cpu.adl_latch as u16) + (cpu.registers.y as u16);
            cpu.adl_latch = u16_low_byte(tmp);
            cpu.page_crossed = u16_high_byte(tmp) != 0;
            step_instruction!(cpu);
        }
        4 => {
            // dummy read from the effective address, always taken by a write: the high byte
            // may not be fixed yet, so the write cannot happen on this cycle
            let _ = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            if cpu.page_crossed {
                cpu.adh_latch = cpu.adh_latch.wrapping_add(1);
                cpu.page_crossed = false; // reset page cross flag
            }
            step_instruction!(cpu);
        }
        5 => {
            // execute op and write its value to the effective address
            Op::execute(cpu);
            bus.write(
                u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]),
                cpu.op_latch,
            );
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_zeropagex<Op: ReadOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            let _ = bus.read(u16::from_le_bytes([cpu.adl_latch, 0x00]));
            cpu.adl_latch = cpu.adl_latch.wrapping_add(cpu.registers.x);
            step_instruction!(cpu);
        }
        3 => {
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, 0x00]));
            Op::execute(cpu);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_zeropagex_write<Op: WriteOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch zero page address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // dummy read and add X to address (zero page wraparound)
            let _ = bus.read(u16::from_le_bytes([cpu.adl_latch, 0x00]));
            cpu.adl_latch = cpu.adl_latch.wrapping_add(cpu.registers.x);
            step_instruction!(cpu);
        }
        3 => {
            // execute op and write its value to the effective address
            Op::execute(cpu);
            bus.write(u16::from_le_bytes([cpu.adl_latch, 0x00]), cpu.op_latch);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_zeropagey<Op: ReadOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch zero page address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // dummy read and add Y to address (zero page wraparound)
            let _ = bus.read(u16::from_le_bytes([cpu.adl_latch, 0x00]));
            cpu.adl_latch = cpu.adl_latch.wrapping_add(cpu.registers.y);
            step_instruction!(cpu);
        }
        3 => {
            // read value from the effective address and execute op
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, 0x00]));
            Op::execute(cpu);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_zeropagey_write<Op: WriteOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch zero page address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // dummy read and add Y to address (zero page wraparound)
            let _ = bus.read(u16::from_le_bytes([cpu.adl_latch, 0x00]));
            cpu.adl_latch = cpu.adl_latch.wrapping_add(cpu.registers.y);
            step_instruction!(cpu);
        }
        3 => {
            // execute op and write its value to the effective address
            Op::execute(cpu);
            bus.write(u16::from_le_bytes([cpu.adl_latch, 0x00]), cpu.op_latch);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn operation_zeropagex_rmw<Op: ModifyOp>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            let _ = bus.read(u16::from_le_bytes([cpu.adl_latch, 0x00]));
            cpu.adl_latch = cpu.adl_latch.wrapping_add(cpu.registers.x);
            step_instruction!(cpu);
        }
        3 => {
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, 0x00]));
            step_instruction!(cpu);
        }
        4 => {
            bus.write(u16::from_le_bytes([cpu.adl_latch, 0x00]), cpu.op_latch);
            Op::execute(cpu);
            step_instruction!(cpu);
        }
        5 => {
            bus.write(u16::from_le_bytes([cpu.adl_latch, 0x00]), cpu.op_latch);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
fn relative_branch<Cond: Condition>(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch operand
            cpu.op_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            // evaluate branch condition
            if !Cond::evaluate(cpu) {
                end_instruction!(cpu);
            } else {
                // a taken branch skips the interrupt poll before cycle 2
                cpu.hold_interrupt = true;
                step_instruction!(cpu);
            }
        }
        2 => {
            // dummy read - fetch opcode for enxt instruction
            let _ = bus.read(cpu.registers.pc);
            // add operand ot pcl and check for page crossing
            let offset = cpu.op_latch as i8 as i16 as u16;
            let new_pc = cpu.registers.pc.wrapping_add(offset as u16);
            cpu.page_crossed = u16_high_byte(new_pc) != u16_high_byte(cpu.registers.pc);
            cpu.registers.pc =
                u16::from_le_bytes([u16_low_byte(new_pc), u16_high_byte(cpu.registers.pc)]);
            if cpu.page_crossed {
                cpu.op_latch = u16_high_byte(new_pc);
                step_instruction!(cpu);
            } else {
                end_instruction!(cpu);
            }
        }
        3 => {
            cpu.page_crossed = false; // reset page cross flag
            // dummy read - fetch opcode for enxt instruction
            let _ = bus.read(cpu.registers.pc);
            // fix high-byte of PC
            cpu.registers.pc = u16::from_le_bytes([u16_low_byte(cpu.registers.pc), cpu.op_latch]);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

/// Adapts an ModifyOP operation to operate on the accumulator thus effectivyly changing it to being a implied op.
struct OnA<Op>(PhantomData<Op>);
impl<Op: ModifyOp> Operation for OnA<Op> {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.op_latch = cpu.registers.a;
        Op::execute(cpu);
        cpu.registers.a = cpu.op_latch;
    }
}
impl<Op: ModifyOp> ImpliedOp for OnA<Op> {}

/// Adapts an ModifyOP operation to operate on the x-register thus effectivyly changing it to being a implied op.
struct OnX<Op>(PhantomData<Op>);
impl<Op: ModifyOp> Operation for OnX<Op> {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.op_latch = cpu.registers.x;
        Op::execute(cpu);
        cpu.registers.x = cpu.op_latch;
    }
}
impl<Op: ModifyOp> ImpliedOp for OnX<Op> {}

/// Adapts an ModifyOP operation to operate on the y-register thus effectivyly changing it to being a implied op.
struct OnY<Op>(PhantomData<Op>);
impl<Op: ModifyOp> Operation for OnY<Op> {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.op_latch = cpu.registers.y;
        Op::execute(cpu);
        cpu.registers.y = cpu.op_latch;
    }
}
impl<Op: ModifyOp> ImpliedOp for OnY<Op> {}

struct Asl;
impl Operation for Asl {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.status
            .set(StatusFlags::CARRY, (cpu.op_latch & 0x80) != 0);
        cpu.op_latch <<= 1;
        set_zn_flags(cpu, cpu.op_latch);
    }
}
impl ModifyOp for Asl {}

struct Rol;
impl Operation for Rol {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        // shift left, the old carry goes into bit 0 and bit 7 goes into carry
        let carry_in = cpu.status.get(StatusFlags::CARRY) as u8;
        cpu.status
            .set(StatusFlags::CARRY, (cpu.op_latch & 0x80) != 0);
        cpu.op_latch = (cpu.op_latch << 1) | carry_in;
        set_zn_flags(cpu, cpu.op_latch);
    }
}
impl ModifyOp for Rol {}

struct Lsr;
impl Operation for Lsr {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        // bit 0 goes into carry, bit 7 becomes 0 so N is always cleared
        cpu.status
            .set(StatusFlags::CARRY, (cpu.op_latch & 0x01) != 0);
        cpu.op_latch >>= 1;
        set_zn_flags(cpu, cpu.op_latch);
    }
}
impl ModifyOp for Lsr {}

struct Ror;
impl Operation for Ror {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        // shift right, the old carry goes into bit 7 and bit 0 goes into carry
        let carry_in = cpu.status.get(StatusFlags::CARRY) as u8;
        cpu.status
            .set(StatusFlags::CARRY, (cpu.op_latch & 0x01) != 0);
        cpu.op_latch = (cpu.op_latch >> 1) | (carry_in << 7);
        set_zn_flags(cpu, cpu.op_latch);
    }
}
impl ModifyOp for Ror {}

struct Dec;
impl Operation for Dec {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.op_latch = cpu.op_latch.wrapping_sub(1);
        set_zn_flags(cpu, cpu.op_latch);
    }
}
impl ModifyOp for Dec {}

struct Inc;
impl Operation for Inc {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.op_latch = cpu.op_latch.wrapping_add(1);
        set_zn_flags(cpu, cpu.op_latch);
    }
}
impl ModifyOp for Inc {}

struct Ora;
impl Operation for Ora {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.a |= cpu.op_latch;
        set_zn_flags(cpu, cpu.registers.a);
    }
}
impl ReadOp for Ora {}

struct And;
impl Operation for And {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.a &= cpu.op_latch;
        set_zn_flags(cpu, cpu.registers.a);
    }
}
impl ReadOp for And {}

struct Eor;
impl Operation for Eor {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.a ^= cpu.op_latch;
        set_zn_flags(cpu, cpu.registers.a);
    }
}
impl ReadOp for Eor {}

/// A + operand + C into A, setting C, V, Z and N; shared by ADC and SBC.
#[inline(always)]
fn add_with_carry(cpu: &mut CPU, operand: u8) {
    // the 2A03 has no decimal mode: the D flag is ignored
    let a = cpu.registers.a;
    let sum = a as u16 + operand as u16 + cpu.status.get(StatusFlags::CARRY) as u16;
    let result = u16_low_byte(sum);
    cpu.status.set(StatusFlags::CARRY, sum > 0xFF);
    // signed overflow: both inputs have the same sign and the result has the other one
    cpu.status.set(
        StatusFlags::OVERFLOW,
        (!(a ^ operand) & (a ^ result) & 0x80) != 0,
    );
    cpu.registers.a = result;
    set_zn_flags(cpu, result);
}

struct Adc;
impl Operation for Adc {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        add_with_carry(cpu, cpu.op_latch);
    }
}
impl ReadOp for Adc {}

struct Sbc;
impl Operation for Sbc {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        // A - M - (1 - C) is A + !M + C in two's complement, so SBC is ADC of the
        // inverted operand; C then means "no borrow"
        add_with_carry(cpu, !cpu.op_latch);
    }
}
impl ReadOp for Sbc {}

struct Ldy;
impl Operation for Ldy {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.y = cpu.op_latch;
        set_zn_flags(cpu, cpu.registers.y);
    }
}
impl ReadOp for Ldy {}

struct Lda;
impl Operation for Lda {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.a = cpu.op_latch;
        set_zn_flags(cpu, cpu.registers.a);
    }
}
impl ReadOp for Lda {}

struct Ldx;
impl Operation for Ldx {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.x = cpu.op_latch;
        set_zn_flags(cpu, cpu.registers.x);
    }
}
impl ReadOp for Ldx {}

struct Sta;
impl Operation for Sta {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.op_latch = cpu.registers.a;
    }
}
impl WriteOp for Sta {}

struct Sty;
impl Operation for Sty {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.op_latch = cpu.registers.y;
    }
}
impl WriteOp for Sty {}

struct Stx;
impl Operation for Stx {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.op_latch = cpu.registers.x;
    }
}
impl WriteOp for Stx {}

struct Bit;
impl Operation for Bit {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        // Z from A AND operand; N and V are copied from bits 7 and 6 of the operand.
        // The accumulator is not changed.
        cpu.status
            .set(StatusFlags::ZERO, (cpu.registers.a & cpu.op_latch) == 0);
        cpu.status
            .set(StatusFlags::NEGATIVE, (cpu.op_latch & 0x80) != 0);
        cpu.status
            .set(StatusFlags::OVERFLOW, (cpu.op_latch & 0x40) != 0);
    }
}
impl ReadOp for Bit {}

struct Cpy;
impl Operation for Cpy {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.status
            .set(StatusFlags::CARRY, cpu.registers.y >= cpu.op_latch);
        set_zn_flags(cpu, cpu.registers.y.wrapping_sub(cpu.op_latch));
    }
}
impl ReadOp for Cpy {}

struct Cmp;
impl Operation for Cmp {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.status
            .set(StatusFlags::CARRY, cpu.registers.a >= cpu.op_latch);
        set_zn_flags(cpu, cpu.registers.a.wrapping_sub(cpu.op_latch));
    }
}
impl ReadOp for Cmp {}

struct Cpx;
impl Operation for Cpx {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.status
            .set(StatusFlags::CARRY, cpu.registers.x >= cpu.op_latch);
        set_zn_flags(cpu, cpu.registers.x.wrapping_sub(cpu.op_latch));
    }
}
impl ReadOp for Cpx {}

struct Clc;
impl Operation for Clc {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.status.set(StatusFlags::CARRY, false);
    }
}
impl ImpliedOp for Clc {}

struct Cli;
impl Operation for Cli {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.status.set(StatusFlags::INTERRUPT_DISABLE, false);
    }
}
impl ImpliedOp for Cli {}

struct Clv;
impl Operation for Clv {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.status.set(StatusFlags::OVERFLOW, false);
    }
}
impl ImpliedOp for Clv {}

struct Cld;
impl Operation for Cld {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        // the 2A03 ignores decimal mode, but the flag itself still exists
        cpu.status.set(StatusFlags::DECIMAL_MODE, false);
    }
}
impl ImpliedOp for Cld {}

struct Sed;
impl Operation for Sed {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        // sets the flag only: ADC and SBC ignore it on the 2A03
        cpu.status.set(StatusFlags::DECIMAL_MODE, true);
    }
}
impl ImpliedOp for Sed {}

struct Nop;
impl Operation for Nop {
    #[inline(always)]
    fn execute(_cpu: &mut CPU) {}
}
impl ImpliedOp for Nop {}

struct Sec;
impl Operation for Sec {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.status.set(StatusFlags::CARRY, true);
    }
}
impl ImpliedOp for Sec {}

struct Sei;
impl Operation for Sei {
    fn execute(cpu: &mut CPU) {
        cpu.status.set(StatusFlags::INTERRUPT_DISABLE, true);
    }
}
impl ImpliedOp for Sei {}

struct Txa;
impl Operation for Txa {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.a = cpu.registers.x;
        set_zn_flags(cpu, cpu.registers.a);
    }
}
impl ImpliedOp for Txa {}

struct Tya;
impl Operation for Tya {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.a = cpu.registers.y;
        set_zn_flags(cpu, cpu.registers.a);
    }
}
impl ImpliedOp for Tya {}

struct Txs;
impl Operation for Txs {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        // the only transfer that sets no flags
        cpu.registers.sp = cpu.registers.x;
    }
}
impl ImpliedOp for Txs {}

struct Tay;
impl Operation for Tay {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.y = cpu.registers.a;
        set_zn_flags(cpu, cpu.registers.y);
    }
}
impl ImpliedOp for Tay {}

struct Tax;
impl Operation for Tax {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.x = cpu.registers.a;
        set_zn_flags(cpu, cpu.registers.x);
    }
}
impl ImpliedOp for Tax {}

struct Tsx;
impl Operation for Tsx {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.registers.x = cpu.registers.sp;
        set_zn_flags(cpu, cpu.registers.x);
    }
}
impl ImpliedOp for Tsx {}

fn brk_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // dummy read
            let _ = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // push PC high byte to stack
            cpu.push_stack(bus, u16_high_byte(cpu.registers.pc));
            step_instruction!(cpu);
        }
        3 => {
            // push PC low byte to stack
            cpu.push_stack(bus, u16_low_byte(cpu.registers.pc));
            step_instruction!(cpu);
        }
        4 => {
            // push status to stack - set B flag
            cpu.push_stack(
                bus,
                cpu.status.0 | StatusFlags::BREAK_COMMAND | StatusFlags::UNUSED,
            );
            cpu.status.set(StatusFlags::INTERRUPT_DISABLE, true);
            step_instruction!(cpu);
        }
        5 => {
            // fetch low byte of interrupt vector
            let low = bus.read(u16::from_le_bytes([0xFE, 0xFF]));
            cpu.registers.pc = u16::from_le_bytes([low, u16_high_byte(cpu.registers.pc)]);
            step_instruction!(cpu);
        }
        6 => {
            // fetch high byte of interrupt vector
            let high = bus.read(u16::from_le_bytes([0xFF, 0xFF]));
            cpu.registers.pc = u16::from_le_bytes([u16_low_byte(cpu.registers.pc), high]);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

fn ora_xindirect(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_xindirect::<Ora>(cpu, bus);
}

fn ora_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Ora>(cpu, bus);
}

fn asl_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage_rmw::<Asl>(cpu, bus);
}

fn asl_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<OnA<Asl>>(cpu, bus);
}

fn php_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // dummy read
            let _ = bus.read(cpu.registers.pc);
            step_instruction!(cpu);
        }
        2 => {
            // push cpu status onto stack
            cpu.push_stack(
                bus,
                cpu.status.0 | StatusFlags::BREAK_COMMAND | StatusFlags::UNUSED,
            );
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

fn plp_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // dummy read
            let _ = bus.read(cpu.registers.pc);
            step_instruction!(cpu);
        }
        2 => {
            // dummy read of the stack, pull happens on the next cycle
            let _ = bus.read(u16::from_le_bytes([cpu.registers.sp, 0x01]));
            step_instruction!(cpu);
        }
        3 => {
            // pull status from stack, B and unused bits are not real flags:
            // keep their current values and ignore the pulled ones
            let pulled = cpu.pop_stack(bus);
            let ignored = StatusFlags::BREAK_COMMAND | StatusFlags::UNUSED;
            cpu.status.0 = (pulled & !ignored) | (cpu.status.0 & ignored);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

fn ora_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<Ora>(cpu, bus);
}

fn ora_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Ora>(cpu, bus);
}

fn asl_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute_rmw::<Asl>(cpu, bus);
}

fn bpl_relative(cpu: &mut CPU, bus: &mut impl Bus) {
    relative_branch::<Bpl>(cpu, bus);
}

fn ora_indirecty(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_indirecty::<Ora>(cpu, bus);
}

fn ora_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex::<Ora>(cpu, bus);
}

fn asl_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex_rmw::<Asl>(cpu, bus);
}

fn clc_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Clc>(cpu, bus);
}

fn ora_absolutey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutey::<Ora>(cpu, bus);
}

fn ora_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex::<Ora>(cpu, bus);
}

fn asl_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex_rmw::<Asl>(cpu, bus);
}

fn jsr_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // internal operation
            let _ = bus.read(u16::from_le_bytes([cpu.registers.sp, 0x01]));
            step_instruction!(cpu);
        }
        3 => {
            // push PCH on stack
            cpu.push_stack(bus, u16_high_byte(cpu.registers.pc));
            step_instruction!(cpu);
        }
        4 => {
            // push PCL on stack
            cpu.push_stack(bus, u16_low_byte(cpu.registers.pc));
            step_instruction!(cpu);
        }
        5 => {
            // fetch high byte off address and set pc
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.registers.pc = u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

fn and_xindirect(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_xindirect::<And>(cpu, bus);
}

fn bit_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Bit>(cpu, bus);
}

fn and_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<And>(cpu, bus);
}

fn rol_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage_rmw::<Rol>(cpu, bus);
}

fn and_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<And>(cpu, bus);
}

fn rol_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<OnA<Rol>>(cpu, bus);
}

fn bit_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Bit>(cpu, bus);
}

fn and_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<And>(cpu, bus);
}

fn rol_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute_rmw::<Rol>(cpu, bus);
}

fn bmi_relative(cpu: &mut CPU, bus: &mut impl Bus) {
    relative_branch::<Bmi>(cpu, bus);
}

fn and_indirecty(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_indirecty::<And>(cpu, bus);
}

fn and_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex::<And>(cpu, bus);
}

fn rol_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex_rmw::<Rol>(cpu, bus);
}

fn sec_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Sec>(cpu, bus);
}

fn and_absolutey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutey::<And>(cpu, bus);
}

fn and_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex::<And>(cpu, bus);
}

fn rol_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex_rmw::<Rol>(cpu, bus);
}

fn rti_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // dummy read
            let _ = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // dummy read of the stack
            let _ = bus.read(u16::from_le_bytes([cpu.registers.sp, 0x01]));
            step_instruction!(cpu);
        }
        3 => {
            // pull status, B and unused bits are not real flags: keep the current ones
            let pulled = cpu.pop_stack(bus);
            let ignored = StatusFlags::BREAK_COMMAND | StatusFlags::UNUSED;
            cpu.status.0 = (pulled & !ignored) | (cpu.status.0 & ignored);
            step_instruction!(cpu);
        }
        4 => {
            // pull PCL
            let pcl = cpu.pop_stack(bus);
            cpu.registers.pc = u16::from_le_bytes([pcl, u16_high_byte(cpu.registers.pc)]);
            step_instruction!(cpu);
        }
        5 => {
            // pull PCH
            let pch = cpu.pop_stack(bus);
            cpu.registers.pc = u16::from_le_bytes([u16_low_byte(cpu.registers.pc), pch]);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

fn eor_xindirect(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_xindirect::<Eor>(cpu, bus);
}

fn eor_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Eor>(cpu, bus);
}

fn lsr_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage_rmw::<Lsr>(cpu, bus);
}

fn pha_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // dummy read
            let _ = bus.read(cpu.registers.pc);
            step_instruction!(cpu);
        }
        2 => {
            // push accumulator onto stack
            cpu.push_stack(bus, cpu.registers.a);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

fn eor_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<Eor>(cpu, bus);
}

fn lsr_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<OnA<Lsr>>(cpu, bus);
}

fn jmp_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of address
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // fetch high byte of address
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.registers.pc = u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

fn eor_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Eor>(cpu, bus);
}

fn lsr_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute_rmw::<Lsr>(cpu, bus);
}

fn bvc_relative(cpu: &mut CPU, bus: &mut impl Bus) {
    relative_branch::<Bvc>(cpu, bus);
}

fn eor_indirecty(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_indirecty::<Eor>(cpu, bus);
}

fn eor_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex::<Eor>(cpu, bus);
}

fn lsr_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex_rmw::<Lsr>(cpu, bus);
}

fn cli_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Cli>(cpu, bus);
}

fn eor_absolutey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutey::<Eor>(cpu, bus);
}

fn eor_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex::<Eor>(cpu, bus);
}

fn lsr_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex_rmw::<Lsr>(cpu, bus);
}

fn rts_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // dummy read
            let _ = bus.read(cpu.registers.pc);
            step_instruction!(cpu);
        }
        2 => {
            // dummy read of the stack
            let _ = bus.read(u16::from_le_bytes([cpu.registers.sp, 0x01]));
            step_instruction!(cpu);
        }
        3 => {
            // pull PCL
            let pcl = cpu.pop_stack(bus);
            cpu.registers.pc = u16::from_le_bytes([pcl, u16_high_byte(cpu.registers.pc)]);
            step_instruction!(cpu);
        }
        4 => {
            // pull PCH
            let pch = cpu.pop_stack(bus);
            cpu.registers.pc = u16::from_le_bytes([u16_low_byte(cpu.registers.pc), pch]);
            step_instruction!(cpu);
        }
        5 => {
            // JSR pushed the address of its last byte: dummy read there, then
            // step past it to the instruction after the JSR
            let _ = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

fn adc_xindirect(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_xindirect::<Adc>(cpu, bus);
}

fn adc_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Adc>(cpu, bus);
}

fn ror_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage_rmw::<Ror>(cpu, bus);
}

fn pla_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // dummy read of the next instruction byte
            let _ = bus.read(cpu.registers.pc);
            step_instruction!(cpu);
        }
        2 => {
            // dummy read of the stack
            let _ = bus.read(u16::from_le_bytes([cpu.registers.sp, 0x01]));
            step_instruction!(cpu);
        }
        3 => {
            cpu.registers.a = cpu.pop_stack(bus);
            set_zn_flags(cpu, cpu.registers.a);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

fn adc_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<Adc>(cpu, bus);
}

fn ror_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<OnA<Ror>>(cpu, bus);
}

fn jmp_indirect(cpu: &mut CPU, bus: &mut impl Bus) {
    match cpu.instruction_cycle {
        1 => {
            // fetch low byte of pointer
            cpu.adl_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        2 => {
            // fetch high byte of pointer
            cpu.adh_latch = bus.read(cpu.registers.pc);
            cpu.increment_pc();
            step_instruction!(cpu);
        }
        3 => {
            // fetch low byte of address
            cpu.op_latch = bus.read(u16::from_le_bytes([cpu.adl_latch, cpu.adh_latch]));
            step_instruction!(cpu);
        }
        4 => {
            // fetch high byte of address and set pc
            let tmp = cpu.op_latch;
            cpu.op_latch = bus.read(u16::from_le_bytes([
                cpu.adl_latch.wrapping_add(1),
                cpu.adh_latch,
            ]));
            cpu.registers.pc = u16::from_le_bytes([tmp, cpu.op_latch]);
            end_instruction!(cpu);
        }
        _ => unreachable!(),
    }
}

fn adc_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Adc>(cpu, bus);
}

fn ror_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute_rmw::<Ror>(cpu, bus);
}

fn bvs_relative(cpu: &mut CPU, bus: &mut impl Bus) {
    relative_branch::<Bvs>(cpu, bus);
}

fn adc_indirecty(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_indirecty::<Adc>(cpu, bus);
}

fn adc_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex::<Adc>(cpu, bus);
}

fn ror_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex_rmw::<Ror>(cpu, bus);
}

fn sei_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Sei>(cpu, bus);
}

fn adc_absolutey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutey::<Adc>(cpu, bus);
}

fn sta_xindirect(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_xindirect_write::<Sta>(cpu, bus);
}

fn sty_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage_write::<Sty>(cpu, bus);
}

fn sta_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage_write::<Sta>(cpu, bus);
}

fn stx_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage_write::<Stx>(cpu, bus);
}

fn dey_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<OnY<Dec>>(cpu, bus);
}

fn txa_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Txa>(cpu, bus);
}

fn sty_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute_write::<Sty>(cpu, bus);
}

fn sta_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute_write::<Sta>(cpu, bus);
}

fn stx_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute_write::<Stx>(cpu, bus);
}

fn bcc_relative(cpu: &mut CPU, bus: &mut impl Bus) {
    relative_branch::<Bcc>(cpu, bus);
}

fn sta_indirecty(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_indirecty_write::<Sta>(cpu, bus);
}

fn sty_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex_write::<Sty>(cpu, bus);
}

fn sta_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex_write::<Sta>(cpu, bus);
}

fn stx_zeropagey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagey_write::<Stx>(cpu, bus);
}

fn tya_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Tya>(cpu, bus);
}

fn sta_absolutey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutey_write::<Sta>(cpu, bus);
}

fn txs_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Txs>(cpu, bus);
}

fn sta_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex_write::<Sta>(cpu, bus);
}

fn ldy_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<Ldy>(cpu, bus);
}

fn lda_xindirect(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_xindirect::<Lda>(cpu, bus);
}

fn ldx_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<Ldx>(cpu, bus);
}

fn ldy_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Ldy>(cpu, bus);
}

fn lda_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Lda>(cpu, bus);
}

fn ldx_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Ldx>(cpu, bus);
}

fn tay_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Tay>(cpu, bus);
}

fn lda_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<Lda>(cpu, bus);
}

fn tax_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Tax>(cpu, bus);
}

fn ldy_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Ldy>(cpu, bus);
}

fn lda_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Lda>(cpu, bus);
}

fn ldx_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Ldx>(cpu, bus);
}

fn bcs_relative(cpu: &mut CPU, bus: &mut impl Bus) {
    relative_branch::<Bcs>(cpu, bus);
}

fn lda_indirecty(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_indirecty::<Lda>(cpu, bus);
}

fn ldy_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex::<Ldy>(cpu, bus);
}

fn lda_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex::<Lda>(cpu, bus);
}

fn ldx_zeropagey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagey::<Ldx>(cpu, bus);
}

fn clv_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Clv>(cpu, bus);
}

fn lda_absolutey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutey::<Lda>(cpu, bus);
}

fn tsx_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Tsx>(cpu, bus);
}

fn ldy_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex::<Ldy>(cpu, bus);
}

fn lda_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex::<Lda>(cpu, bus);
}

fn ldx_absolutey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutey::<Ldx>(cpu, bus);
}

fn cpy_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<Cpy>(cpu, bus);
}

fn cmp_xindirect(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_xindirect::<Cmp>(cpu, bus);
}

fn cpy_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Cpy>(cpu, bus);
}

fn cmp_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Cmp>(cpu, bus);
}

fn dec_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage_rmw::<Dec>(cpu, bus);
}

fn iny_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<OnY<Inc>>(cpu, bus);
}

fn cmp_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<Cmp>(cpu, bus);
}

fn dex_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<OnX<Dec>>(cpu, bus);
}

fn cpy_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Cpy>(cpu, bus);
}

fn cmp_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Cmp>(cpu, bus);
}

fn dec_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute_rmw::<Dec>(cpu, bus);
}

fn bne_relative(cpu: &mut CPU, bus: &mut impl Bus) {
    relative_branch::<Bne>(cpu, bus);
}

fn cmp_indirecty(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_indirecty::<Cmp>(cpu, bus);
}

fn cmp_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex::<Cmp>(cpu, bus);
}

fn dec_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex_rmw::<Dec>(cpu, bus);
}

fn cld_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Cld>(cpu, bus);
}

fn cmp_absolutey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutey::<Cmp>(cpu, bus);
}

fn cmp_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex::<Cmp>(cpu, bus);
}

fn dec_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex_rmw::<Dec>(cpu, bus);
}

fn cpx_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<Cpx>(cpu, bus);
}

fn sbc_xindirect(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_xindirect::<Sbc>(cpu, bus);
}

fn cpx_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Cpx>(cpu, bus);
}

fn sbc_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage::<Sbc>(cpu, bus);
}

fn inc_zeropage(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropage_rmw::<Inc>(cpu, bus);
}

fn inx_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<OnX<Inc>>(cpu, bus);
}

fn sbc_immediate(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_immediate::<Sbc>(cpu, bus);
}

fn nop_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Nop>(cpu, bus);
}

fn cpx_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Cpx>(cpu, bus);
}

fn sbc_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute::<Sbc>(cpu, bus);
}

fn inc_absolute(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolute_rmw::<Inc>(cpu, bus);
}

fn beq_relative(cpu: &mut CPU, bus: &mut impl Bus) {
    relative_branch::<Beq>(cpu, bus);
}

fn adc_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex::<Adc>(cpu, bus);
}

fn ror_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex_rmw::<Ror>(cpu, bus);
}

fn sbc_indirecty(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_indirecty::<Sbc>(cpu, bus);
}

fn sbc_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex::<Sbc>(cpu, bus);
}

fn inc_zeropagex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_zeropagex_rmw::<Inc>(cpu, bus);
}

fn sed_implied(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_implied::<Sed>(cpu, bus);
}

fn sbc_absolutey(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutey::<Sbc>(cpu, bus);
}

fn sbc_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex::<Sbc>(cpu, bus);
}

fn inc_absolutex(cpu: &mut CPU, bus: &mut impl Bus) {
    operation_absolutex_rmw::<Inc>(cpu, bus);
}

instructions! {
    0x00 => BRK, Implied, 7, [2,3,4], brk_implied;
    0x01 => ORA, XIndirect, 6, [], ora_xindirect;
    0x05 => ORA, ZeroPage, 3, [], ora_zeropage;
    0x06 => ASL, ZeroPage, 5, [3, 4], asl_zeropage;
    0x08 => PHP, Implied, 3, [2], php_implied;
    0x09 => ORA, Immediate, 2, [], ora_immediate;
    0x0A => ASL, Implied, 2, [], asl_implied;
    0x0D => ORA, Absolute, 4, [], ora_absolute;
    0x0E => ASL, Absolute, 6, [4, 5], asl_absolute;
    0x10 => BPL, Relative, 2, [], bpl_relative;
    0x11 => ORA, IndirectY, 5, [], ora_indirecty;
    0x15 => ORA, ZeroPageX, 4, [], ora_zeropagex;
    0x16 => ASL, ZeroPageX, 6, [4, 5], asl_zeropagex;
    0x18 => CLC, Implied, 2, [], clc_implied;
    0x19 => ORA, AbsoluteY, 4, [], ora_absolutey;
    0x1D => ORA, AbsoluteX, 4, [], ora_absolutex;
    0x1E => ASL, AbsoluteX, 7, [5, 6], asl_absolutex;
    0x20 => JSR, Absolute, 6, [3, 4], jsr_absolute;
    0x21 => AND, XIndirect, 6, [], and_xindirect;
    0x24 => BIT, ZeroPage, 3, [], bit_zeropage;
    0x25 => AND, ZeroPage, 3, [], and_zeropage;
    0x26 => ROL, ZeroPage, 5, [3, 4], rol_zeropage;
    0x28 => PLP, Implied, 4, [], plp_implied;
    0x29 => AND, Immediate, 2, [], and_immediate;
    0x2A => ROL, Implied, 2, [], rol_implied;
    0x2C => BIT, Absolute, 4, [], bit_absolute;
    0x2D => AND, Absolute, 4, [], and_absolute;
    0x2E => ROL, Absolute, 6, [4, 5], rol_absolute;
    0x30 => BMI, Relative, 2, [], bmi_relative;
    0x31 => AND, IndirectY, 5, [], and_indirecty;
    0x35 => AND, ZeroPageX, 4, [], and_zeropagex;
    0x36 => ROL, ZeroPageX, 6, [4, 5], rol_zeropagex;
    0x38 => SEC, Implied, 2, [], sec_implied;
    0x39 => AND, AbsoluteY, 4, [], and_absolutey;
    0x3D => AND, AbsoluteX, 4, [], and_absolutex;
    0x3E => ROL, AbsoluteX, 7, [5, 6], rol_absolutex;
    0x40 => RTI, Implied, 6, [], rti_implied;
    0x41 => EOR, XIndirect, 6, [], eor_xindirect;
    0x45 => EOR, ZeroPage, 3, [], eor_zeropage;
    0x46 => LSR, ZeroPage, 5, [3, 4], lsr_zeropage;
    0x48 => PHA, Implied, 3, [2], pha_implied;
    0x49 => EOR, Immediate, 2, [], eor_immediate;
    0x4A => LSR, Implied, 2, [], lsr_implied;
    0x4C => JMP, Absolute, 3, [], jmp_absolute;
    0x4D => EOR, Absolute, 4, [], eor_absolute;
    0x4E => LSR, Absolute, 6, [4, 5], lsr_absolute;
    0x50 => BVC, Relative, 2, [], bvc_relative;
    0x51 => EOR, IndirectY, 5, [], eor_indirecty;
    0x55 => EOR, ZeroPageX, 4, [], eor_zeropagex;
    0x56 => LSR, ZeroPageX, 6, [4, 5], lsr_zeropagex;
    0x58 => CLI, Implied, 2, [], cli_implied;
    0x59 => EOR, AbsoluteY, 4, [], eor_absolutey;
    0x5D => EOR, AbsoluteX, 4, [], eor_absolutex;
    0x5E => LSR, AbsoluteX, 7, [5, 6], lsr_absolutex;
    0x60 => RTS, Implied, 6, [], rts_implied;
    0x61 => ADC, XIndirect, 6, [], adc_xindirect;
    0x65 => ADC, ZeroPage, 3, [], adc_zeropage;
    0x66 => ROR, ZeroPage, 5, [3, 4], ror_zeropage;
    0x68 => PLA, Implied, 4, [], pla_implied;
    0x69 => ADC, Immediate, 2, [], adc_immediate;
    0x6A => ROR, Implied, 2, [], ror_implied;
    0x6C => JMP, Indirect, 5, [], jmp_indirect;
    0x6D => ADC, Absolute, 4, [], adc_absolute;
    0x6E => ROR, Absolute, 6, [4, 5], ror_absolute;
    0x70 => BVS, Relative, 2, [], bvs_relative;
    0x71 => ADC, IndirectY, 5, [], adc_indirecty;
    0x75 => ADC, ZeroPageX, 4, [], adc_zeropagex;
    0x76 => ROR, ZeroPageX, 6, [4, 5], ror_zeropagex;
    0x78 => SEI, Implied, 2, [], sei_implied;
    0x79 => ADC, AbsoluteY, 4, [], adc_absolutey;
    0x7D => ADC, AbsoluteX, 4, [], adc_absolutex;
    0x7E => ROR, AbsoluteX, 7, [5, 6], ror_absolutex;
    0x81 => STA, XIndirect, 6, [5], sta_xindirect;
    0x84 => STY, ZeroPage, 3, [2], sty_zeropage;
    0x85 => STA, ZeroPage, 3, [2], sta_zeropage;
    0x86 => STX, ZeroPage, 3, [2], stx_zeropage;
    0x88 => DEY, Implied, 2, [], dey_implied;
    0x8A => TXA, Implied, 2, [], txa_implied;
    0x8C => STY, Absolute, 4, [3], sty_absolute;
    0x8D => STA, Absolute, 4, [3], sta_absolute;
    0x8E => STX, Absolute, 4, [3], stx_absolute;
    0x90 => BCC, Relative, 2, [], bcc_relative;
    0x91 => STA, IndirectY, 6, [5], sta_indirecty;
    0x94 => STY, ZeroPageX, 4, [3], sty_zeropagex;
    0x95 => STA, ZeroPageX, 4, [3], sta_zeropagex;
    0x96 => STX, ZeroPageY, 4, [3], stx_zeropagey;
    0x98 => TYA, Implied, 2, [], tya_implied;
    0x99 => STA, AbsoluteY, 5, [4], sta_absolutey;
    0x9A => TXS, Implied, 2, [], txs_implied;
    0x9D => STA, AbsoluteX, 5, [4], sta_absolutex;
    0xA0 => LDY, Immediate, 2, [], ldy_immediate;
    0xA1 => LDA, XIndirect, 6, [], lda_xindirect;
    0xA2 => LDX, Immediate, 2, [], ldx_immediate;
    0xA4 => LDY, ZeroPage, 3, [], ldy_zeropage;
    0xA5 => LDA, ZeroPage, 3, [], lda_zeropage;
    0xA6 => LDX, ZeroPage, 3, [], ldx_zeropage;
    0xA8 => TAY, Implied, 2, [], tay_implied;
    0xA9 => LDA, Immediate, 2, [], lda_immediate;
    0xAA => TAX, Implied, 2, [], tax_implied;
    0xAC => LDY, Absolute, 4, [], ldy_absolute;
    0xAD => LDA, Absolute, 4, [], lda_absolute;
    0xAE => LDX, Absolute, 4, [], ldx_absolute;
    0xB0 => BCS, Relative, 2, [], bcs_relative;
    0xB1 => LDA, IndirectY, 5, [], lda_indirecty;
    0xB4 => LDY, ZeroPageX, 4, [], ldy_zeropagex;
    0xB5 => LDA, ZeroPageX, 4, [], lda_zeropagex;
    0xB6 => LDX, ZeroPageY, 4, [], ldx_zeropagey;
    0xB8 => CLV, Implied, 2, [], clv_implied;
    0xB9 => LDA, AbsoluteY, 4, [], lda_absolutey;
    0xBA => TSX, Implied, 2, [], tsx_implied;
    0xBC => LDY, AbsoluteX, 4, [], ldy_absolutex;
    0xBD => LDA, AbsoluteX, 4, [], lda_absolutex;
    0xBE => LDX, AbsoluteY, 4, [], ldx_absolutey;
    0xC0 => CPY, Immediate, 2, [], cpy_immediate;
    0xC1 => CMP, XIndirect, 6, [], cmp_xindirect;
    0xC4 => CPY, ZeroPage, 3, [], cpy_zeropage;
    0xC5 => CMP, ZeroPage, 3, [], cmp_zeropage;
    0xC6 => DEC, ZeroPage, 5, [3, 4], dec_zeropage;
    0xC8 => INY, Implied, 2, [], iny_implied;
    0xC9 => CMP, Immediate, 2, [], cmp_immediate;
    0xCA => DEX, Implied, 2, [], dex_implied;
    0xCC => CPY, Absolute, 4, [], cpy_absolute;
    0xCD => CMP, Absolute, 4, [], cmp_absolute;
    0xCE => DEC, Absolute, 6, [4, 5], dec_absolute;
    0xD0 => BNE, Relative, 2, [], bne_relative;
    0xD1 => CMP, IndirectY, 5, [], cmp_indirecty;
    0xD5 => CMP, ZeroPageX, 4, [], cmp_zeropagex;
    0xD6 => DEC, ZeroPageX, 6, [4, 5], dec_zeropagex;
    0xD8 => CLD, Implied, 2, [], cld_implied;
    0xD9 => CMP, AbsoluteY, 4, [], cmp_absolutey;
    0xDD => CMP, AbsoluteX, 4, [], cmp_absolutex;
    0xDE => DEC, AbsoluteX, 7, [5, 6], dec_absolutex;
    0xE0 => CPX, Immediate, 2, [], cpx_immediate;
    0xE1 => SBC, XIndirect, 6, [], sbc_xindirect;
    0xE4 => CPX, ZeroPage, 3, [], cpx_zeropage;
    0xE5 => SBC, ZeroPage, 3, [], sbc_zeropage;
    0xE6 => INC, ZeroPage, 5, [3, 4], inc_zeropage;
    0xE8 => INX, Implied, 2, [], inx_implied;
    0xE9 => SBC, Immediate, 2, [], sbc_immediate;
    0xEA => NOP, Implied, 2, [], nop_implied;
    0xEC => CPX, Absolute, 4, [], cpx_absolute;
    0xED => SBC, Absolute, 4, [], sbc_absolute;
    0xEE => INC, Absolute, 6, [4, 5], inc_absolute;
    0xF0 => BEQ, Relative, 2, [], beq_relative;
    0xF1 => SBC, IndirectY, 5, [], sbc_indirecty;
    0xF5 => SBC, ZeroPageX, 4, [], sbc_zeropagex;
    0xF6 => INC, ZeroPageX, 6, [4, 5], inc_zeropagex;
    0xF8 => SED, Implied, 2, [], sed_implied;
    0xF9 => SBC, AbsoluteY, 4, [], sbc_absolutey;
    0xFD => SBC, AbsoluteX, 4, [], sbc_absolutex;
    0xFE => INC, AbsoluteX, 7, [5, 6], inc_absolutex;
}

pub fn is_write_cycle(op_code: u8, instruction_cycle: u8) -> bool {
    let Some(info) = INFO[usize::from(op_code)] else {
        return false;
    };
    (info.write_cycles & (1 << instruction_cycle)) != 0
}

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
    Relative,
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

struct Clc;
impl Operation for Clc {
    #[inline(always)]
    fn execute(cpu: &mut CPU) {
        cpu.status.set(StatusFlags::CARRY, false);
    }
}
impl ImpliedOp for Clc {}

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
}

pub fn is_write_cycle(op_code: u8, instruction_cycle: u8) -> bool {
    let Some(info) = INFO[usize::from(op_code)] else {
        return false;
    };
    (info.write_cycles & (1 << instruction_cycle)) != 0
}

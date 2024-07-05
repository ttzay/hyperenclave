/* author : zhou_yuz zhou_yuz@whu.edu.cn */

use core::fmt::Debug;
use aarch64_cpu::registers::*;
use tock_registers::interfaces::{Readable, Writeable};

use crate::arch::segmentation::Segment;
use crate::arch::tables::{GDTStruct, IDTStruct, GDT, IDT};

const SAVED_LINUX_REGS: usize = 31;

#[derive(Debug)]
pub struct LinuxContext {

    /*aarch64 架构有 31 个 64 位通用寄存器，命名为 X0 至 X30，其中一些寄存器有特定用途：

        X0-X7: 参数寄存器，用于传递函数参数和返回值。
        X8: 间接结果地址寄存器，常用于系统调用。
        X9-X15: 临时寄存器，无特殊用途，可以用作任意目的。
        X16-X17: 内部调用寄存器，常用于过程间调用（例如保存链接地址）。
        X18: 平台寄存器，保留给平台使用。
        X19-X28: 保存寄存器，调用者需要在使用前保存和恢复它们的值。
        X29: 帧指针寄存器（FP），用于帧指针。
        X30: 链接寄存器（LR），保存函数返回地址。
        SP (X31): 堆栈指针（Stack Pointer），用于指向当前堆栈顶。

        PC: 程序计数器（Program Counter），保存当前指令地址

        NZCV: 状态标志寄存器，包含条件标志（Negative, Zero, Carry, Overflow）。
        DAIF: 中断屏蔽寄存器，控制中断的使能和屏蔽（Debug, SError, IRQ, FIQ）

        SCTLR_EL1: 系统控制寄存器，控制系统行为。
        TTBR0_EL1: 译码基址寄存器，指向当前页表的基地址。
        TTBR1_EL1: 高地址范围的译码基址寄存器。
        TCR_EL1: 翻译控制寄存器，配置地址翻译。
     */


    // TODO 完善LinuxContext结构体，补充arm架构中切换上下文的寄存器内容
    pub sp: u64,
    pub pc: u64,

    pub x29: u64,
    pub x30: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,

    // Other architecture-specific fields
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct GuestRegisters {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,
    pub x30: u64,
    pub sp: u64,
    pub pc: u64,
}

macro_rules! save_regs_to_stack {
    () => {
        "
        stp x29, x30, [sp, #-16]!
        stp x27, x28, [sp, #-16]!
        stp x25, x26, [sp, #-16]!
        stp x23, x24, [sp, #-16]!
        stp x21, x22, [sp, #-16]!
        stp x19, x20, [sp, #-16]!
        stp x17, x18, [sp, #-16]!
        stp x15, x16, [sp, #-16]!
        stp x13, x14, [sp, #-16]!
        stp x11, x12, [sp, #-16]!
        stp x9, x10, [sp, #-16]!
        stp x7, x8, [sp, #-16]!
        stp x5, x6, [sp, #-16]!
        stp x3, x4, [sp, #-16]!
        stp x1, x2, [sp, #-16]!
        str x0, [sp, #-8]!
        "
    };
}

macro_rules! restore_regs_from_stack {
    () => {
        "
        ldr x0, [sp], #8
        ldp x1, x2, [sp], #16
        ldp x3, x4, [sp], #16
        ldp x5, x6, [sp], #16
        ldp x7, x8, [sp], #16
        ldp x9, x10, [sp], #16
        ldp x11, x12, [sp], #16
        ldp x13, x14, [sp], #16
        ldp x15, x16, [sp], #16
        ldp x17, x18, [sp], #16
        ldp x19, x20, [sp], #16
        ldp x21, x22, [sp], #16
        ldp x23, x24, [sp], #16
        ldp x25, x26, [sp], #16
        ldp x27, x28, [sp], #16
        ldp x29, x30, [sp], #16
        "
    };
}

impl LinuxContext {
    pub fn load_from(linux_sp: usize) -> Self {
        let regs = unsafe { core::slice::from_raw_parts(linux_sp as *const u64, SAVED_LINUX_REGS) };

        let ret = Self {
            sp: regs[0],
            x29: regs[1],
            x30: regs[2],
            x1: regs[3],
            x2: regs[4],
            x3: regs[5],
            x4: regs[6],
            x5: regs[7],
            x6: regs[8],
            x7: regs[9],
            x8: regs[10],
            x9: regs[11],
            x10: regs[12],
            x11: regs[13],
            x12: regs[14],
            x13: regs[15],
            x14: regs[16],
            x15: regs[17],
            x16: regs[18],
            x17: regs[19],
            x18: regs[20],
            x19: regs[21],
            x20: regs[22],
            x21: regs[23],
            x22: regs[24],
            x23: regs[25],
            x24: regs[26],
            x25: regs[27],
            x26: regs[28],
            x27: regs[29],
            x28: regs[30],
            sp: regs[31],
            pc: regs[32],
        };

        // Set up other architecture-specific context if needed

        ret
    }

    pub fn restore(&self) {
        unsafe {
            // Restore registers and other architecture-specific state
            asm!(
                "
                mov sp, {sp}
                mov x0, {pc}
                br x0
                ",
                sp = in(reg) self.sp,
                pc = in(reg) self.pc,
                options(noreturn),
            );
        }
    }
}

impl GuestRegisters {
    pub fn return_to_linux(&self, linux: &LinuxContext) -> ! {
        unsafe {
            asm!(
                "mov sp, {linux_sp}",
                "ldr x0, {linux_pc}",
                restore_regs_from_stack!(),
                "br x0",
                linux_sp = in(reg) linux.sp,
                linux_pc = in(reg) linux.pc,
                options(noreturn),
            );
        }
    }
}

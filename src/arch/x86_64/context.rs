// Copyright (C) 2023 Ant Group CO., Ltd. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::fmt::Debug;

use libvmm::msr::Msr;
use x86::{segmentation, segmentation::SegmentSelector, task};
use x86_64::registers::control::{Cr0, Cr0Flags, Cr3, Cr3Flags, Cr4, Cr4Flags};
use x86_64::{addr::PhysAddr, structures::paging::PhysFrame, structures::DescriptorTablePointer};

use super::segmentation::Segment;
use super::tables::{GDTStruct, IDTStruct, GDT, IDT};

const SAVED_LINUX_REGS: usize = 7;

#[derive(Debug)]
pub struct LinuxContext {

    // 用在percpu位置，CPU的寄存器状态
    pub rsp: u64, // 栈指针寄存器，指向当前堆栈的顶部。
    pub rip: u64, // 指令指针寄存器，存储下一条将要执行的指令地址。

    pub r15: u64, // 通用寄存器r15，用于保存临时数据。
    pub r14: u64, // 通用寄存器r14，用于保存临时数据。
    pub r13: u64, // 通用寄存器r13，用于保存临时数据。
    pub r12: u64, // 通用寄存器r12，用于保存临时数据。
    pub rbx: u64, // 基址寄存器，通常用于保存指针或计数器。
    pub rbp: u64, // 基址指针寄存器，指向当前栈帧的基址。

    /*
        CS: 代码段寄存器（Code Segment），指向代码段的基址。
        DS: 数据段寄存器（Data Segment），指向数据段的基址。
        SS: 堆栈段寄存器（Stack Segment），指向堆栈段的基址。
        ES, FS, GS: 额外段寄存器，用于指向额外的数据段或特定用途。
     */
    pub cs: Segment, // 代码段寄存器，指向代码段的基址。
    pub ds: Segment, // 数据段寄存器，指向数据段的基址。
    pub es: Segment, // 额外段寄存器，指向额外的数据段。
    pub fs: Segment, // 额外段寄存器，常用于线程本地存储。
    pub gs: Segment, // 额外段寄存器，常用于内核数据结构。
    pub tss: Segment, // 任务状态段寄存器，包含任务切换相关信息。
    pub gdt: DescriptorTablePointer, // 全局描述符表指针，指向全局描述符表。
    pub idt: DescriptorTablePointer, // 中断描述符表指针，指向中断描述符表。

    pub cr0: Cr0Flags, // 控制寄存器0，控制CPU的操作模式。
    pub cr3: u64, // 控制寄存器3，包含页目录的物理地址，用于内存分页。
    pub cr4: Cr4Flags, // 控制寄存器4，控制各种扩展功能。

    pub efer: u64, // 扩展功能寄存器，用于启用64位模式等。
    pub lstar: u64, // 系统调用目标地址寄存器。
    pub pat: u64, // 页属性表寄存器，控制内存页的缓存属性。
    pub kernel_gsbase: u64, // GS段寄存器的基址，用于内核模式。
    pub star: u64, // 系统调用相关寄存器。
    pub cstar: u64, // 兼容模式下的系统调用目标地址寄存器。
    pub fmask: u64, // 系统调用屏蔽位寄存器。
    pub mtrr_def_type: u64, // 内存类型范围寄存器的默认类型。
}


#[repr(C)]
#[derive(Debug, Default)]
pub struct GuestRegisters {
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    _unused_rsp: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

macro_rules! save_regs_to_stack {
    () => {
        "
        push r15
        push r14
        push r13
        push r12
        push r11
        push r10
        push r9
        push r8
        push rdi
        push rsi
        push rbp
        sub rsp, 8
        push rbx
        push rdx
        push rcx
        push rax"
    };
}

macro_rules! restore_regs_from_stack {
    () => {
        "
        pop rax
        pop rcx
        pop rdx
        pop rbx
        add rsp, 8
        pop rbp
        pop rsi
        pop rdi
        pop r8
        pop r9
        pop r10
        pop r11
        pop r12
        pop r13
        pop r14
        pop r15"
    };
}

impl LinuxContext {
    pub fn load_from(linux_sp: usize) -> Self {
        let regs = unsafe { core::slice::from_raw_parts(linux_sp as *const u64, SAVED_LINUX_REGS) };
        let gdt = GDTStruct::sgdt();
        let mut fs = Segment::from_selector(segmentation::fs(), &gdt);
        let mut gs = Segment::from_selector(segmentation::gs(), &gdt);
        fs.base = Msr::IA32_FS_BASE.read();
        gs.base = Msr::IA32_GS_BASE.read();

        let ret = Self {
            rsp: regs.as_ptr_range().end as _,
            r15: regs[0],
            r14: regs[1],
            r13: regs[2],
            r12: regs[3],
            rbx: regs[4],
            rbp: regs[5],
            rip: regs[6],
            cs: Segment::from_selector(segmentation::cs(), &gdt),
            ds: Segment::from_selector(segmentation::ds(), &gdt),
            es: Segment::from_selector(segmentation::es(), &gdt),
            fs,
            gs,
            tss: Segment::from_selector(task::tr(), &gdt),
            gdt,
            idt: IDTStruct::sidt(),
            cr0: Cr0::read(),
            cr3: Cr3::read().0.start_address().as_u64(),
            cr4: Cr4::read(),
            efer: Msr::IA32_EFER.read(),
            lstar: Msr::IA32_LSTAR.read(),
            kernel_gsbase: Msr::IA32_KERNEL_GSBASE.read(),
            star: Msr::IA32_STAR.read(),
            cstar: Msr::IA32_CSTAR.read(),
            fmask: Msr::IA32_FMASK.read(),
            pat: Msr::IA32_PAT.read(),
            mtrr_def_type: Msr::IA32_MTRR_DEF_TYPE.read(),
        };

        // Setup new GDT, IDT, CS, TSS
        GDT.lock().load();
        unsafe {
            segmentation::load_cs(GDTStruct::KCODE_SELECTOR);
            segmentation::load_ds(SegmentSelector::from_raw(0));
            segmentation::load_es(SegmentSelector::from_raw(0));
            segmentation::load_ss(SegmentSelector::from_raw(0));
        }
        IDT.lock().load();
        GDT.lock().load_tss(GDTStruct::TSS_SELECTOR);

        // PAT0: WB, PAT1: WC, PAT2: UC
        unsafe { Msr::IA32_PAT.write(0x070106) };

        ret
    }

    pub fn restore(&self) {
        unsafe {
            Msr::IA32_PAT.write(self.pat);
            Msr::IA32_EFER.write(self.efer);
            Msr::IA32_KERNEL_GSBASE.write(self.kernel_gsbase);
            Msr::IA32_STAR.write(self.star);
            Msr::IA32_CSTAR.write(self.cstar);
            Msr::IA32_FMASK.write(self.fmask);

            Cr0::write(self.cr0);
            Cr4::write(self.cr4);
            // cr3 must be last in case cr4 enables PCID
            Cr3::write(
                PhysFrame::containing_address(PhysAddr::new(self.cr3)),
                Cr3Flags::empty(), // clear PCID
            );

            // Copy Linux TSS descriptor into our GDT, clearing the busy flag,
            // then reload TR from it. We can't use Linux' GDT as it is r/o.
            {
                let mut hv_gdt_lock = GDT.lock();
                let hv_gdt = GDTStruct::table_of_mut(hv_gdt_lock.pointer());
                let liunx_gdt = GDTStruct::table_of(&self.gdt);
                let tss_idx = self.tss.selector.index() as usize;
                hv_gdt[tss_idx] = liunx_gdt[tss_idx];
                hv_gdt[tss_idx + 1] = liunx_gdt[tss_idx + 1];
                hv_gdt_lock.load_tss(self.tss.selector);
            }

            GDTStruct::lgdt(&self.gdt);
            IDTStruct::lidt(&self.idt);

            segmentation::load_cs(self.cs.selector); // XXX: failed to swtich to user CS
            segmentation::load_ds(self.ds.selector);
            segmentation::load_es(self.es.selector);
            segmentation::load_fs(self.fs.selector);
            segmentation::load_gs(self.gs.selector);

            Msr::IA32_FS_BASE.write(self.fs.base);
            Msr::IA32_GS_BASE.write(self.gs.base);
        }
    }
}

impl GuestRegisters {
    pub fn return_to_linux(&self, linux: &LinuxContext) -> ! {
        unsafe {
            asm!(
                "mov rsp, {linux_rsp}",
                "push {linux_rip}",
                "mov rcx, rsp",
                "mov rsp, {guest_regs}",
                "mov [rsp + {guest_regs_size}], rcx",
                restore_regs_from_stack!(),
                "pop rsp",
                "ret",
                linux_rsp = in(reg) linux.rsp,
                linux_rip = in(reg) linux.rip,
                guest_regs = in(reg) self,
                guest_regs_size = const core::mem::size_of::<Self>(),
                options(noreturn),
            );
        }
    }
}

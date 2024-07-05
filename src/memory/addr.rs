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

//! Definition of phyical and virtual addresses.

#![allow(dead_code)]

use crate::consts::{HV_BASE, PAGE_SIZE, SME_C_BIT_OFFSET};

pub type VirtAddr = usize;
pub type PhysAddr = usize;

pub type GuestVirtAddr = usize;
pub type GuestPhysAddr = usize;

pub type HostVirtAddr = VirtAddr;
pub type HostPhysAddr = PhysAddr;

// 使用lazy_static宏定义了一个静态变量PHYS_VIRT_OFFSET，它在第一次使用时被初始化。它的值是HV_BASE减去从配置中获取的物理内存起始地址
lazy_static! {
    static ref PHYS_VIRT_OFFSET: usize = HV_BASE
        - crate::config::HvSystemConfig::get()
            .hypervisor_memory
            .phys_start as usize;
}

pub fn phys_encrypted(paddr: PhysAddr) -> PhysAddr {
    // 将物理地址paddr与SME_C_BIT_OFFSET按位或操作，以启用加密
    paddr | SME_C_BIT_OFFSET
}

pub fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    // 将虚拟地址转换为物理地址，减去PHYS_VIRT_OFFSET的值
    vaddr - *PHYS_VIRT_OFFSET
}

pub fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    // 将物理地址转换为虚拟地址，先与SME_C_BIT_OFFSET减1按位与，然后加上PHYS_VIRT_OFFSET的值
    (paddr & (SME_C_BIT_OFFSET.wrapping_sub(1))) + *PHYS_VIRT_OFFSET
}

pub const fn align_down(addr: usize) -> usize {
    // 将地址向下对齐到页面边界
    addr & !(PAGE_SIZE - 1)
}

pub const fn align_up(addr: usize) -> usize {
    // 将地址向上对齐到页面边界
    (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

pub const fn is_aligned(addr: usize) -> bool {
    // 检查地址是否已经对齐
    page_offset(addr) == 0
}

pub const fn page_count(size: usize) -> usize {
    // 计算给定大小需要的页面数量
    align_up(size) / PAGE_SIZE
}

pub const fn page_offset(addr: usize) -> usize {
    // 计算地址在页面中的偏移
    addr & (PAGE_SIZE - 1)
}

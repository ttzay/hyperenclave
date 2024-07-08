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

use super::cpuid::CpuFeatures;
use crate::error::HvResult;

pub fn id() -> usize {
    // 创建一个新的CpuId实例，并获取CPU特性信息，然后返回初始的本地APIC ID
    super::cpuid::CpuId::new()
        .get_feature_info()
        .unwrap()
        .initial_local_apic_id() as usize
}

pub fn time_now() -> u64 {
    // 获取当前时间
    unsafe { core::arch::x86_64::_rdtsc() }
}

pub fn check_cpuid() -> HvResult {
    // 检查CPU是否支持PAE（Physical Address Extension）和OSXSAVE（操作系统扩展保存/恢复）。如果不支持任一功能，则返回错误，否则返回成功
    let features = CpuFeatures::new();
    // CR4.PAE will be set in HOST_CR4
    if !features.has_pae() {
        return hv_result_err!(ENODEV, "PAE is not supported!");
    }
    // CR4.OSXSAVE will be set in HOST_CR4
    if !features.has_xsave() {
        return hv_result_err!(ENODEV, "OSXSAVE is not supported!");
    }
    Ok(())
}

// def 缓存行的大小（64字节）
#[allow(dead_code)]
const CACHE_LINE_SIZE: usize = 64;

#[allow(dead_code)]
pub fn clflush_cache_range(vaddr: usize, length: usize) {

    //用于刷新指定范围内的缓存行。clflush_cache_range函数在使用clflush指令前后使用mfence指令进行内存屏障，以避免顺序问题。
    //这个函数也是在unsafe块中实现的，因为它直接调用了底层的汇编指令
    // clflush is an unordered instruction which needs fencing with mfence or
    // sfence to avoid ordering issues.
    unsafe { asm!("mfence") };
    for addr in (vaddr..(vaddr + length)).step_by(CACHE_LINE_SIZE) {
        unsafe {
            core::arch::x86_64::_mm_clflush(addr as *const u8);
        }
    }
    unsafe { asm!("mfence") };
}

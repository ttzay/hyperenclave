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

use crate::header::HvHeader;
use crate::HvResult;

use core::mem::size_of;

// NR_CPUS：最大支持的CPU数量，设置为512
const NR_CPUS: usize = 512;
// BITS_PER_BYTE：每个字节的位数，设置为8
const BITS_PER_BYTE: usize = 8;
// BITS_PER_USIZE：每个usize的位数，设置为8 * usize的字节数
const BITS_PER_USIZE: usize = size_of::<usize>() * BITS_PER_BYTE;
// CPU_MASK_LEN：CPU掩码的长度，设置为NR_CPUS / BITS_PER_USIZE
pub const CPU_MASK_LEN: usize = (NR_CPUS + BITS_PER_USIZE - 1) / BITS_PER_USIZE;

#[repr(C)]
#[derive(Debug, Default)]
// 定义了一个CpuMask结构体，包含一个长度为CPU_MASK_LEN的usize数组
pub struct CpuMask([usize; CPU_MASK_LEN]);

impl CpuMask {
    pub fn set_cpu(&mut self, cpuid: usize) {
        self.0[cpuid / BITS_PER_USIZE] |= 1 << (cpuid % BITS_PER_USIZE);
    }

    pub fn clear_cpu(&mut self, cpuid: usize) {
        self.0[cpuid / BITS_PER_USIZE] &= !(1 << (cpuid % BITS_PER_USIZE));
    }

    pub fn test_cpu(&self, cpuid: usize) -> usize {
        self.0[cpuid / BITS_PER_USIZE] & (1 << (cpuid % BITS_PER_USIZE))
    }

    pub fn clear(&mut self) {
        self.0 = [0; CPU_MASK_LEN];
    }
}

pub fn check_max_cpus() -> HvResult {
    let max_cpus = HvHeader::get().max_cpus as usize;

    if max_cpus > NR_CPUS {
        println!(
            "Invalid max_cpus: {}, supported max cpus are {}",
            max_cpus, NR_CPUS
        );
        return hv_result_err!(EINVAL);
    }
    Ok(())
}

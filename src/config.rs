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
use core::{mem::size_of, slice};

use crate::consts::HV_BASE;
use crate::header::HvHeader;
use crate::memory::MemFlags;
use crate::percpu::PER_CPU_SIZE;

// 最大iommu单元数
const HV_MAX_IOMMU_UNITS: usize = 16;
// 最大rmrr范围
const HV_MAX_RMRR_RANGE: usize = 4;

#[derive(Debug)]
#[repr(C, packed)]
pub struct HvMemoryRegion {
    // 描述内存区域的结构体，包括物理起始地址、虚拟起始地址、大小和内存标志
    pub phys_start: u64,
    pub virt_start: u64,
    pub size: u64,
    pub flags: MemFlags,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct HvIommuInfo {
    // 描述IOMMU信息的结构体，包括基地址和大小
    pub base: u64,
    pub size: u32,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct HvRmrrRange {
    // 描述RMRR（Reserved Memory Region Report）范围的结构体，包括基地址和限制地址
    pub base: u64,
    pub limit: u64,
}

#[cfg(target_arch = "x86_64")]
#[derive(Debug)]
#[repr(C, packed)]
struct ArchPlatformInfo {
    // 描述架构特定平台信息的结构体，包括IOMMU单元和RMRR范围，仅在x86_64架构上定义
    // TODO  need arm64 support
    iommu_units: [HvIommuInfo; HV_MAX_IOMMU_UNITS],
    rmrr_ranges: [HvRmrrRange; HV_MAX_RMRR_RANGE],
}

#[derive(Debug)]
#[repr(C, packed)]
struct PlatformInfo {
    // 包含架构平台信息的结构体
    arch: ArchPlatformInfo,
}

/// General descriptor of the system.
#[derive(Debug)]
#[repr(C, packed)]
pub struct HvSystemConfig {
    // 描述系统配置的结构体，包括虚拟机监控器内存、平台信息和内存区域数量
    pub hypervisor_memory: HvMemoryRegion,
    platform_info: PlatformInfo,
    num_memory_regions: u32,
    // ConfigLayout placed here.
}

/// A dummy layout with all variant-size fields empty.
#[derive(Debug)]
#[repr(C, packed)]
struct ConfigLayout {
    // 描述配置布局的结构体，包含变长的内存区域数组
    mem_regions: [HvMemoryRegion; 0],
}

impl HvSystemConfig {
    pub fn get<'a>() -> &'a Self {
        // 获取系统配置的静态方法，计算虚拟机监控器核心和每CPU大小的总和，然后返回指向系统配置的指针
        let header = HvHeader::get();
        let core_and_percpu_size =
            header.core_size as usize + header.max_cpus as usize * PER_CPU_SIZE;
        unsafe { &*((HV_BASE + core_and_percpu_size) as *const Self) }
    }

    fn config_ptr<T>(&self) -> *const T {
        // 返回指向系统配置中指定类型字段的指针
        unsafe { (self as *const HvSystemConfig).add(1) as _ }
    }

    pub const fn size(&self) -> usize {
        // 计算系统配置的总大小，包括所有内存区域
        size_of::<Self>() + self.num_memory_regions as usize * size_of::<HvMemoryRegion>()
    }

    pub fn iommu_units(&self) -> &[HvIommuInfo] {
        // 返回IOMMU单元信息的切片
        let mut n = 0;
        while n < HV_MAX_IOMMU_UNITS && self.platform_info.arch.iommu_units[n].base != 0 {
            n += 1;
        }
        &self.platform_info.arch.iommu_units[..n]
    }
    pub fn rmrr_ranges(&self) -> &[HvRmrrRange] {
        // 返回RMRR范围信息的切片
        let mut n = 0;
        while n < HV_MAX_RMRR_RANGE && self.platform_info.arch.rmrr_ranges[n].limit != 0 {
            n += 1;
        }
        &self.platform_info.arch.rmrr_ranges[..n]
    }

    pub fn mem_regions(&self) -> &[HvMemoryRegion] {
        // 返回内存区域信息的切片
        unsafe { slice::from_raw_parts(self.config_ptr(), self.num_memory_regions as usize) }
    }
}


## cell.rs

这段代码是一个用于虚拟机监控器（Hypervisor）的内存管理部分，主要包括内存区域的定义和初始化。代码中定义了一个`Cell`结构体，用于表示虚拟机的内存集，包括客体物理内存、宿主虚拟内存和DMA内存。以下是对代码的详细解释：

### 版权和许可证
```rust
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
```
这段代码开头是版权声明和许可证声明，声明了此代码的版权归Ant Group所有，并且用户需要遵守Apache License 2.0的条款。

### 引入库和模块
```rust
use crate::arch::{vmm::IoPageTable, HostPageTable, NestedPageTable};
use crate::config::HvSystemConfig;
use crate::consts::{HV_BASE, PER_CPU_SIZE};
use crate::error::HvResult;
use crate::header::HvHeader;
use crate::intervaltree::IntervalTree;
use crate::memory::addr::{phys_to_virt, GuestPhysAddr, HostPhysAddr, HostVirtAddr};
use crate::memory::cmr::NR_INIT_EPC_RANGES;
use crate::memory::{MemFlags, MemoryRegion, MemorySet};
```
这部分代码引入了多个模块和库，包括虚拟内存管理、系统配置、常量定义、错误处理、头文件、区间树、内存地址处理和内存管理相关的结构和方法。

### `Cell`结构体
```rust
#[derive(Debug)]
pub struct Cell {
    /// Guest physical memory set.
    pub gpm: MemorySet<NestedPageTable>,
    /// Host virtual memory set.
    pub hvm: MemorySet<HostPageTable>,
    /// DMA memory set.
    pub dma_regions: MemorySet<IoPageTable>,
    /// Normal world region which can be accessed by hypervisor.
    normal_world_mem_region: IntervalTree,
}
```
`Cell`结构体定义了四个字段：
- `gpm`：客体物理内存集，使用嵌套页表（NestedPageTable）。
- `hvm`：宿主虚拟内存集，使用宿主页表（HostPageTable）。
- `dma_regions`：DMA内存集，使用IO页表（IoPageTable）。
- `normal_world_mem_region`：正常世界的内存区域，可以被虚拟机监控器访问，使用区间树（IntervalTree）进行管理。

### `Cell`结构体的方法
#### `new_root`
```rust
impl Cell {
    fn new_root() -> HvResult<Self> {
        let header = HvHeader::get();
        let sys_config = HvSystemConfig::get();

        let hv_phys_start = sys_config.hypervisor_memory.phys_start as usize;
        let hv_phys_size = sys_config.hypervisor_memory.size as usize;
        let mut gpm = MemorySet::new();
        let mut hvm = MemorySet::new();
        let mut dma_regions = MemorySet::new();
        let mut normal_world_mem_region = IntervalTree::new();
```
- 获取虚拟机监控器头信息和系统配置。
- 初始化虚拟机监控器物理内存的起始地址和大小。
- 初始化三个内存集合：`gpm`、`hvm`和`dma_regions`，以及区间树`normal_world_mem_region`。

#### 初始化客体物理内存集（`gpm`）
```rust
        gpm.insert(MemoryRegion::new_with_empty_mapper(
            hv_phys_start,
            hv_phys_size,
            MemFlags::READ | MemFlags::ENCRYPTED,
        ))?;
```
- 将虚拟机监控器内存映射为空页，设置为只读和加密标志。

#### 初始化EPC内存区域
```rust
        for epc_range in &header.init_epc_ranges[..*NR_INIT_EPC_RANGES] {
            let epc_start_hpa = epc_range.start as HostPhysAddr;
            let epc_size = epc_range.size;
            gpm.insert(MemoryRegion::new_with_empty_mapper(
                epc_start_hpa,
                epc_size,
                MemFlags::READ | MemFlags::ENCRYPTED,
            ))?;
        }
```
- 将EPC（可扩展保护内存）区域映射为空页，设置为只读和加密标志。

#### 初始化宿主虚拟内存集（`hvm`）
```rust
        let core_and_percpu_size =
            header.core_size as usize + header.max_cpus as usize * PER_CPU_SIZE;
        hvm.insert(MemoryRegion::new_with_offset_mapper(
            HV_BASE,
            hv_phys_start,
            header.core_size,
            MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE | MemFlags::ENCRYPTED,
        ))?;
        hvm.insert(MemoryRegion::new_with_offset_mapper(
            HV_BASE + core_and_percpu_size,
            hv_phys_start + core_and_percpu_size,
            hv_phys_size - core_and_percpu_size,
            MemFlags::READ | MemFlags::WRITE | MemFlags::ENCRYPTED,
        ))?;
```
- 映射虚拟机监控器核心内存和配置内存，设置为读写、执行和加密标志。

#### 初始化DMA内存区域和IOMMU
```rust
        for region in sys_config.mem_regions() {
            let r = MemoryRegion::new_with_offset_mapper(
                region.virt_start as GuestPhysAddr,
                region.phys_start as HostPhysAddr,
                region.size as usize,
                region.flags - MemFlags::ENCRYPTED,
            );
            if region.flags.contains(MemFlags::DMA) {
                dma_regions.insert(r.clone())?;
            }
            gpm.insert(r)?;
        }

        for iommu in sys_config.iommu_units() {
            let paddr = iommu.base as HostPhysAddr;
            hvm.insert(MemoryRegion::new_with_offset_mapper(
                phys_to_virt(paddr),
                paddr,
                iommu.size as usize,
                MemFlags::READ | MemFlags::WRITE,
            ))?;
        }
```
- 初始化DMA内存区域和IOMMU（输入输出内存管理单元）区域。

### `is_valid_normal_world_gpaddr`
```rust
    pub fn is_valid_normal_world_gpaddr(&self, gpaddr: GuestPhysAddr) -> bool {
        self.normal_world_mem_region.contains(&gpaddr)
    }
}
```
- 判断一个给定的客体物理地址是否在正常世界的内存区域内。

### 全局静态变量`ROOT_CELL`和初始化函数`init`
```rust
lazy_static! {
    pub static ref ROOT_CELL: Cell = Cell::new_root().unwrap();
}

pub fn init() -> HvResult {
    crate::arch::vmm::check_hypervisor_feature()?;
    lazy_static::initialize(&ROOT_CELL);
    Ok(())
}
```
- 使用`lazy_static`宏定义了一个全局静态变量`ROOT_CELL`，它会调用`Cell::new_root()`进行初始化。
- `init`函数检查虚拟机监控器特性并初始化`ROOT_CELL`。

### 总结
这段代码实现了一个虚拟机监控器的内存管理系统，通过定义和初始化内存区域，确保虚拟机监控器可以正确管理和访问不同的内存区域。这些区域包括客体物理内存、宿主虚拟内存、DMA内存和IOMMU区域。通过这些管理，虚拟机监控器可以在不同的虚拟机之间进行内存隔离和管理。
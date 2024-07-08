use core::fmt;
use crate::memory::PagingResult;
use crate::memory::{GenericPTE, MemFlags, PageTableLevel, PagingInstr, PhysAddr, VirtAddr};
use crate::memory::{Level4PageTable, Level4PageTableImmut, Level4PageTableUnlocked};


bitflags::bitflags! {
    /// Memory attribute fields in the VMSAv8-64 translation table format descriptors.
    pub struct DescriptorAttr: u64 {
        // Attribute fields in stage 1 VMSAv8-64 Block and Page descriptors:

        /// Whether the descriptor is valid.
        const VALID =       1 << 0;
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   1 << 1;
        /// Memory attributes index field.
        const ATTR_INDX =   0b111 << 2;
        /// Non-secure bit. For memory accesses from Secure state, specifies whether the output
        /// address is in Secure or Non-secure memory.
        const NS =          1 << 5;
        /// Access permission: accessable at EL0.
        const AP_EL0 =      1 << 6;
        /// Access permission: read-only.
        const AP_RO =       1 << 7;
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER =       1 << 8;
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   1 << 9;
        /// The Access flag.
        const AF =          1 << 10;
        /// The not global bit.
        const NG =          1 << 11;
        /// Indicates that 16 adjacent translation table entries point to contiguous memory regions.
        const CONTIGUOUS =  1 <<  52;
        /// The Privileged execute-never field.
        const PXN =         1 <<  53;
        /// The Execute-never or Unprivileged execute-never field.
        const UXN =         1 <<  54;

        // Next-level attributes in stage 1 VMSAv8-64 Table descriptors:

        /// PXN limit for subsequent levels of lookup.
        const PXN_TABLE =           1 << 59;
        /// XN limit for subsequent levels of lookup.
        const XN_TABLE =            1 << 60;
        /// Access permissions limit for subsequent levels of lookup: access at EL0 not permitted.
        const AP_NO_EL0_TABLE =     1 << 61;
        /// Access permissions limit for subsequent levels of lookup: write access not permitted.
        const AP_NO_WRITE_TABLE =   1 << 62;
        /// For memory accesses from Secure state, specifies the Security state for subsequent
        /// levels of lookup.
        const NS_TABLE =            1 << 63;
    }
}


#[repr(u64)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MemType {
    Device = 0,
    Normal = 1,
}

impl DescriptorAttr {
    const ATTR_INDEX_MASK: u64 = 0b111_00;

    const fn from_mem_type(mem_type: MemType) -> Self {
        let mut bits = (mem_type as u64) << 2;
        if matches!(mem_type, MemType::Normal) {
            bits |= Self::INNER.bits() | Self::SHAREABLE.bits();
        }
        Self::from_bits_truncate(bits)
    }

    fn mem_type(&self) -> MemType {
        let idx = (self.bits() & Self::ATTR_INDEX_MASK) >> 2;
        match idx {
            0 => MemType::Device,
            1 => MemType::Normal,
            _ => panic!("Invalid memory attribute index"),
        }
    }
}


//
impl From<DescriptorAttr> for MemFlags {
    fn from(attr: DescriptorAttr) -> Self {
        let mem_flags = MemFlags::empty();

        // TODO 处理 MemFlags 与 DescriptorAttr 的关系
        if attr.contains(DescriptorAttr::AF) {
            mem_flags |= MemFlags::READ;  // 假设AF对应READ
        }
        if attr.contains(DescriptorAttr::AP_RO) {
            mem_flags |= MemFlags::WRITE;  // 假设AP_RO对应WRITE
        }
        if attr.contains(DescriptorAttr::UXN) {
            mem_flags |= MemFlags::EXECUTE;  // 假设UXN对应EXECUTE
        }
        if attr.contains(DescriptorAttr::NS) {
            mem_flags |= MemFlags::DMA;  // 假设NS对应DMA
        }
        if attr.contains(DescriptorAttr::SHAREABLE) {
            mem_flags |= MemFlags::IO;  // 假设SHAREABLE对应IO
        }
        if attr.contains(DescriptorAttr::INNER) {
            mem_flags |= MemFlags::COMM_REGION;  // 假设INNER对应COMM_REGION
        }
        if attr.contains(DescriptorAttr::CONTIGUOUS) {
            mem_flags |= MemFlags::NO_HUGEPAGES;  // 假设CONTIGUOUS对应NO_HUGEPAGES
        }
        if attr.contains(DescriptorAttr::AP_EL0) {
            mem_flags |= MemFlags::USER;  // 假设AP_EL0对应USER
        }
        if attr.contains(DescriptorAttr::PXN) {
            mem_flags |= MemFlags::ENCRYPTED;  // 假设PXN对应ENCRYPTED
        }
        if attr.contains(DescriptorAttr::VALID) {
            mem_flags |= MemFlags::NO_PRESENT;  // 假设VALID对应NO_PRESENT
        }

        mem_flags
    }
}







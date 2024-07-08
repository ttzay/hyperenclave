use core::fmt;
use crate::memory::PagingResult;
use crate::memory::{GenericPTE, MemFlags, PageTableLevel, PagingInstr, PhysAddr, VirtAddr};
use crate::memory::{Level4PageTable, Level4PageTableImmut, Level4PageTableUnlocked};

// TODO finish stage-2 translation

bitflags::bitflags! {
    /// BLOCK Entry
    ///        63              50 49  48 47                   n n-1  16 15       12 11                    2  1   0
    ///   Upper block attributes | RES0 | Output address[47:n] | RES0 | nT | RES0 | Lower block attributes | 0 | 1

    /// Table entry
    ///     63      62    61    60         59     58     52 51  48  47                           m  m-1 12  11     2   1    0
    ///   NSTable | APTable | XNTable | PXNTable | IGNORED | RES0 | Next-level table address[47:m] | RES0 | IGNORED |  1  | 1



    /// Memory attribute fields in the VMSAv8-64 translation table format descriptors.
    pub struct DescriptorAttr: u64 {
        // Attribute fields in stage 2 VMSAv8-64 Block and Page descriptors:

        /// Whether the descriptor is valid.
        const VALID =       1 << 0;
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   1 << 1;
        /// Memory attributes index field.
        const ATTR      =   0b1111 << 2;
        /// Access permission: accessable at EL0/1, Read / Write.
        const S2AP_R      =   1 << 6;
        /// Access permission: accessable at EL0/1, Write.
        const S2AP_W      =   1 << 7;
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER     =   1 << 8;
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   1 << 9;
        /// The Access flag.
        const AF =          1 << 10;
        /// Indicates that 16 adjacent translation table entries point to contiguous memory regions.
        const CONTIGUOUS =  1 <<  52;
        /// The execute-never field.
        const XN_0 =        1 << 53;
        const XN_1 =        1 << 54;

        // Next-level attributes in stage 2 VMSAv8-64 Table descriptors:(TODO)

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
#[derive(Debug,Clone,Copy,Eq,PartialEq)]
enum MemType {
    Device = 1,
    Normal = 15,
}


impl DescriptorAttr {
    const ATTR_INDEX_MASK:u64  =0b1111_00;


    const fn from_mem_type(mem_type: MemType) -> Self {
        let mut bits = (mem_type as u64) << 2;
        if match  {  }
    }



}





use core::fmt;
use crate::memory::PagingResult;
use crate::memory::{GenericPTE, MemFlags, PageTableLevel, PagingInstr, PhysAddr, VirtAddr};
use crate::memory::{Level4PageTable, Level4PageTableImmut, Level4PageTableUnlocked};
use crate::memory::PAGE_SIZE;


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

impl From<DescriptorAttr> for MemFlags {
    // GPT  对比 MemFlags 和 DescriptorAttr
    // MemFlags	DescriptorAttr	说明
    // READ         隐含在 VALID 和 AP	DescriptorAttr 中没有直接的 READ 标志，但有效的页通常可读，这由 VALID 和访问权限决定。
    // WRITE	    AP_RO（反向）	    MemFlags 中的 WRITE 与 DescriptorAttr 中的 AP_RO 相反，后者用于标记只读。
    // EXECUTE	    UXN/PXN（反向）	    MemFlags 中的 EXECUTE 与 DescriptorAttr 中的 UXN 和 PXN 相反，后者用于禁止执行。
    // DMA	        无直接对应	        DMA 用于标记页面是否参与直接内存访问，DescriptorAttr 中无直接标志，但可通过缓存策略影响。
    // IO	        无直接对应	        用于标记页面参与 I/O 操作，通常与设备映射有关。
    // COMM_REGION	无直接对应	        用于标记内存用于通信，可能与共享属性有关。
    // NO_HUGEPAGES	NON_BLOCK（反向）	NO_HUGEPAGES 防止使用大页面，NON_BLOCK 标记页面而不是块。
    // USER	        AP_EL0	            MemFlags的 USER标志类似于 DescriptorAttr 的 AP_EL0，用于用户模式的访问控制。
    // ENCRYPTED	无直接对应	        ENCRYPTED 用于指示页面加密，DescriptorAttr 中没有直接等效的标志。
    // NO_PRESENT	VALID（反向）	    NO_PRESENT 指示页面不在内存中，VALID 标志用于检查页面是否有效。
    // 详细比较说明
    // 读权限：在 DescriptorAttr 中，页面是否可读主要由页面是否有效（VALID）以及是否具有合适的访问权限（AP）来决定，而 MemFlags 中有显式的 READ 标志。
    // 写权限：DescriptorAttr 使用 AP_RO 标志来指定只读，若未设置该标志则页面可写，而 MemFlags 使用 WRITE 来明确指定页面是否可写。
    // 执行权限：DescriptorAttr 使用 UXN（用户模式不可执行）和 PXN（特权模式不可执行）来控制执行权限，而 MemFlags 中有直接的 EXECUTE 标志用于允许执行。
    // 用户访问：DescriptorAttr 中的 AP_EL0 控制页面是否可以被用户模式访问，这与 MemFlags 中的 USER 类似，都是为了限制页面的访问级别
    // 大页面控制：MemFlags 的 NO_HUGEPAGES 标志用于防止使用大页面，而 DescriptorAttr 的 NON_BLOCK 用于指示非块（即小页面），功能上有些重叠。
    // 其他特性：MemFlags 中的一些标志（如 DMA、IO、COMM_REGION、ENCRYPTED）主要用于软件层次的内存管理，没有直接对应的硬件层面标志。DescriptorAttr 专注于硬件层面，主要管理内存页面的属性和安全性。
    fn from(attr: DescriptorAttr) -> Self {
        let mut flags = Self::empty();
        if !attr.contains(DescriptorAttr::VALID) {
            flags |= Self::NO_PRESENT;
        } else {
            flags |= Self::READ;
            if !attr.contains(DescriptorAttr::AP_RO) {
                flags |= Self::WRITE;
            }
            if attr.contains(DescriptorAttr::AP_EL0) {
                flags |= Self::USER;
                if !attr.contains(DescriptorAttr::UXN) {
                    flags |= Self::EXECUTE;
                }
            } else if !attr.intersects(DescriptorAttr::PXN) {
                flags |= Self::EXECUTE;
            }
        }
        flags
    }
}

pub struct PTEntry(u64);

// PAGE_SIZE which could change
const PHYS_ADDR_MASK: usize = 0xffff_ffff_ffff & !(PAGE_SIZE - 1); //


impl GenericPTE for PTEntry {
    /// Returns the physical address mapped by this entry.
    fn addr(&self) -> PhysAddr {
        (self.0 & PHYS_ADDR_MASK) as PhysAddr
    }
    /// Returns the flags of this entry.
    fn flags(&self) -> MemFlags {
        /// Returns whether this entry is zero.
        //PTF::from_bits_truncate(self.0).into()
        DescriptorAttr::from_bits_truncate(self.0).into()
    }
    /// Returns whether this entry is zero.
    fn is_unused(&self) -> bool{
        self.0  ==  0
    }
    /// Returns whether this entry flag indicates present.
    fn is_present(&self) -> bool{
        self.0 & DescriptorAttr::VALID.bits() != 0
    }
    /// Returns whether this entry maps to a huge frame (terminate page translation).
    fn is_leaf(&self) -> bool {
        !DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::NON_BLOCK)
    }
    /// Returns whether this entry's ACCESSED bit is set.
    fn is_young(&self) -> bool{
        self.0 & DescriptorAttr::AF.bits() != 0        
    }

    /// Mark the PTE as non-ACCESSED.
    fn set_old(&mut self){
        let flags = !DescriptorAttr::AF;
        self.0 &= flags.bits() | PHYS_ADDR_MASK as u64;
    }
    /// Set physical address for terminal entries.
    fn set_addr(&mut self, paddr: PhysAddr){
        // TODO check this
        self.0 = (self.0 & !PHYS_ADDR_MASK as u64) | (paddr as u64 & PHYS_ADDR_MASK as u64);
    }
    /// Set flags for terminal entries.
    fn set_flags(&mut self, flags: MemFlags, is_huge: bool) -> PagingResult{
        // TODO check this
        let mut attr = DescriptorAttr::from(flags);
        if is_huge {
            attr.remove(DescriptorAttr::NON_BLOCK);
        } else {
            attr.insert(DescriptorAttr::NON_BLOCK);
        }
        self.0 = attr.bits() | (self.0 & PHYS_ADDR_MASK as u64);
        Ok(())
    }
    /// Set physical address and flags for intermediate entry,
    /// `is_present` controls whether to setting its P bit.
    fn set_table(
        &mut self,
        paddr: PhysAddr,
        next_level: PageTableLevel,
        is_present: bool,
    ) -> PagingResult{
        // TODO
    }
    /// Mark the intermediate or terminal entry as present (or valid), its other parts remain unchanged.
    fn set_present(&mut self) -> PagingResult{
        // TODO
    }
    /// Mark the intermediate or terminal entry as non-present (or invalid), its other parts remain unchanged.
    fn set_notpresent(&mut self) -> PagingResult{
        // TODO
    }
    /// Set this entry to zero.
    fn clear(&mut self){
        // TODO
    }
}


impl fmt::Debug for PTEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Stage1PageTableEntry")
            .field("raw", &self.0)
            .field("paddr", &self.addr())
            .field("attr", &DescriptorAttr::from_bits_truncate(self.0))
            .field("flags", &self.pt_flags())
            .field("memory_type", &self.memory_type())
            .finish()
    }
}

pub struct S1PTInstr;

impl PagingInstr for S1PTInstr {
    unsafe fn activate(root_paddr: PhysAddr){
        // TODO need to check this
        // why this need EL2
        TTBR0_EL2.set(root_paddr as _);
        core::arch::asm!("isb");
        core::arch::asm!("tlbi alle2");
        core::arch::asm!("dsb nsh");

    }
    fn flush(vaddr: Option<VirtAddr>){
        // do nothing
    }

}

pub type PageTable = Level4PageTable<VirtAddr, PTEntry, S1PTInstr>;
pub type PageTableImmut = Level4PageTableImmut<VirtAddr, PTEntry>;
pub type EnclaveGuestPageTableUnlocked = Level4PageTableUnlocked<VirtAddr, PTEntry, S1PTInstr>;






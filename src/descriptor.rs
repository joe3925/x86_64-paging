use crate::address::Granule4KiB;
use crate::attrs::PageTableFlags;
use core::marker::PhantomData;
use core::ptr::{read_volatile, write_volatile};
use paging::address::{Level, PhysAddr};
use paging::descriptor::{
    DescriptorError, DescriptorFormat, DescriptorKind, DescriptorLayout, SupportsLiveDescriptorIo,
};
use paging::table::{TableAddr, TableTransition};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Amd64;

unsafe impl DescriptorFormat for Amd64 {
    type Raw = u64;
    const DESCRIPTOR_BYTES: usize = 8;
    const DESCRIPTOR_SHIFT: u8 = 3;
    const MAX_INPUT_ADDRESS_BITS: u8 = 57;
    const OUTPUT_ADDRESS_BITS: u8 = 52;
    const SUPPORTED_OUTPUT_ADDRESS_BITS: &'static [u8] = &[
        32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52,
    ];
    const EXTENDED_LOWEST_ROOT_LEVEL: Level = Level::NEG1;
    const FINAL_LEVEL: Level = Level::L3;
    fn invalid() -> Self::Raw {
        0
    }
    unsafe fn read_descriptor(ptr: *const Self::Raw) -> Self::Raw {
        unsafe { read_volatile(ptr) }
    }
    unsafe fn write_descriptor(ptr: *mut Self::Raw, raw: Self::Raw) {
        unsafe { write_volatile(ptr, raw) }
    }
}

unsafe impl SupportsLiveDescriptorIo for Amd64 {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Amd64Layout<R>(PhantomData<R>);

unsafe impl<R: Copy + 'static> DescriptorLayout<Granule4KiB> for Amd64Layout<R> {
    type Format = Amd64;
    type LeafFields = PageTableFlags;
    type TableFields = PageTableFlags;
    type Error = DescriptorError;
    const ADDRESS_FIELD_MASK: u128 = 0x000f_ffff_ffff_f000;

    fn supports_leaf_level(level: Level) -> bool {
        level == Level::L1 || level == Level::L2 || level == Level::L3
    }

    fn kind(raw: u64, level: Level) -> DescriptorKind {
        if raw & PageTableFlags::PRESENT.bits() == 0 {
            DescriptorKind::Invalid
        } else if level == Level::L3 {
            DescriptorKind::Page
        } else if (level == Level::L1 || level == Level::L2)
            && raw & PageTableFlags::HUGE_PAGE.bits() != 0
        {
            DescriptorKind::Block
        } else {
            DescriptorKind::Table
        }
    }

    fn decode_leaf_fields(raw: u64, level: Level) -> Self::LeafFields {
        let mut flags = raw & !Self::ADDRESS_FIELD_MASK as u64;
        if level == Level::L3 {
            if flags & PageTableFlags::HUGE_PAGE.bits() != 0 {
                flags &= !PageTableFlags::HUGE_PAGE.bits();
                flags |= PageTableFlags::PAT_HUGE.bits();
            }
        } else if raw & PageTableFlags::PAT_HUGE.bits() != 0 {
            flags |= PageTableFlags::PAT_HUGE.bits();
        }
        PageTableFlags::from_bits_retain(flags)
    }

    fn decode_table_fields(raw: u64, _: Level) -> Self::TableFields {
        PageTableFlags::from_bits_retain(raw & !Self::ADDRESS_FIELD_MASK as u64)
    }

    fn leaf_descriptor(
        output: PhysAddr,
        level: Level,
        fields: Self::LeafFields,
    ) -> Result<u64, DescriptorError> {
        if !Self::supports_leaf_level(level) {
            return Err(DescriptorError::InvalidLeafLevel { level });
        }
        let alignment = if level == Level::L1 {
            1u64 << 30
        } else if level == Level::L2 {
            1u64 << 21
        } else {
            1u64 << 12
        };
        if output.0 & (alignment - 1) != 0 || output.0 & !(Self::ADDRESS_FIELD_MASK as u64) != 0 {
            return Err(DescriptorError::InvalidReservedBitState);
        }
        let leaf_allowed = 0x1fff | (0xfffu64 << 52);
        if fields.bits() & !leaf_allowed != 0 {
            return Err(DescriptorError::InvalidReservedBitState);
        }
        let mut flags = fields.bits();
        if level == Level::L3 {
            if flags & PageTableFlags::PAT_HUGE.bits() != 0 {
                flags &= !PageTableFlags::PAT_HUGE.bits();
                flags |= PageTableFlags::HUGE_PAGE.bits();
            }
        } else {
            flags |= PageTableFlags::HUGE_PAGE.bits();
        }
        Ok(output.0 | flags)
    }

    fn table_descriptor(
        table: TableAddr<Granule4KiB>,
        transition: TableTransition<Amd64, Granule4KiB>,
        fields: Self::TableFields,
    ) -> Result<u64, DescriptorError> {
        if transition.level_step() != 1 || transition.child().stride_count().raw() != 1 {
            return Err(DescriptorError::InvalidTableTransition {
                parent_level: transition.parent_level(),
                child_level: transition.child_level(),
                stride_count: transition.child().stride_count().raw(),
            });
        }
        if table.raw() & !(Self::ADDRESS_FIELD_MASK as u64) != 0 {
            return Err(DescriptorError::InvalidReservedBitState);
        }
        let table_allowed = 0x0e3f | (0x7fu64 << 52) | (1u64 << 63);
        if fields.bits() & !table_allowed != 0 {
            return Err(DescriptorError::InvalidReservedBitState);
        }
        Ok(table.raw() | (fields.bits() & !PageTableFlags::HUGE_PAGE.bits()))
    }

    fn output_address(raw: u64, level: Level) -> PhysAddr {
        let mask = if level == Level::L1 {
            !((1u64 << 30) - 1)
        } else if level == Level::L2 {
            !((1u64 << 21) - 1)
        } else {
            !((1u64 << 12) - 1)
        };
        PhysAddr(raw & Self::ADDRESS_FIELD_MASK as u64 & mask)
    }

    fn table_address(raw: u64, _: Level) -> TableAddr<Granule4KiB> {
        unsafe { TableAddr::new_unchecked(raw & Self::ADDRESS_FIELD_MASK as u64) }
    }
}

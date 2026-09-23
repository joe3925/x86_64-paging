use crate::address::Granule4KiB;
use crate::attrs::{AttributeError, LeafAttrs, MemoryType, PageTableFlags, TableAttrs};
use crate::descriptor::{Amd64, Amd64Layout};
use paging::address::Level;
use paging::attrs::AttributeCodec;
use paging::regime::{SemanticRegime, TranslationRegime};

pub trait LongMode: Copy + 'static {
    const ADDRESS_BITS: u8;
    const ROOT_LEVEL: Level;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LongMode4Level;

impl LongMode for LongMode4Level {
    const ADDRESS_BITS: u8 = 48;
    const ROOT_LEVEL: Level = Level::L0;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LongMode5Level;

impl LongMode for LongMode5Level {
    const ADDRESS_BITS: u8 = 57;
    const ROOT_LEVEL: Level = Level::NEG1;
}

macro_rules! regime {
    ($regime:ty) => {
        impl TranslationRegime<Amd64, Granule4KiB> for $regime {
            type Layout = Amd64Layout<$regime>;
        }

        impl SemanticRegime<Amd64> for $regime {
            type Leaf = LeafAttrs;
            type Table = TableAttrs;
            type Codec = Amd64AttributeCodec;
        }
    };
}

regime!(LongMode4Level);
regime!(LongMode5Level);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Amd64AttributeCodec;

impl<R, Cfg> AttributeCodec<Amd64, R, Granule4KiB, Cfg> for Amd64AttributeCodec
where
    R: LongMode
        + TranslationRegime<Amd64, Granule4KiB, Layout = Amd64Layout<R>>
        + SemanticRegime<Amd64, Leaf = LeafAttrs, Table = TableAttrs>,
{
    type Error = AttributeError;

    fn encode_leaf(_: &Cfg, _: Level, attrs: LeafAttrs) -> Result<PageTableFlags, Self::Error> {
        if attrs.protection_key > 15 {
            return Err(AttributeError::InvalidProtectionKey {
                key: attrs.protection_key,
            });
        }
        if attrs.software > 0x3ff {
            return Err(AttributeError::InvalidSoftwareBits {
                value: attrs.software,
            });
        }
        let pat = match attrs.memory_type {
            MemoryType::WriteBack => 0,
            MemoryType::WriteThrough => 1,
            MemoryType::Uncacheable => 3,
            MemoryType::WriteCombining => 4,
            MemoryType::WriteProtected => 5,
            MemoryType::Uncached => 7,
            MemoryType::Pat(index) if index < 8 => index,
            MemoryType::Pat(index) => return Err(AttributeError::InvalidPatIndex { index }),
        };
        let mut bits = 0u64;
        bits |= attrs.present as u64;
        bits |= (attrs.writable as u64) << 1;
        bits |= (attrs.user_accessible as u64) << 2;
        bits |= ((pat as u64) & 1) << 3;
        bits |= (((pat as u64) >> 1) & 1) << 4;
        bits |= (attrs.accessed as u64) << 5;
        bits |= (attrs.dirty as u64) << 6;
        bits |= (attrs.global as u64) << 8;
        bits |= ((attrs.software as u64) & 0x7) << 9;
        bits |= (((pat as u64) >> 2) & 1) << 12;
        bits |= (((attrs.software as u64) >> 3) & 0x7f) << 52;
        bits |= (attrs.protection_key as u64) << 59;
        bits |= (!attrs.executable as u64) << 63;
        Ok(PageTableFlags::from_bits_retain(bits))
    }

    fn encode_table(_: &Cfg, _: Level, attrs: TableAttrs) -> Result<PageTableFlags, Self::Error> {
        if attrs.software > 0x3ff {
            return Err(AttributeError::InvalidSoftwareBits {
                value: attrs.software,
            });
        }
        let mut bits = 0u64;
        bits |= attrs.present as u64;
        bits |= (attrs.writable as u64) << 1;
        bits |= (attrs.user_accessible as u64) << 2;
        bits |= (attrs.write_through as u64) << 3;
        bits |= (attrs.cache_disable as u64) << 4;
        bits |= (attrs.accessed as u64) << 5;
        bits |= ((attrs.software as u64) & 0x7) << 9;
        bits |= (((attrs.software as u64) >> 3) & 0x7f) << 52;
        bits |= (!attrs.executable as u64) << 63;
        Ok(PageTableFlags::from_bits_retain(bits))
    }

    fn decode_leaf(_: &Cfg, _: Level, raw: PageTableFlags) -> Result<LeafAttrs, Self::Error> {
        let bits = raw.bits();
        let pat = (((bits >> 3) & 1) | (((bits >> 4) & 1) << 1) | (((bits >> 12) & 1) << 2)) as u8;
        Ok(LeafAttrs {
            present: bits & (1 << 0) != 0,
            writable: bits & (1 << 1) != 0,
            user_accessible: bits & (1 << 2) != 0,
            accessed: bits & (1 << 5) != 0,
            dirty: bits & (1 << 6) != 0,
            global: bits & (1 << 8) != 0,
            executable: bits & (1 << 63) == 0,
            memory_type: MemoryType::Pat(pat),
            protection_key: ((bits >> 59) & 0xf) as u8,
            software: (((bits >> 9) & 0x7) | (((bits >> 52) & 0x7f) << 3)) as u16,
        })
    }

    fn decode_table(_: &Cfg, _: Level, raw: PageTableFlags) -> Result<TableAttrs, Self::Error> {
        let bits = raw.bits();
        Ok(TableAttrs {
            present: bits & (1 << 0) != 0,
            writable: bits & (1 << 1) != 0,
            user_accessible: bits & (1 << 2) != 0,
            write_through: bits & (1 << 3) != 0,
            cache_disable: bits & (1 << 4) != 0,
            accessed: bits & (1 << 5) != 0,
            executable: bits & (1 << 63) == 0,
            software: (((bits >> 9) & 0x7) | (((bits >> 52) & 0x7f) << 3)) as u16,
        })
    }
}

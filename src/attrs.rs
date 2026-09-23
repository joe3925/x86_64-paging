use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct PageTableFlags(u64);

impl PageTableFlags {
    pub const EMPTY: Self = Self(0);
    pub const PRESENT: Self = Self(1 << 0);
    pub const WRITABLE: Self = Self(1 << 1);
    pub const USER_ACCESSIBLE: Self = Self(1 << 2);
    pub const WRITE_THROUGH: Self = Self(1 << 3);
    pub const CACHE_DISABLE: Self = Self(1 << 4);
    pub const ACCESSED: Self = Self(1 << 5);
    pub const DIRTY: Self = Self(1 << 6);
    pub const HUGE_PAGE: Self = Self(1 << 7);
    pub const GLOBAL: Self = Self(1 << 8);
    pub const SOFTWARE_LOW: Self = Self(0b111 << 9);
    pub const PAT_HUGE: Self = Self(1 << 12);
    pub const SOFTWARE_HIGH: Self = Self(0x7f << 52);
    pub const PROTECTION_KEY: Self = Self(0xf << 59);
    pub const NO_EXECUTE: Self = Self(1 << 63);

    pub const fn from_bits_retain(bits: u64) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u64 {
        self.0
    }
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
    pub const fn with(self, other: Self, enabled: bool) -> Self {
        if enabled {
            Self(self.0 | other.0)
        } else {
            Self(self.0 & !other.0)
        }
    }
}

impl BitOr for PageTableFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for PageTableFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for PageTableFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for PageTableFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for PageTableFlags {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryType {
    WriteBack,
    WriteThrough,
    Uncacheable,
    WriteCombining,
    WriteProtected,
    Uncached,
    Pat(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeafAttrs {
    pub present: bool,
    pub writable: bool,
    pub user_accessible: bool,
    pub accessed: bool,
    pub dirty: bool,
    pub global: bool,
    pub executable: bool,
    pub memory_type: MemoryType,
    pub protection_key: u8,
    pub software: u16,
}

impl Default for LeafAttrs {
    fn default() -> Self {
        Self {
            present: true,
            writable: false,
            user_accessible: false,
            accessed: false,
            dirty: false,
            global: false,
            executable: true,
            memory_type: MemoryType::WriteBack,
            protection_key: 0,
            software: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableAttrs {
    pub present: bool,
    pub writable: bool,
    pub user_accessible: bool,
    pub write_through: bool,
    pub cache_disable: bool,
    pub accessed: bool,
    pub executable: bool,
    pub software: u16,
}

impl Default for TableAttrs {
    fn default() -> Self {
        Self {
            present: true,
            writable: true,
            user_accessible: false,
            write_through: false,
            cache_disable: false,
            accessed: false,
            executable: true,
            software: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributeError {
    InvalidProtectionKey { key: u8 },
    InvalidSoftwareBits { value: u16 },
    InvalidPatIndex { index: u8 },
}

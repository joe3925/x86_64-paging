pub use paging::address::{GranuleError, Level, PhysAddr, TranslationGranule, VirtAddr};
pub use paging::translation::walk::WalkInputAddr;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Granule4KiB;

unsafe impl TranslationGranule for Granule4KiB {
    const SHIFT: u8 = 12;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalAddressError {
    InvalidAddressBits { addr_bits: u8 },
    NotCanonical { addr: u64, addr_bits: u8 },
}

pub const fn from_canonical(
    raw: u64,
    addr_bits: u8,
) -> Result<WalkInputAddr, CanonicalAddressError> {
    if addr_bits == 0 || addr_bits > 64 {
        return Err(CanonicalAddressError::InvalidAddressBits { addr_bits });
    }
    if addr_bits == 64 {
        return Ok(WalkInputAddr::new(raw));
    }
    let mask = (1u64 << addr_bits) - 1;
    let upper = raw & !mask;
    let expected_upper = if raw & (1u64 << (addr_bits - 1)) != 0 {
        !mask
    } else {
        0
    };
    if upper != expected_upper {
        return Err(CanonicalAddressError::NotCanonical {
            addr: raw,
            addr_bits,
        });
    }
    Ok(WalkInputAddr::new(raw & mask))
}

pub const fn canonicalize(raw: u64, addr_bits: u8) -> Result<u64, CanonicalAddressError> {
    if addr_bits == 0 || addr_bits > 64 {
        return Err(CanonicalAddressError::InvalidAddressBits { addr_bits });
    }
    if addr_bits == 64 {
        return Ok(raw);
    }
    let mask = (1u64 << addr_bits) - 1;
    let value = raw & mask;
    if value & (1u64 << (addr_bits - 1)) != 0 {
        Ok(value | !mask)
    } else {
        Ok(value)
    }
}

pub use paging::mapper::*;

use paging::address::TranslationGranule;
use paging::descriptor::DescriptorFormat;
use paging::regime::TranslationRegime;
use paging::table::{TableAccessLocation, TableAddr, TableAllocLayout};

pub type OfflineMapper<F, R, G, A, P> =
    paging::mapper::Mapper<F, R, G, A, P, paging::mapper::Offline>;
pub type LiveMapper<F, R, G, A, P, I> =
    paging::mapper::Mapper<F, R, G, A, P, paging::mapper::Live<I>>;

#[allow(clippy::missing_safety_doc)]
pub unsafe trait TlbFlush {
    fn flush_all(&mut self);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FullFlushInvalidation<T> {
    flush: T,
    pending: bool,
}

impl<T> FullFlushInvalidation<T> {
    pub const fn new(flush: T) -> Self {
        Self {
            flush,
            pending: false,
        }
    }
    pub const fn flush(&self) -> &T {
        &self.flush
    }
    pub fn flush_mut(&mut self) -> &mut T {
        &mut self.flush
    }
    pub fn into_inner(self) -> T {
        self.flush
    }
}

unsafe impl<F, R, G, T> paging::mapper::MapperInvalidation<F, R, G> for FullFlushInvalidation<T>
where
    F: DescriptorFormat,
    R: TranslationRegime<F, G>,
    G: TranslationGranule,
    T: TlbFlush,
{
    fn leaf_inserted(&mut self, _: TableAccessLocation<F, G>, _: usize, _: F::Raw, _: F::Raw) {
        self.pending = true;
    }
    fn leaf_removed(&mut self, _: TableAccessLocation<F, G>, _: usize, _: F::Raw) {
        self.pending = true;
    }
    fn table_descriptor_inserted(
        &mut self,
        _: TableAccessLocation<F, G>,
        _: usize,
        _: F::Raw,
        _: F::Raw,
    ) {
        self.pending = true;
    }
    fn table_descriptor_removed(&mut self, _: TableAccessLocation<F, G>, _: usize, _: F::Raw) {
        self.pending = true;
    }
    fn before_table_frame_reclaim(&mut self, _: TableAddr<G>, _: TableAllocLayout) {
        self.pending = true;
    }
    fn synchronize(&mut self) {
        if self.pending {
            self.flush.flush_all();
            self.pending = false;
        }
    }
}

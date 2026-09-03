use core::marker::PhantomData;

use crate::OutputAddress;

pub trait SizeForFrame {
    const SIZE: u64;
}
pub struct Size4Kib {}
impl SizeForFrame for Size4Kib {
    const SIZE: u64 = 1024 * 4;
}
pub struct Size2Mib {}
impl SizeForFrame for Size2Mib {
    const SIZE: u64 = 1024 * 1024 * 2;
}
pub struct Size1Gib {}
impl SizeForFrame for Size1Gib {
    const SIZE: u64 = 1024 * 1024 * 1024;
}
pub struct PhysFrame<S: SizeForFrame> {
    pub _data: PhantomData<S>,
    pub frame_base: OutputAddress,
}

pub trait ZeroedFrameAllocator<S: SizeForFrame> {
    fn alloc_frame(&self, frame_size: dyn SizeForFrame) -> PhysFrame;
    fn free_frame(&self, frame_size: FrameSize) -> PhysFrame;
}

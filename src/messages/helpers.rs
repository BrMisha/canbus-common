pub trait CopyIntoSlice {
    fn copy_into_slice(&self, dst: &mut [u8]) -> Option<usize>;
}

impl CopyIntoSlice for bool {
    fn copy_into_slice(&self, dst: &mut [u8]) -> Option<usize> {
        *(dst.get_mut(0)?) = *self as u8;
        Some(1)
    }
}

use core::{
    marker::PhantomData,
    slice::{from_raw_parts, from_raw_parts_mut},
};

/// Adding this level of indirection improves generated code
///
/// In the SysV64 ABI structs of up to 2 pointer sizes can be returned in registers.
/// By marking the outer struct as transparent,
/// it is treated the same as this struct, which can be returned in registers
#[repr(C)]
struct FfiSliceMutData<T> {
    pub ptr: *mut T,
    pub len: usize,
}

#[repr(transparent)]
pub struct FfiSliceMut<'lt, T> {
    data: FfiSliceMutData<T>,
    _phantom_lt: PhantomData<&'lt [T]>,
}

impl<'lt, T> FfiSliceMut<'lt, T> {
    pub fn new(s: &'lt mut [T]) -> Self {
        let len = s.len();
        let ptr = s.as_mut_ptr();

        FfiSliceMut {
            data: FfiSliceMutData { len, ptr },
            _phantom_lt: Default::default(),
        }
    }

    #[inline(always)]
    pub fn into_slice(self) -> &'lt mut [T] {
        self.into()
    }

    #[inline(always)]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.data.ptr, self.data.len) }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        unsafe { from_raw_parts(self.data.ptr, self.data.len) }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.data.len
    }
}

impl<'lt, T> Into<&'lt mut [T]> for FfiSliceMut<'lt, T> {
    fn into(self) -> &'lt mut [T] {
        unsafe { from_raw_parts_mut(self.data.ptr, self.data.len) }
    }
}

impl<'lt, T> From<&'lt mut [T]> for FfiSliceMut<'lt, T> {
    fn from(s: &'lt mut [T]) -> Self {
        FfiSliceMut::new(s)
    }
}

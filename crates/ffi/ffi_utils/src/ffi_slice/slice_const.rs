use core::{marker::PhantomData, ptr::NonNull, slice::from_raw_parts};

/// Adding this level of indirection improves generated code
///
/// In the SysV64 ABI structs of up to 2 pointer sizes can be returned in registers.
/// By marking the outer struct as transparent,
/// it is treated the same as this struct, which can be returned in registers
#[repr(C)]
#[derive(Copy, Clone)]
struct FfiSliceData<T> {
    pub ptr: NonNull<T>,
    pub len: usize,
}

#[repr(transparent)]
pub struct FfiSlice<'lt, T> {
    data: FfiSliceData<T>,
    _phantom_lt: PhantomData<&'lt [T]>,
}

impl<'lt, T> FfiSlice<'lt, T> {
    pub fn new(s: &'lt [T]) -> Self {
        let len = s.len();
        let ptr = s.as_ptr();
        let ptr = unsafe { NonNull::new_unchecked(ptr as *mut T) };

        FfiSlice {
            data: FfiSliceData { len, ptr },
            _phantom_lt: Default::default(),
        }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            from_raw_parts(self.data.ptr.as_ref() as *const T, self.data.len)
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.data.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'lt, T> Into<&'lt [T]> for FfiSlice<'lt, T> {
    fn into(self) -> &'lt [T] {
        unsafe { from_raw_parts(self.data.ptr.as_ptr(), self.data.len) }
    }
}

impl<'lt, T> From<&'lt [T]> for FfiSlice<'lt, T> {
    fn from(s: &'lt [T]) -> Self {
        FfiSlice::new(s)
    }
}

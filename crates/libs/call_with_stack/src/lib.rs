#![no_std]

use core::{
    ffi::c_void,
    mem::{ManuallyDrop, MaybeUninit},
};

pub type RawCallFunctionSignature = unsafe extern "sysv64" fn(*mut c_void);
pub type RawJumpFunctionSignature = unsafe extern "sysv64" fn(*mut c_void) -> !;

extern "sysv64" {
    pub fn call_with_stack_raw(
        arg: *mut c_void,
        function: RawCallFunctionSignature,
        stack: *mut u8,
    );

    pub fn jump_with_stack_raw(
        arg: *mut c_void,
        function: RawJumpFunctionSignature,
        stack: *mut u8,
    ) -> !;
}

pub type CallFunctionSignature<T> = unsafe extern "sysv64" fn(*mut T);
pub type JumpFunctionSignature<T> = unsafe extern "sysv64" fn(*mut T) -> !;

pub unsafe fn call_with_stack<T>(
    arg: *mut T,
    function: CallFunctionSignature<T>,
    stack: *mut u8,
) {
    call_with_stack_raw(
        arg as *mut c_void,
        core::mem::transmute::<
            CallFunctionSignature<T>,
            RawCallFunctionSignature,
        >(function),
        stack,
    )
}

pub unsafe fn jump_with_stack<T>(
    arg: *mut T,
    function: JumpFunctionSignature<T>,
    stack: *mut u8,
) -> ! {
    jump_with_stack_raw(
        arg as *mut c_void,
        core::mem::transmute::<
            JumpFunctionSignature<T>,
            RawJumpFunctionSignature,
        >(function),
        stack,
    )
}

/// Calls a closure and returns the result
///
/// This function is unsafe because it changes the stack pointer to stack.
/// stack must be suitable to be used as a stack pointer on the target system.
pub unsafe fn call_closure_with_stack<F, R>(closure: F, stack: *mut u8) -> R
where
    F: FnOnce() -> R,
{
    unsafe extern "sysv64" fn inner<F, R>(
        data: *mut (ManuallyDrop<F>, MaybeUninit<R>),
    ) where
        F: FnOnce() -> R,
    {
        let data = &mut *data;

        let result = {
            // Read the closure from context, taking ownership of it
            let function = ManuallyDrop::take(&mut data.0);

            // Call the closure.
            // This consumes it and returns the result
            function()
        };

        // Write the result into the context
        data.1 = MaybeUninit::new(result);
    }

    // The context contains the closure and uninitialized memory for the return value
    let mut context = (ManuallyDrop::new(closure), MaybeUninit::uninit());

    call_with_stack(
        &mut context as *mut _,
        // We create a new, internal function that does not close over anything
        // and takes a context reference as its argument
        inner,
        stack,
    );

    // Read the result from the context
    // No values are in the context anymore afterwards
    context.1.assume_init()
}

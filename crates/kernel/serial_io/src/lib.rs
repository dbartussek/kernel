#![no_std]

pub mod logger;

use core::fmt::Write;
use kernel_spin::KernelMutex;
///! With thanks to
///! https://os.phil-opp.com/testing/#serial-port
use lazy_static::lazy_static;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: KernelMutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        KernelMutex::new(serial_port)
    };
}

pub fn access_serial<F>(f: F)
where
    F: FnOnce(&mut SerialPort),
{
    SERIAL1.lock(|serial| {
        assert_ne!(
            (&serial as &SerialPort as *const SerialPort),
            core::ptr::null()
        );

        f(serial)
    });
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    access_serial(move |serial| {
        serial.write_fmt(args).expect("Printing to serial failed")
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

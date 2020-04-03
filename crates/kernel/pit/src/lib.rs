//! X86 pic8259 pit configuration
//!
//! Based on https://en.wikibooks.org/wiki/X86_Assembly/Programmable_Interval_Timer
//! and https://wiki.osdev.org/Programmable_Interval_Timer

#![no_std]

use core::time::Duration;
use x86_64::instructions::port::*;

/// Programmable interrupt timer
///
///
pub struct Pit {
    channel_0: Port<u8>,
    channel_0_divider: u16,
    channel_0_duration: Duration,

    // Channel 1 was once used for DRAM refresh. It no longer exists

    // Channel 2 is used for the audio speaker. We don't use it here
    command: PortWriteOnly<u8>,
}

impl Pit {
    pub const fn new() -> Self {
        Pit {
            channel_0: Port::new(0x40),
            channel_0_divider: 0,
            channel_0_duration: Duration::from_secs(0),
            command: PortWriteOnly::new(0x43),
        }
    }

    pub unsafe fn init(&mut self) {
        self.write_command(Command::default());
        self.set_divider(0);
    }

    pub unsafe fn set_divider(&mut self, divider: u16) {
        self.channel_0_divider = divider;
        self.channel_0_duration = Self::calculate_timer_duration(divider);

        self.channel_0.write(divider as u8);
        self.channel_0.write((divider >> 8) as u8);
    }

    pub unsafe fn write_command(&mut self, command: Command) {
        self.command.write(command.compile())
    }

    /// How much time passes between each interrupt when the divider == 1
    pub fn base_duration() -> Duration {
        const FREQUENCY: u64 = 1_193_182;
        const SECOND: u64 = 1_000_000_000;

        Duration::from_nanos(SECOND / FREQUENCY)
    }

    fn calculate_timer_duration(frequency: u16) -> Duration {
        Self::base_duration()
            * if frequency == 0 {
                (core::u16::MAX as u32) + 1
            } else {
                frequency as u32
            }
    }

    pub fn duration(&self) -> Duration {
        self.channel_0_duration
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum AccessMode {
    Latch = 0b00,
    LoHi = 0b11,
}

impl Default for AccessMode {
    fn default() -> Self {
        AccessMode::LoHi
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct Command {
    pub access_mode: AccessMode,
}

impl Command {
    pub fn compile(self) -> u8 {
        let access_mode = (self.access_mode as u8) << 4;

        // Repeated interrupts
        let mode = 2 << 1;

        access_mode | mode
    }
}

#![no_std]

use page_management::page_table::identity_base;
use x86_64::registers::model_specific::Msr;

#[repr(C, align(16))]
struct Register(u32);

impl Register {
    pub unsafe fn read(&self) -> u32 {
        (&self.0 as *const u32).read_volatile()
    }

    pub unsafe fn write(&mut self, value: u32) {
        (&mut self.0 as *mut u32).write_volatile(value)
    }
}

#[repr(transparent)]
struct ReservedRegister(Register);
#[repr(transparent)]
pub struct ReadRegister(Register);
#[repr(transparent)]
pub struct WriteRegister(Register);
#[repr(transparent)]
pub struct ReadWriteRegister(Register);

impl ReadRegister {
    pub unsafe fn read(&self) -> u32 {
        self.0.read()
    }
}
impl WriteRegister {
    pub unsafe fn write(&mut self, value: u32) {
        self.0.write(value)
    }
}
impl ReadWriteRegister {
    pub unsafe fn read(&self) -> u32 {
        self.0.read()
    }

    pub unsafe fn write(&mut self, value: u32) {
        self.0.write(value)
    }
}

#[repr(C)]
pub struct Registers {
    _reserved_a: [ReservedRegister; 2],

    pub id: ReadWriteRegister,
    pub version: ReadRegister,

    _reserved_b: [ReservedRegister; 4],

    pub task_priority: ReadWriteRegister,
    pub arbitration_priority: ReadRegister,
    pub processor_priority: ReadRegister,

    pub end_of_interrupt: WriteRegister,

    pub remote_read: ReadRegister,

    pub logical_destination: ReadWriteRegister,
    pub destination_format: ReadWriteRegister,

    pub spurious_interrupt_vector: ReadWriteRegister,

    pub in_service_register: [ReadRegister; 8],
    pub trigger_mode: [ReadRegister; 8],
    pub interrupt_request: [ReadRegister; 8],
    pub error_status: ReadRegister,

    _reserved_c: [ReservedRegister; 6],

    pub lvt_corrected_machine_check_interrupt: ReadWriteRegister,

    pub interrupt_command: [ReadWriteRegister; 2],

    pub lvt_timer: ReadWriteRegister,
    pub lvt_thermal_sensor: ReadWriteRegister,
    pub lvt_performance_monitoring_counters: ReadWriteRegister,

    pub lvt_lint0: ReadWriteRegister,
    pub lvt_lint1: ReadWriteRegister,
    pub lvt_error: ReadWriteRegister,

    pub initial_count: ReadWriteRegister,
    pub current_count: ReadRegister,

    _reserved_d: [ReservedRegister; 4],

    pub divide_configuration_register: ReadWriteRegister,

    _reserved_e: ReservedRegister,
}

impl Registers {
    pub unsafe fn global() -> &'static mut Self {
        // TODO UEFI may have changed the APIC base address.
        // Parse the ACPI tables to figure it out properly

        &mut *(identity_base().start_address() + 0xFEE00000u64)
            .as_mut_ptr::<Self>()
    }

    pub fn end_of_interrupt(&mut self) {
        unsafe {
            self.end_of_interrupt.write(0);
        }
    }
}

#[repr(u32)]
pub enum ApicTimerDivider {
    Divide2 = 0,
    Divide4 = 1,
    Divide8 = 2,
    Divide16 = 3,
    Divide32 = 4,
    Divide64 = 5,
    Divide128 = 6,
    Divide1 = 7,
}

#[repr(u32)]
pub enum ApicTimerMode {
    OneShot = 0,
    Periodic = 1,
    TscDeadline = 2,
}

pub const SPURIOUS_INTERRUPT: u32 = 0xFF;
pub const TIMER_INTERRUPT: u32 = 48;

pub unsafe fn init() {
    let mut apic_base = Msr::new(0x1B);

    let lapic = Registers::global();

    // Reset apic values
    lapic.destination_format.write(0xFF_FF_FF_FF);

    lapic
        .logical_destination
        .write((lapic.logical_destination.read() & 0x00_FF_FF_FF) | 1);

    lapic.lvt_timer.write(0x10000);
    lapic.lvt_lint0.write(0x10000);
    lapic.lvt_lint1.write(0x10000);

    lapic.task_priority.write(0);

    // Enable apic globally
    apic_base.write(apic_base.read() | (1 << 11));

    // Enable apic by mapping spurious interrupt timer
    lapic
        .spurious_interrupt_vector
        .write(SPURIOUS_INTERRUPT | 0x100);

    // Enable the timer interrupt and set it to vector 48
    lapic
        .lvt_timer
        .write(((ApicTimerMode::Periodic as u32) << 17) | TIMER_INTERRUPT);

    lapic.initial_count.write(0x1_00_00);
    lapic
        .divide_configuration_register
        .write(ApicTimerDivider::Divide16 as u32);
}

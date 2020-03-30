mod core_id;

pub use self::core_id::*;

pub struct CpuLocalData {
    pub core_id: CoreId,
}

#[repr(C, packed)]
pub struct MDMemoryDescriptor64 {
    pub start_of_memory_range: u64,
    pub data_size: u64,
}

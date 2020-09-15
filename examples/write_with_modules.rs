use minidump_writer::{
    minidump::Minidump,
    streams::{
        MemoryDescriptor, MemoryListStream, MinidumpModule, ModuleListStream, SystemInfoStream,
    },
};

use std::fs::File;

fn main() {
    let mut file = File::create("mini.dmp").unwrap();
    let mut minidump = Minidump::default();

    let mut module_list = ModuleListStream::default();

    module_list.add_module(MinidumpModule {
        base_of_image: 0xffffffaa345000,
        size_of_image: 0x12000,
        checksum: 0,
        time_date_stamp: 0,
        name: "epic.exe".to_string(),
    });

    minidump.directory.push(Box::new(module_list));

    let mut memory_list = MemoryListStream::default();

    memory_list.list.push(MemoryDescriptor {
        start_of_memory: 0xffffffaa345000,
        buf: &[77, 90, 0xff, 1, 2, 3, 4, 5, 6, 7, 2, 3, 3],
    });

    memory_list.list.push(MemoryDescriptor {
        start_of_memory: 0xffffffaa349000,
        buf: &[37, 10, 0xf3, 1, 2, 3, 4, 5, 6, 7, 2, 3, 3, 1],
    });

    minidump.directory.push(Box::new(memory_list));

    let system_info = SystemInfoStream::default();
    minidump.directory.push(Box::new(system_info));

    minidump
        .write_all(&mut file)
        .expect("Failed to write minidump");
}

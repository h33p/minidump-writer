use minidump_common::format as md;
use std::io::{self, Cursor, Seek, SeekFrom, Write};
use std::ops::Deref;

use crate::md_extras;
use crate::util::as_slice;

pub trait MinidumpStream {
    fn write(&self, pos: usize, writer: &mut dyn Write) -> io::Result<()>;
    fn stream_type(&self) -> u32;
}

#[derive(Default)]
pub struct ModuleListStream {
    modules: Vec<MinidumpModule>,
}

impl MinidumpStream for ModuleListStream {
    fn write(&self, pos: usize, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&(self.modules.len() as u32).to_ne_bytes())?;

        let end = pos
            + std::mem::size_of::<u32>()
            + self.modules.len() * std::mem::size_of::<md::MINIDUMP_MODULE>();

        let mut end_buf: Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut end_buf);

        for i in &self.modules {
            let mut win_str: Vec<u8> = vec![];
            for c in i.name.encode_utf16() {
                win_str.extend_from_slice(&c.to_ne_bytes());
            }
            let rva = end + cursor.seek(SeekFrom::Current(0))? as usize;
            cursor.write_all(&(win_str.len() as u32).to_ne_bytes())?;
            cursor.write_all(&win_str)?;
            // Null terminator
            cursor.write_all(&0u16.to_ne_bytes())?;

            let module = md::MINIDUMP_MODULE {
                base_of_image: i.base_of_image,
                size_of_image: i.size_of_image,
                time_date_stamp: i.time_date_stamp,
                checksum: i.checksum,
                module_name_rva: rva as _,
                cv_record: md::MINIDUMP_LOCATION_DESCRIPTOR {
                    data_size: 0,
                    rva: 0,
                },
                misc_record: md::MINIDUMP_LOCATION_DESCRIPTOR {
                    data_size: 0,
                    rva: 0,
                },
                version_info: Default::default(),
                reserved0: [0; 2],
                reserved1: [0; 2],
            };

            writer.write_all(unsafe { as_slice(&module) })?;
        }

        writer.write_all(&end_buf)?;

        Ok(())
    }

    fn stream_type(&self) -> u32 {
        4 //ModuleListStream
    }
}

impl ModuleListStream {
    pub fn add_module(&mut self, module: MinidumpModule) {
        self.modules.push(module);
    }
}

pub struct MinidumpModule {
    pub base_of_image: u64,
    pub size_of_image: u32,
    pub checksum: u32,
    pub time_date_stamp: u32,
    pub name: String,
}

#[derive(Default)]
pub struct MemoryListStream<T> {
    pub list: Vec<MemoryDescriptor<T>>,
}

impl<T: Deref<Target = [u8]>> MinidumpStream for MemoryListStream<T> {
    fn write(&self, pos: usize, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&(self.list.len() as u32).to_ne_bytes())?;

        let end = pos
            + std::mem::size_of::<u32>()
            + self.list.len() * std::mem::size_of::<md::MINIDUMP_MEMORY_DESCRIPTOR>();

        let mut end_buf: Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut end_buf);

        for i in &self.list {
            let rva = end + cursor.seek(SeekFrom::Current(0))? as usize;
            cursor.write_all(i.buf.deref())?;

            let descriptor = md::MINIDUMP_MEMORY_DESCRIPTOR {
                start_of_memory_range: i.start_of_memory,
                memory: md::MINIDUMP_LOCATION_DESCRIPTOR {
                    data_size: i.buf.len() as _,
                    rva: rva as _,
                },
            };

            writer.write_all(unsafe { as_slice(&descriptor) })?;
        }

        writer.write_all(&end_buf)?;
        Ok(())
    }

    fn stream_type(&self) -> u32 {
        5 //MemoryListStream
    }
}

#[derive(Default)]
pub struct Memory64ListStream<T> {
    pub list: Vec<MemoryDescriptor<T>>,
}

impl<T: Deref<Target = [u8]>> MinidumpStream for Memory64ListStream<T> {
    fn write(&self, pos: usize, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&(self.list.len() as u64).to_ne_bytes())?;

        let end = pos
            + std::mem::size_of::<u64>()
            + std::mem::size_of::<u64>()
            + self.list.len() * std::mem::size_of::<md_extras::MDMemoryDescriptor64>();

        writer.write_all(&(end as u64).to_ne_bytes())?;

        let mut end_buf: Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut end_buf);

        for i in &self.list {
            cursor.write_all(i.buf.deref())?;

            let descriptor = md_extras::MDMemoryDescriptor64 {
                start_of_memory_range: i.start_of_memory,
                data_size: i.buf.len() as _,
            };

            writer.write_all(unsafe { as_slice(&descriptor) })?;
        }

        writer.write_all(&end_buf)?;
        Ok(())
    }

    fn stream_type(&self) -> u32 {
        9 //Memory64ListStream
    }
}

pub struct MemoryDescriptor<T> {
    pub start_of_memory: u64,
    pub buf: T,
}

pub struct SystemInfoStream {
    pub system_info: md::MINIDUMP_SYSTEM_INFO,
    pub service_pack_str: String,
}

impl Default for SystemInfoStream {
    fn default() -> Self {
        Self {
            system_info: md::MINIDUMP_SYSTEM_INFO {
                processor_architecture: 0,
                processor_level: 0,
                processor_revision: 0,
                number_of_processors: 0,
                major_version: 0,
                minor_version: 0,
                build_number: 0,
                product_type: 0,
                platform_id: 0,
                csd_version_rva: 0,
                suite_mask: 0,
                reserved2: 0,
                cpu: md::CPU_INFORMATION { data: [0; 24] },
            },
            service_pack_str: String::new(),
        }
    }
}

impl SystemInfoStream {
    pub fn with_arch_and_version(
        arch: u32,
        major_version: u32,
        minor_version: u32,
        build_number: u32,
    ) -> Self {
        Self {
            system_info: md::MINIDUMP_SYSTEM_INFO {
                processor_architecture: arch as _,
                major_version,
                minor_version,
                build_number,

                // defaults
                processor_level: 0,
                processor_revision: 0,
                number_of_processors: 0,
                product_type: 0,
                platform_id: 0,
                csd_version_rva: 0,
                suite_mask: 0,
                reserved2: 0,
                cpu: md::CPU_INFORMATION { data: [0; 24] },
            },
            ..Default::default()
        }
    }
}

impl MinidumpStream for SystemInfoStream {
    fn write(&self, pos: usize, writer: &mut dyn Write) -> io::Result<()> {
        let end = pos + std::mem::size_of::<md::MINIDUMP_SYSTEM_INFO>();
        let mut system_info = self.system_info.clone();
        system_info.csd_version_rva = end as _;

        writer.write_all(unsafe { as_slice(&system_info) })?;

        let mut win_str: Vec<u8> = vec![];
        for c in self.service_pack_str.encode_utf16() {
            win_str.extend_from_slice(&c.to_ne_bytes());
        }
        writer.write_all(&(win_str.len() as u32).to_ne_bytes())?;
        writer.write_all(&win_str)?;
        // Null terminator
        writer.write_all(&0u16.to_ne_bytes())?;

        Ok(())
    }

    fn stream_type(&self) -> u32 {
        7 //SystemInfoStream
    }
}

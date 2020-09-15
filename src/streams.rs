use minidump_common::format as md;
use std::io::{self, Cursor, Seek, SeekFrom, Write};

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
            + self.modules.len() * std::mem::size_of::<md::MDRawModule>();

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

            let module = md::MDRawModule {
                base_of_image: i.base_of_image,
                size_of_image: i.size_of_image,
                time_date_stamp: i.time_date_stamp,
                checksum: i.checksum,
                module_name_rva: rva as _,
                cv_record: md::MDLocationDescriptor {
                    data_size: 0,
                    rva: 0,
                },
                misc_record: md::MDLocationDescriptor {
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
pub struct MemoryListStream<'a> {
    pub list: Vec<MemoryDescriptor<'a>>,
}

impl MinidumpStream for MemoryListStream<'_> {
    fn write(&self, pos: usize, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&(self.list.len() as u32).to_ne_bytes())?;

        let end = pos
            + std::mem::size_of::<u32>()
            + self.list.len() * std::mem::size_of::<md::MDMemoryDescriptor>();

        let mut end_buf: Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut end_buf);

        for i in &self.list {
            let rva = end + cursor.seek(SeekFrom::Current(0))? as usize;
            cursor.write_all(i.buf)?;

            let descriptor = md::MDMemoryDescriptor {
                start_of_memory_range: i.start_of_memory,
                memory: md::MDLocationDescriptor {
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
pub struct Memory64ListStream<'a> {
    pub list: Vec<MemoryDescriptor<'a>>,
}

impl MinidumpStream for Memory64ListStream<'_> {
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
            cursor.write_all(i.buf)?;

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

pub struct MemoryDescriptor<'a> {
    pub start_of_memory: u64,
    pub buf: &'a [u8],
}

#[derive(Default)]
pub struct SystemInfoStream {
    system_info: md::MDRawSystemInfo,
}

impl MinidumpStream for SystemInfoStream {
    fn write(&self, _pos: usize, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(unsafe { as_slice(&self.system_info) })?;
        Ok(())
    }

    fn stream_type(&self) -> u32 {
        7 //SystemInfoStream
    }
}
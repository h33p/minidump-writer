use crate::streams::MinidumpStream;
use crate::util::as_slice;
use minidump_common::format as md;
use std::io::{self, Cursor, Seek, Write};

#[derive(Default)]
pub struct Minidump {
    pub directory: Vec<Box<dyn MinidumpStream>>,
}

impl Minidump {
    pub fn write_all<T: Write + Seek>(&self, writer: &mut T) -> io::Result<()> {
        let header = md::MINIDUMP_HEADER {
            signature: md::MINIDUMP_SIGNATURE,
            version: md::MINIDUMP_VERSION,
            stream_count: self.directory.len() as _,
            stream_directory_rva: std::mem::size_of::<md::MINIDUMP_HEADER>() as _,
            checksum: 0,
            time_date_stamp: 0,
            flags: 0,
        };

        let mut buf = vec![];
        let mut cursor = Cursor::new(&mut buf);
        let offset: usize = header.stream_directory_rva as usize
            + self.directory.len() * std::mem::size_of::<md::MINIDUMP_DIRECTORY>();

        let mut directories = vec![];

        for st in &self.directory {
            let stream_type = st.stream_type();
            let pos_start = cursor.stream_position()? as usize;
            st.write(pos_start + offset, &mut cursor)?;
            let pos_end = cursor.stream_position()? as usize;

            // Pad the end so that the next entry is aligned
            let padding = 4 - pos_end % 4;
            cursor.write_all(&0u64.to_le_bytes()[..padding])?;
            let pos_end = pos_end + padding;

            directories.push(md::MINIDUMP_DIRECTORY {
                stream_type,
                location: md::MINIDUMP_LOCATION_DESCRIPTOR {
                    data_size: (pos_end - pos_start) as _,
                    rva: (pos_start + offset) as _,
                },
            });
        }

        _ = writer.write(unsafe { as_slice(&header) })?;

        for d in &directories {
            _ = writer.write(unsafe { as_slice(d) })?;
        }

        _ = writer.write(&buf)?;

        Ok(())
    }
}

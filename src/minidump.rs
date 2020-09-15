use crate::streams::MinidumpStream;
use crate::util::as_slice;
use minidump_common::format as md;
use std::io::{self, Cursor, Seek, SeekFrom, Write};

#[derive(Default)]
pub struct Minidump {
    pub directory: Vec<Box<dyn MinidumpStream>>,
}

impl Minidump {
    pub fn write_all<T: Write + Seek>(&self, writer: &mut T) -> io::Result<()> {
        let header = md::MDRawHeader {
            signature: md::MD_HEADER_SIGNATURE,
            version: md::MD_HEADER_VERSION,
            stream_count: self.directory.len() as _,
            stream_directory_rva: std::mem::size_of::<md::MDRawHeader>() as _,
            checksum: 0,
            time_date_stamp: 0,
            flags: 0,
        };

        let mut buf = vec![];
        let mut cursor = Cursor::new(&mut buf);
        let offset: usize = header.stream_directory_rva as usize
            + self.directory.len() * std::mem::size_of::<md::MDRawDirectory>();

        let mut directories = vec![];

        for st in &self.directory {
            let stream_type = st.stream_type();
            let pos_start = cursor.seek(SeekFrom::Current(0))? as usize;
            st.write(pos_start + offset, &mut cursor)?;
            let pos_end = cursor.seek(SeekFrom::Current(0))? as usize;

            // Pad the end so that the next entry is aligned
            let padding = 4 - pos_end % 4;
            cursor.write_all(&0u64.to_le_bytes()[..padding])?;
            let pos_end = pos_end + padding;

            directories.push(md::MDRawDirectory {
                stream_type,
                location: md::MDLocationDescriptor {
                    data_size: (pos_end - pos_start) as _,
                    rva: (pos_start + offset) as _,
                },
            });
        }

        writer.write(unsafe { as_slice(&header) })?;

        for d in &directories {
            writer.write(unsafe { as_slice(d) })?;
        }

        writer.write(&buf)?;

        Ok(())
    }
}

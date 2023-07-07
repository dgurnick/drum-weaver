use std::io::{Cursor, Read, Seek};

use symphonia::core::io::MediaSource;

pub struct BeepMediaSource {
    data: &'static [u8],
    cursor: Cursor<&'static [u8]>,
}

impl BeepMediaSource {
    pub fn new(data: &'static [u8]) -> Self {
        Self { data, cursor: Cursor::new(data) }
    }
}

impl Read for BeepMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.cursor.read(buf)
    }
}

impl Seek for BeepMediaSource {
    fn seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, std::io::Error> {
        self.cursor.seek(pos)
    }
}

impl MediaSource for BeepMediaSource {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.data.len() as u64)
    }
}

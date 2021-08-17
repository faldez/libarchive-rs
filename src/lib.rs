use std::ffi::{CStr, CString};
use std::fmt;
use std::ptr;

use libarchive_sys::{
    archive, archive_entry_pathname_utf8, archive_error_string, archive_read_data,
    archive_read_free, archive_read_new, archive_read_next_header, archive_read_open_filename,
    archive_read_support_compression_all, archive_read_support_filter_all,
    archive_read_support_format_all, ARCHIVE_EOF, ARCHIVE_OK,
};

pub struct ArchiveReader {
    archive: *mut archive,
    state: ArchiveState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArchiveReaderError {
    /// failed to allocate an object
    AllocationFailure,
    // libarchive returned an error
    Message(String),
    WrongState,
}

impl std::error::Error for ArchiveReaderError {}

impl fmt::Display for ArchiveReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocationFailure => write!(f, "failed to allocate"),
            Self::Message(msg) => write!(f, "error: {}", msg),
            Self::WrongState => write!(f, "tried to read from the archive in the wrong state"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArchiveState {
    Initialized,
    /// Only in this state can data be read from the archive (prevents reading from the same file multiple times).
    ReadyForRead,
    /// Reached the end of the archive (no more data can be read)
    Eof,
}

impl ArchiveReader {
    pub fn new(filename: &CStr, block_size: usize) -> Result<Self, ArchiveReaderError> {
        let archive = unsafe { archive_read_new() };

        // a null pointer is returned if it failed to allocate
        if archive.is_null() {
            return Err(ArchiveReaderError::AllocationFailure);
        }

        // see: https://stackoverflow.com/a/62800478
        unsafe {
            // enable all available compression "filters" for the archive
            archive_read_support_filter_all(archive);
            // enable all possible file formats
            archive_read_support_format_all(archive);
            archive_read_support_compression_all(archive);
        }

        let res =
            unsafe { archive_read_open_filename(archive, filename.as_ptr(), block_size as u64) };
        if res != ARCHIVE_OK as i32 {
            let message = unsafe { CStr::from_ptr(archive_error_string(archive)) };
            let message = message.to_string_lossy().to_string();

            return Err(ArchiveReaderError::Message(message));
        }

        Ok(Self {
            archive,
            state: ArchiveState::Initialized,
        })
    }

    pub fn read(&mut self) -> Result<Vec<u8>, ArchiveReaderError> {
        let mut buf = [0_u8; 10_240];
        let mut data = vec![];

        // this prevents reading the same file more than once
        if self.state != ArchiveState::ReadyForRead {
            return Err(ArchiveReaderError::WrongState);
        }

        loop {
            let res = unsafe {
                archive_read_data(
                    self.archive,
                    buf.as_mut_ptr() as *mut std::ffi::c_void,
                    10_240,
                )
            };

            // check if the end has been reached:
            if res == 0 {
                break;
            // an error occured
            } else if res < 0 {
                let message = unsafe { CStr::from_ptr(archive_error_string(self.archive)) };
                let message = message.to_string_lossy().to_string();
                return Err(ArchiveReaderError::Message(message));
            }

            // res = number of bytes read
            data.extend_from_slice(&buf[0..res as usize]);
        }

        Ok(data)
    }
}

impl<'a> Iterator for ArchiveReader {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == ArchiveState::Eof {
            return None;
        }

        let mut entry = ptr::null_mut();

        let res = unsafe { archive_read_next_header(self.archive, &mut entry as *mut _) };

        if res == ARCHIVE_OK as i32 {
            self.state = ArchiveState::ReadyForRead;
            let c_str = unsafe { CStr::from_ptr(archive_entry_pathname_utf8(entry)) };
            let name = c_str.to_str().unwrap().to_string();

            Some(name)
        } else if res == ARCHIVE_EOF as i32 {
            self.state = ArchiveState::Eof;
            None
        } else {
            panic!("a fatal error occured while reading the archive")
        }
    }
}

impl Drop for ArchiveReader {
    fn drop(&mut self) {
        // SAFETY: free might fail, but drops should always succeed
        unsafe { archive_read_free(self.archive) };
    }
}

pub fn list_archive_files(filename: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let filename = CString::new(filename).expect("CString::new failed");
    let reader = ArchiveReader::new(&filename, 10_240)?;

    Ok(reader.collect::<Vec<_>>())
}

pub fn extract_archive_file(
    filename: &str,
    path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let filename = CString::new(filename).expect("CString::new failed");
    let mut reader = ArchiveReader::new(&filename, 16_384)?;

    if reader.any(|file| file == path) {
        Ok(reader.read()?)
    } else {
        Err(format!("error could not find the file: {}", path).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_ZIP_PATH: &str = "resources/example.zip";

    #[test]
    fn test_list_archive_files() {
        assert_eq!(
            list_archive_files(EXAMPLE_ZIP_PATH).unwrap(),
            vec![
                "a.txt".to_string(),
                "b.txt".to_string(),
                "c.txt".to_string(),
            ]
        );
    }

    #[test]
    fn test_extract_archive_file() {
        assert_eq!(
            extract_archive_file(EXAMPLE_ZIP_PATH, "a.txt").unwrap(),
            b"a".to_vec()
        );
        assert_eq!(
            extract_archive_file(EXAMPLE_ZIP_PATH, "b.txt").unwrap(),
            b"b".to_vec()
        );
        assert_eq!(
            extract_archive_file(EXAMPLE_ZIP_PATH, "c.txt").unwrap(),
            b"c".to_vec()
        );
    }
}

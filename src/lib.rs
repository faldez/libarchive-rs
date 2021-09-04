use std::ffi::{c_void, CStr, CString};
use std::fmt;
use std::io::Read;
use std::ptr;

use libarchive_sys::{
    archive, archive_entry, archive_entry_pathname, archive_error_string, archive_read_close,
    archive_read_data, archive_read_free, archive_read_new, archive_read_next_header,
    archive_read_open, archive_read_support_compression_all, archive_read_support_format_all,
    archive_set_error, la_ssize_t, ARCHIVE_EOF, ARCHIVE_OK,
};

const BLOCK_SIZE: usize = 10_240;

pub struct ArchiveReader<R>
where
    R: Read,
{
    archive: *mut archive,
    entry: *mut archive_entry,
    state: ArchiveState,
    _reader: Reader<R>,
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

pub struct Reader<R>
where
    R: Read,
{
    reader: R,
    buffer: [u8; BLOCK_SIZE],
}

unsafe extern "C" fn libarchive_read_callback<R>(
    archive: *mut archive,
    client_data: *mut c_void,
    buff: *mut *const c_void,
) -> la_ssize_t
where
    R: Read,
{
    let reader = (client_data as *mut Reader<R>).as_mut().unwrap();

    *buff = reader.buffer.as_ptr() as *const c_void;

    match reader.reader.read(&mut reader.buffer) {
        Ok(size) => size as la_ssize_t,
        Err(e) => {
            let str = CString::new(e.to_string()).unwrap();
            archive_set_error(archive, e.raw_os_error().unwrap_or(0), str.as_ptr());
            -1
        }
    }
}

impl<R> ArchiveReader<R>
where
    R: Read,
{
    pub fn from_read(read: R) -> Result<Self, ArchiveReaderError> {
        let mut reader = Reader {
            reader: read,
            buffer: [0; BLOCK_SIZE],
        };

        let entry: *mut archive_entry = ptr::null_mut();

        let archive = unsafe { archive_read_new() };

        // a null pointer is returned if it failed to allocate
        if archive.is_null() {
            return Err(ArchiveReaderError::AllocationFailure);
        }

        // see: https://stackoverflow.com/a/62800478
        unsafe {
            // enable all available compression "filters" for the archive
            archive_read_support_compression_all(archive);
            // enable all possible file formats
            archive_read_support_format_all(archive);
        }

        let res = unsafe {
            archive_read_open(
                archive,
                (&mut reader as *mut Reader<R>) as *mut c_void,
                None,
                Some(libarchive_read_callback::<R>),
                None,
            )
        };

        if res != ARCHIVE_OK as i32 {
            let message = unsafe { CStr::from_ptr(archive_error_string(archive)) };
            let message = message.to_string_lossy().to_string();

            return Err(ArchiveReaderError::Message(message));
        }

        Ok(Self {
            archive,
            entry,
            _reader: reader,
            state: ArchiveState::Initialized,
        })
    }

    pub fn read(&mut self) -> Result<Vec<u8>, ArchiveReaderError> {
        let mut buf = [0_u8; BLOCK_SIZE];
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
                    BLOCK_SIZE as _,
                )
            };

            // check if the end has been reached:
            match res {
                0 => break,
                // an error occured
                res if res < 0 => {
                    let message = unsafe { CStr::from_ptr(archive_error_string(self.archive)) };
                    let message = message.to_string_lossy().to_string();
                    return Err(ArchiveReaderError::Message(message));
                }
                // res = number of bytes read
                _ => data.extend_from_slice(&buf[0..res as usize]),
            }
        }

        Ok(data)
    }
}

impl<'a, R> Iterator for ArchiveReader<R>
where
    R: Read,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == ArchiveState::Eof {
            return None;
        }

        unsafe {
            match archive_read_next_header(self.archive, &mut self.entry as *mut _) as _ {
                ARCHIVE_OK => {
                    self.state = ArchiveState::ReadyForRead;
                    let c_str = CStr::from_ptr(archive_entry_pathname(self.entry));
                    let name = c_str.to_string_lossy().to_string();

                    Some(name)
                }
                ARCHIVE_EOF => {
                    self.state = ArchiveState::Eof;
                    None
                }
                _ => {
                    let message = CStr::from_ptr(archive_error_string(self.archive));
                    let message = message.to_string_lossy().to_string();
                    panic!(
                        "a fatal error occured while reading the archive: {}",
                        message
                    )
                }
            }
        }
    }
}

impl<R> Drop for ArchiveReader<R>
where
    R: Read,
{
    fn drop(&mut self) {
        // SAFETY: free might fail, but drops should always succeed
        unsafe {
            archive_read_close(self.archive);
            archive_read_free(self.archive);
        }
    }
}

pub fn list_archive_files(filename: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file =
        std::fs::File::open(filename).map_err(|e| ArchiveReaderError::Message(e.to_string()))?;
    let reader = ArchiveReader::from_read(&file)?;

    let mut files = vec![];
    for f in reader {
        files.push(f);
    }

    Ok(files)
}

pub fn extract_archive_file(
    filename: &str,
    path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file =
        std::fs::File::open(filename).map_err(|e| ArchiveReaderError::Message(e.to_string()))?;
    let mut reader = ArchiveReader::from_read(&file)?;
    if reader.any(|file| file == path) {
        Ok(reader.read()?)
    } else {
        Err(format!("error could not find the file: {}", path).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_ZIP_PATH: &str = "resources/Space_Adventures_004__c2c__diff_ver.cbz";

    #[test]
    fn test_list_archive_files() {
        assert_eq!(
            list_archive_files(EXAMPLE_ZIP_PATH).unwrap(),
            vec![
                "SPA00401.JPG".to_string(),
                "SPA00402.JPG".to_string(),
                "SPA00403.JPG".to_string(),
                "SPA00404.JPG".to_string(),
                "SPA00405.JPG".to_string(),
                "SPA00406.JPG".to_string(),
                "SPA00407.JPG".to_string(),
                "SPA00408.JPG".to_string(),
                "SPA00409.JPG".to_string(),
                "SPA00410.JPG".to_string(),
                "SPA00411.JPG".to_string(),
                "SPA00412.JPG".to_string(),
                "SPA00413.JPG".to_string(),
                "SPA00414.JPG".to_string(),
                "SPA00415.JPG".to_string(),
                "SPA00416.JPG".to_string(),
                "SPA00417.JPG".to_string(),
                "SPA00418.JPG".to_string(),
                "SPA00419.JPG".to_string(),
                "SPA00420.JPG".to_string(),
                "SPA00421.JPG".to_string(),
                "SPA00422.JPG".to_string(),
                "SPA00423.JPG".to_string(),
                "SPA00424.JPG".to_string(),
                "SPA00425.JPG".to_string(),
                "SPA00426.JPG".to_string(),
                "SPA00427.JPG".to_string(),
                "SPA00428.JPG".to_string(),
                "SPA00429.JPG".to_string(),
                "SPA00430.JPG".to_string(),
                "SPA00431.JPG".to_string(),
                "SPA00432.JPG".to_string(),
                "SPA00433.JPG".to_string(),
                "SPA00434.JPG".to_string(),
                "SPA00435.JPG".to_string(),
                "SPA00436.JPG".to_string(),
            ]
        );
    }

    #[test]
    fn test_extract_archive_file() {
        let image = std::include_bytes!("../resources/SPA00401.JPG");
        assert_eq!(
            extract_archive_file(EXAMPLE_ZIP_PATH, "SPA00401.JPG").unwrap(),
            image
        );
    }
}

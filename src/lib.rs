use std::ffi::{CStr, CString};

use libarchive_sys::*;

pub fn list_archive_files(filename: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    unsafe {
        let a = archive_read_new();
        archive_read_support_filter_all(a);
        archive_read_support_format_all(a);

        let filename = CString::new(filename).expect("CString::new failed");
        let mut r = archive_read_open_filename(a, filename.as_ptr(), 10240);
        if r != ARCHIVE_OK as i32 {
            return Err(format!("failed to read archive: {}", r).into());
        }

        let mut entry: *mut archive_entry = std::mem::zeroed();
        let mut files = vec![];
        while archive_read_next_header(a, &mut entry as *mut _) != ARCHIVE_EOF as i32 {
            let c_str = CStr::from_ptr(archive_entry_pathname(entry));
            let name = c_str.to_str().unwrap();
            files.push(name.to_string());
            archive_read_data_skip(a);
        }

        r = archive_read_free(a);
        if r != ARCHIVE_OK as i32 {
            return Err(format!("failed to free: {}", r).into());
        }

        Ok(files)
    }
}

pub fn extract_archive_file(
    filename: &str,
    path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    unsafe {
        let a = archive_read_new();
        archive_read_support_compression_all(a);
        archive_read_support_format_all(a);

        let filename = CString::new(filename).expect("CString::new failed");
        let mut r = archive_read_open_filename(a, filename.as_ptr(), 16384);
        if r != ARCHIVE_OK as i32 {
            return Err(format!("failed to read archive: {}", r).into());
        }

        let mut entry: *mut archive_entry = std::mem::zeroed();
        let mut found = false;
        while archive_read_next_header(a, &mut entry as *mut _) != ARCHIVE_EOF as i32 {
            let c_str = CStr::from_ptr(archive_entry_pathname(entry));
            let name = c_str.to_str().unwrap();
            if name == path {
                found = true;
                break;
            }
            archive_read_data_skip(a);
        }

        if !found {
            return Err("path not found in archive".into());
        }

        let mut buf = [0_u8; 10240];
        let mut data = vec![];
        while archive_read_data(
            a,
            buf.as_mut_ptr() as *mut std::ffi::c_void,
            10240,
        ) > 0
        {
            data.extend_from_slice(&buf);
        }

        r = archive_read_free(a);
        if r != ARCHIVE_OK as i32 {
            return Err(format!("failed to free: {}", r).into());
        }

        Ok(data)
    }
}

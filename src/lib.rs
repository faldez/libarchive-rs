use std::ffi::{CStr, CString};

use libarchive_sys::*;

pub fn list_archive_files(filename: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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

pub fn uncompress_archive_file(filename: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    unsafe  {
        // int r;
        // ssize_t size;

        // struct archive *a = archive_read_new();
        // archive_read_support_compression_all(a);
        // archive_read_support_format_raw(a);
        // r = archive_read_open_filename(a, filename, 16384);
        // if (r != ARCHIVE_OK) {
        //   /* ERROR */
        // }
        // r = archive_read_next_header(a, &ae);
        // if (r != ARCHIVE_OK) {
        //   /* ERROR */
        // }

        // for (;;) {
        //   size = archive_read_data(a, buff, buffsize);
        //   if (size < 0) {
        //     /* ERROR */
        //   }
        //   if (size == 0)
        //     break;
        //   write(1, buff, size);
        // }

        // archive_read_free(a));

        let mut a = archive_read_new();
        archive_read_support_compression_all(a);
        archive_read_support_format_raw(a);

        let filename = CString::new(filename).expect("CString::new failed");
        let mut r = archive_read_open_filename(a, filename.as_ptr(), 16384);
        if r != ARCHIVE_OK as i32 {
            return Err(format!("failed to read archive: {}", r).into());
        }

        let mut entry: *mut archive_entry = std::mem::zeroed();
        while archive_read_next_header(a, &mut entry as *mut _) != ARCHIVE_EOF as i32 {
            let c_str = CStr::from_ptr(archive_entry_pathname(entry));
            let name = c_str.to_str().unwrap();
            files.push(name.to_string());
            archive_read_data_skip(a);
        }
    }
}
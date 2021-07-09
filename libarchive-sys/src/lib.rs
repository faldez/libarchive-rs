#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::*;

    #[test]
    fn list_contents() {
        // struct archive *a;
        // struct archive_entry *entry;
        // int r;
        // a = archive_read_new();
        // archive_read_support_filter_all(a);
        // archive_read_support_format_all(a);
        // r = archive_read_open_filename(a, "archive.tar", 10240); // Note 1
        // if (r != ARCHIVE_OK)
        //   exit(1);
        // while (archive_read_next_header(a, &entry) == ARCHIVE_OK) {
        //   printf("%s\n",archive_entry_pathname(entry));
        //   archive_read_data_skip(a);  // Note 2
        // }
        // r = archive_read_free(a);  // Note 3
        // if (r != ARCHIVE_OK)
        //   exit(1);

        unsafe {
            let mut a: *mut archive = std::mem::zeroed();
            let mut entry: *mut archive_entry = std::mem::zeroed();
            let mut r: i32 = 0;

            a = archive_read_new();
            archive_read_support_filter_all(a);
            archive_read_support_format_all(a);

            let path = CString::new("komi.cbz").expect("CString::new failed");
            r = archive_read_open_filename(a, path.as_ptr(), 10240);
            if r != ARCHIVE_OK as i32 {
                panic!("failed to read archive: {}", r);
            }
            
            while archive_read_next_header(a,  &mut entry as *mut _) > 0 {
                let c_str = std::ffi::CStr::from_ptr(archive_entry_pathname(entry));
                println!("{}", c_str.to_str().unwrap());
                archive_read_data_skip(a);
            }

            r = archive_read_free(a);
            if r != ARCHIVE_OK as i32 {
                panic!("failed to free: {}", r);
            }
        }
    }
}
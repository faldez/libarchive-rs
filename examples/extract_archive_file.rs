use libarchive_rs::*;

fn main() {
    let buf = extract_archive_file(
        "resources/Space_Adventures_004__c2c__diff_ver.cbz",
        "SPA00401.JPG",
    )
    .expect("failed to extract single file");
    if buf.len() > 0 {
        std::fs::write("SPA00401.JPG", &buf).expect("failed to write file");
    }
}

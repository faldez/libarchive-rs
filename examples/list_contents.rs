use libarchive_rs::*;

fn main() {
    let reader = ArchiveReader::new("resources/Space_Adventures_004__c2c__diff_ver.cbz").unwrap();
    for file in reader {
        println!("{}", file);
    }

    let files = list_archive_files("resources/Space_Adventures_004__c2c__diff_ver.cbz")
        .expect("failed to read file");
    for file in files {
        println!("{}", file);
    }
}

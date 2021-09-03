use libarchive_rs::*;

fn main() {
    let file = std::fs::File::open("resources/Space_Adventures_004__c2c__diff_ver.cbz").unwrap();
    let reader = ArchiveReader::from_read(&file).unwrap();
    for file in reader {
        println!("{}", file);
    }

    let files = list_archive_files("resources/Space_Adventures_004__c2c__diff_ver.cbz")
        .expect("failed to read file");
    for file in files {
        println!("{}", file);
    }
}

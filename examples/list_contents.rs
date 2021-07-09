use libarchive_rs::*;

fn main() {
    let files = list_archive_files("komi.cbz".to_string()).expect("failed to read file");
    for file in files {
        println!("{}", file);
    }
}

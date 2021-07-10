use libarchive_rs::*;

fn main() {
    let files = list_archive_files("komi.cbz").expect("failed to read file");
    for file in files {
        println!("{}", file);
    }
}

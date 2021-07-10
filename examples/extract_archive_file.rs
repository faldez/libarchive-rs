use libarchive_rs::*;

fn main() {
    let buf = extract_archive_file("komi.cbz", "Komi Can't Communicate - c001 (v01) - p000 [Cover] [dig] [Totally Normal] [VIZ Media] [danke-Empire] {r2}.jpg").expect("failed to extract single file");
    if buf.len() > 0 {
        std::fs::write("Komi Can't Communicate - c001 (v01) - p000 [Cover] [dig] [Totally Normal] [VIZ Media] [danke-Empire] {r2}.jpg", &buf).expect("failed to write file");
    }
}

use std::fs;
use std::path::Path;

fn main() {
    // The npm Cargo preflight fingerprints migration contents and clears stale SQLx embeds.
    let migrations = Path::new("migrations");
    println!("cargo:rerun-if-changed={}", migrations.display());

    let Ok(entries) = fs::read_dir(migrations) else {
        return;
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("sql"))
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

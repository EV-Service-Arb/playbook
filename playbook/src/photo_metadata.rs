
use sha2::{Digest, Sha256};
use serde_json::{json, Value};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

const EXTENSIONS: &[&str] = &["jpg", "jpeg", "tif", "tiff", "png", "webp", "heic", "heif"];

#[derive(clap::Args)]
pub struct Options {
    directory: PathBuf,
    #[arg(long)]
    city: String,
    #[arg(long)]
    seed: Option<u64>,
    /// synthetic fixtures, reproducible from the seed
    #[arg(long, requires_all = ["seed", "reference_unix"])]
    write_sidecars: bool,
    #[arg(long)]
    reference_unix: Option<i64>,
}

pub fn run(options: Options) {
    assert!(options.directory.is_dir(), "{} is not a directory", options.directory.display());
    let files = image_files(&options.directory);
    if files.is_empty() {
        panic!("{} contains no supported images", options.directory.display());
    }
    if Command::new("exiftool").arg("-ver").output().is_err() {
        panic!("exiftool is required but was not found on PATH");
    }

    let mut records = Vec::with_capacity(files.len());
    for (index, path) in files.iter().enumerate() {
        let hash = sha256(path);
        let metadata = exiftool(path);
        let sidecar = if options.write_sidecars {
            let seed = options.seed.expect("clap requires it with --write-sidecars");
            let reference = options.reference_unix.expect("clap requires it with --write-sidecars");
            let generated = synthetic_fixture(&options.city, seed, reference, index as u64);
            let sidecar_path = sidecar_path(path);
            let rendered = serde_json::to_string_pretty(&generated).expect("synthetic JSON is serializable");
            fs::write(&sidecar_path, format!("{rendered}\n")).unwrap_or_else(|error| panic!("writing {}: {error}", sidecar_path.display()));
            Some(sidecar_path.display().to_string())
        } else {
            None
        };
        records.push(json!({
            "path": path.display().to_string(),
            "sha256": hash,
            "metadata": metadata,
            "synthetic_sidecar": sidecar,
        }));
    }

    let report = json!({
        "synthetic": false,
        "purpose": "metadata-audit",
        "directory": options.directory.display().to_string(),
        "city_context": options.city,
        "records": records,
    });
    println!("{}", serde_json::to_string_pretty(&report).expect("report is serializable"));
}


fn image_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    visit(root, &mut files);
    files.sort();
    files
}

fn visit(directory: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(directory).unwrap_or_else(|error| panic!("reading {}: {error}", directory.display()));
    for entry in entries {
        let path = entry.expect("directory entry is readable").path();
        let file_type = fs::symlink_metadata(&path).expect("file type is readable").file_type();
        if file_type.is_symlink() { panic!("refusing symlink {}", path.display()); }
        if file_type.is_dir() { visit(&path, files); continue; }
        if file_type.is_file() && EXTENSIONS.iter().any(|extension| path.extension().and_then(OsStr::to_str).is_some_and(|actual| actual.eq_ignore_ascii_case(extension))) {
            files.push(path);
        }
    }
}

fn sha256(path: &Path) -> String {
    let mut file = File::open(path).unwrap_or_else(|error| panic!("opening {}: {error}", path.display()));
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 128 * 1024];
    loop {
        let read = file.read(&mut buffer).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));
        if read == 0 { break; }
        digest.update(&buffer[..read]);
    }
    format!("{:x}", digest.finalize())
}

fn exiftool(path: &Path) -> Value {
    let output = Command::new("exiftool")
        .args(["-json", "-G1", "-a", "-s"])
        .arg(path)
        .output()
        .unwrap_or_else(|error| panic!("running exiftool for {}: {error}", path.display()));
    assert!(output.status.success(), "exiftool failed for {}: {}", path.display(), String::from_utf8_lossy(&output.stderr));
    let parsed: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| panic!("exiftool returned invalid JSON for {}: {error}", path.display()));
    parsed.get(0).cloned().expect("exiftool returns one metadata object per image")
}

fn sidecar_path(path: &Path) -> PathBuf {
    let mut sidecar = path.as_os_str().to_os_string();
    sidecar.push(".synthetic-metadata.json");
    PathBuf::from(sidecar)
}

fn synthetic_fixture(city: &str, seed: u64, reference_unix: i64, index: u64) -> Value {
    let mut state = seed ^ index.wrapping_mul(0x9e3779b97f4a7c15);
    let age = next(&mut state) % (365 * 24 * 60 * 60);
    let timestamp = reference_unix.checked_sub(age as i64).expect("reference timestamp supports one year of subtraction");
    json!({
        "synthetic": true,
        "purpose": "test-fixture-only",
        "city": city,
        "generated_unix": timestamp,
        "seed": seed,
        "record_index": index,
        "notice": "Not recovered metadata. Not suitable for publication, attribution, provenance, or chain-of-custody claims.",
    })
}

fn next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *state
}

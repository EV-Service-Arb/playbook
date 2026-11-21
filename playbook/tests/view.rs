//! `playbook view` on a tree of one capture, one digest and one page.

use std::{
	fs,
	path::{Path, PathBuf},
	process::{Command, Output},
};

const README: &str = include_str!("../../knowledge/README.md");
const CAPTURE: &str = "# Group call

- source: <https://www.loom.com/share/abc>
- recorded: 2026-09-25T00:01:52.046Z
- duration: 00:10:00

## transcript

### [05:53](https://www.loom.com/share/abc?t=353)

said
";
const DIGEST: &str = "# Group call

- of: [src.md](src.md)
- digested by: a test, 2026-10-01

## gist

two members on their week

## chapters

### [05:53](https://www.loom.com/share/abc?t=353) Dante finds a contractor

- found a plumber on facebook marketplace in ten minutes
  <!--Dante ran 2σ ?t=353-->
";
const PAGE: &str = "# sourcing
<!--half-life 5y-->

post the job where contractors already look

## where

- (?) facebook marketplace finds one within the hour
  <!--2026-09-25 Dante ran 2σ https://www.loom.com/share/abc?t=353-->
";

fn tree(name: &str) -> PathBuf {
	let root = std::env::temp_dir().join(format!("view-{}-{name}", std::process::id()));
	for (path, text) in [
		("knowledge/README.md", README),
		("knowledge/references/loom/call/src.md", CAPTURE),
		("knowledge/references/loom/call/content.md", DIGEST),
		("knowledge/structured/contractors/sourcing.md", PAGE),
		("knowledge/references/loom/call/frames/1.webp", "webp"),
	] {
		fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
		fs::write(root.join(path), text).unwrap();
	}
	fs::create_dir_all(root.join(".git")).unwrap();
	root
}

fn view(root: &Path, args: &[&str]) -> Output {
	Command::new(env!("CARGO_BIN_EXE_playbook")).current_dir(root).arg("view").args(args).output().unwrap()
}

#[test]
fn renders_every_citation_in_place() {
	let root = tree("render");
	let out = root.with_extension("out");
	let run = |args: &[&str]| {
		let o = view(&root, &[&[out.to_str().unwrap()][..], args].concat());
		assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
		let read = |p: &str| fs::read_to_string(out.join(p)).unwrap();
		(
			read("structured/contractors/sourcing.md"),
			read("references/loom/call/content.md"),
			read("references/loom/call/src.md"),
		)
	};

	// r[verify trust.arithmetic+2]
	let (page, digest, capture) = run(&[]);
	for (rendered, original) in [(&page, PAGE), (&digest, DIGEST), (&capture, CAPTURE)] {
		assert_eq!(rendered.lines().count(), original.lines().count(), "line for line:\n{rendered}");
		assert!(!rendered.lines().any(|l| l.contains("<!--") && l.contains('σ')), "a raw citation is left:\n{rendered}");
	}
	let label = page.lines().find(|l| l.trim_start().starts_with('[')).unwrap();
	for part in ["[Dante ran, 95% sure · ", "d · init 0.", " · norm 0.", " · https://www.loom.com/share/abc?t=353]"] {
		assert!(label.contains(part), "{part} in {label}");
	}
	assert!(digest.contains("[Dante ran, 95% sure · "), "{digest}");
	assert!(digest.contains(" · ?t=353]"), "{digest}");
	assert!(
		page.starts_with("# sourcing · half-life 5y, suggested by knowledge/structured/contractors/sourcing.md · step 4\n"),
		"{page}"
	);
	assert!(capture.lines().any(|l| l.starts_with("- recorded: 2026-09-25T00:01:52.046Z · ") && l.ends_with('d')), "{capture}");
	assert!(fs::symlink_metadata(out.join("references/loom/call/frames/1.webp")).unwrap().is_symlink());

	let norm = |page: &str| page.split(" · norm ").nth(1).unwrap()[..4].to_owned();
	let (overridden, ..) = run(&["--half-life", "1d"]);
	assert!(overridden.starts_with("# sourcing · half-life 1d, --half-life · step 4\n"), "{overridden}");
	assert!(norm(&overridden) < norm(&page), "a day's half-life leaves less than five years':\n{overridden}\n{page}");
	let (stepped, ..) = run(&["--half-life", "1d", "--step", "100"]);
	assert!(stepped.lines().next().unwrap().ends_with(" · step 100, --step"), "{stepped}");
	assert!(norm(&stepped) > norm(&overridden), "a wider step ages the tail slower:\n{stepped}\n{overridden}");

	fs::remove_dir_all(&root).unwrap();
	fs::remove_dir_all(&out).unwrap();
}

#[test]
fn refuses_a_dir_it_did_not_write() {
	let root = tree("refuse");
	let out = root.with_extension("out");
	fs::create_dir_all(&out).unwrap();
	fs::write(out.join("notes.md"), "mine").unwrap();
	let o = view(&root, &[out.to_str().unwrap()]);
	assert!(!o.status.success());
	assert!(String::from_utf8_lossy(&o.stderr).contains("is no view"), "{}", String::from_utf8_lossy(&o.stderr));
	assert_eq!(fs::read_to_string(out.join("notes.md")).unwrap(), "mine");

	// r[verify trust.derived+2]
	let inside = view(&root, &[root.join("view").to_str().unwrap()]);
	assert!(!inside.status.success());
	assert!(!root.join("knowledge/view").exists());

	fs::remove_dir_all(&root).unwrap();
	fs::remove_dir_all(&out).unwrap();
}

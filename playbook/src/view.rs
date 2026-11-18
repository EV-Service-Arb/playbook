//! `playbook view`: `knowledge/` mirrored line for line, every citation rendered as what it is worth.

use std::{
	fs,
	path::{Path, PathBuf},
};

use corpus::Timeframe;

/// What marks a directory as a view, so a later one may replace it.
const MARK: &str = ".playbook-view";
const DATED: [&str; 4] = ["- recorded: ", "- uploaded: ", "- updated: ", "- pulled: "];

pub fn run(root: &Path, dir: &Path, half_life: Option<Timeframe>, step: Option<f64>) {
	let refuse = |why: String| -> ! {
		eprintln!("{why}");
		std::process::exit(1)
	};
	let index = corpus::compile(root).unwrap_or_else(|errors| {
		errors.iter().for_each(|e| eprintln!("{e}"));
		refuse(format!("{} errors under knowledge/; nothing viewed", errors.len()))
	});
	if half_life.is_some_and(|h| h.0 == 0) {
		refuse("a half-life is longer than nothing".into());
	}
	if step.is_some_and(|s| s <= 1.) {
		refuse("a step is above 1: how many times older is one notch more ancient".into());
	}
	fs::create_dir_all(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
	let dir = fs::canonicalize(dir).expect("just created");
	let root = fs::canonicalize(root).expect("the checkout exists");
	if dir.starts_with(&root) {
		// r[impl trust.derived+2]
		refuse(format!("{} is inside the checkout; a view goes outside it", dir.display()));
	}
	if fs::read_dir(&dir).expect("just created").next().is_some() {
		if !dir.join(MARK).is_file() {
			refuse(format!("{} holds files and is no view; nothing touched", dir.display()));
		}
		fs::remove_dir_all(&dir).expect("a view is ours to replace");
		fs::create_dir(&dir).expect("just removed");
	}
	fs::write(dir.join(MARK), "written by `playbook view`, and replaced whole by the next\n").expect("the view dir is writable");

	let today = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("after 1970").as_secs() as i64 / 86_400;
	let knowledge = root.join("knowledge");
	let mut files = Vec::new();
	walk(&knowledge, &mut files);
	for path in files {
		let out = dir.join(path.strip_prefix(&knowledge).expect("walked from it"));
		fs::create_dir_all(out.parent().expect("under the view")).expect("the view dir is writable");
		if path.extension().is_none_or(|e| e != "md") {
			std::os::unix::fs::symlink(&path, &out).unwrap_or_else(|e| panic!("{}: {e}", out.display()));
			continue;
		}
		let rel = path.strip_prefix(&root).expect("walked from it").to_str().expect("corpus paths are utf-8");
		let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
		let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
		for (i, backing) in index.claims.get(rel).into_iter().flatten().flat_map(|c| &c.backing) {
			let indent = lines[*i].len() - lines[*i].trim_start().len();
			lines[*i] = format!("{}[{}]", &lines[*i][..indent], backing.label(today, half_life, step));
		}
		if let Some(h) = index.half_lives.get(rel)
			&& lines.first().is_some_and(|l| l.starts_with("# "))
		{
			lines[0] += &match half_life {
				Some(span) => format!(" · half-life {span}, --half-life"),
				None => format!(" · {h}"),
			};
			lines[0] += &match step {
				Some(s) => format!(" · step {s}, --step"),
				None => format!(" · step {}", index.step),
			};
		}
		if let Some(capture) = index.captures.get(rel) {
			let dated = DATED
				.iter()
				.find_map(|k| lines.iter().position(|l| l.starts_with(k)))
				.expect("`compile` holds a capture to a date");
			lines[dated] += &format!(" · {}", corpus::age(today - capture.day));
			if let Some(speaker) = &capture.speaker {
				let source = lines.iter().position(|l| l.starts_with("- source: ")).expect("a capture states its source");
				lines[source] += &format!(" · {}", speaker.label(today, half_life, step));
			}
		}
		let end = if text.ends_with('\n') { "\n" } else { "" };
		fs::write(&out, lines.join("\n") + end).unwrap_or_else(|e| panic!("{}: {e}", out.display()));
	}
	eprintln!("{} viewed into {}", knowledge.display(), dir.display());
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
	for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
		let path = entry.expect("dir entries are readable").path();
		match path.is_dir() {
			true => walk(&path, out),
			false => out.push(path),
		}
	}
}

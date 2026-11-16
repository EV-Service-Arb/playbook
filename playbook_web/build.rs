//! Bakes the corpus, the guides and the operations' docs into the binary: the server reads nothing from disk per
//! request, and the image carries only what is listed here (docs/ARCHITECTURE.md, Invariants).

use std::{
	fmt::Write,
	fs,
	path::{Path, PathBuf},
};

fn main() {
	let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
		.parent()
		.expect("playbook_web/ sits in the workspace root")
		.to_path_buf();
	let mut files = Vec::new();
	for dir in ["guides", "knowledge/structured/approved", "knowledge/structured/suggested", "knowledge/references"] {
		println!("cargo:rerun-if-changed={}", root.join(dir).display());
		walk(&root.join(dir), &mut files);
	}
	files.sort();

	let mut out = String::from("pub static FILES: &[(&str, &str)] = &[\n");
	let mut guides = Vec::new();
	for f in &files {
		let rel = f.strip_prefix(&root).unwrap().to_str().expect("corpus paths are utf-8");
		if let Some(name) = rel.strip_prefix("guides/") {
			let name = name.trim_end_matches(".md");
			let text = fs::read_to_string(f).unwrap();
			let covers = text
				.lines()
				.next()
				.and_then(|l| l.strip_prefix(&format!("# {name}: ")))
				.unwrap_or_else(|| panic!("{rel} opens `# {name}: <what it covers>`"));
			guides.push(format!("- {name}: {covers}"));
		}
		writeln!(out, "\t({rel:?}, include_str!({:?})),", f.display().to_string()).unwrap();
	}
	out.push_str("];\n");
	let guides = guides.join("\n");
	writeln!(out, "const GUIDES: &str = {guides:?};").unwrap();

	// the image carries no code and no docs, so what the ranking and its labels take from them is inlined
	let cite_check = root.join("playbook/src/cite_check.rs");
	println!("cargo:rerun-if-changed={}", cite_check.display());
	let cite_check = fs::read_to_string(&cite_check).unwrap();
	let decay_days = cite_check
		.lines()
		.find_map(|l| l.strip_prefix("const DECAY_DAYS: f64 = ")?.strip_suffix(".;"))
		.expect("cite_check.rs states `const DECAY_DAYS: f64 = <n>.;`");
	writeln!(out, "const DECAY_DAYS: f64 = {decay_days}.;").unwrap();
	let architecture = root.join("docs/ARCHITECTURE.md");
	println!("cargo:rerun-if-changed={}", architecture.display());
	let architecture = fs::read_to_string(&architecture).unwrap();
	let reliability = architecture.split_once("\n## Reliability\n").expect("docs/ARCHITECTURE.md has `## Reliability`").1;
	let reliability = &reliability[..reliability.find("\n## ").unwrap_or(reliability.len())];
	let (mut roles, mut people, mut in_people) = (Vec::new(), Vec::new(), false);
	for l in reliability.lines() {
		in_people |= l.contains("id=\"people\"");
		let cells: Vec<&str> = l.split('|').map(str::trim).collect();
		let [_, base, who, _] = cells[..] else { continue };
		let Ok(base) = base.parse::<u8>() else { continue }; // the header and its rule
		let who = who.replace(['[', ']'], "");
		match in_people {
			false => roles.push(format!("{base} {who}")),
			true => people.push((who, base)),
		}
	}
	assert!(
		!roles.is_empty() && !people.is_empty(),
		"docs/ARCHITECTURE.md#reliability holds the role table, then the people table"
	);
	writeln!(out, "const PEOPLE: &[(&str, u8)] = &{people:?};").unwrap();
	let people: Vec<String> = people.iter().map(|(who, base)| format!("{base} {who}")).collect();
	// `document()` takes each handler's summary, then its description after a blank line
	let docs = [
		(
			"search",
			format!(
				"Case-insensitive regex over the service-arb playbook.\n\nEach kind of source has its own slots, so none crowds out another: knowledge/structured/approved and knowledge/structured/suggested notes, the course (lessons and the recordings they link), calls, videos, texts. Within a kind, a passage holding the rarer of the pattern's alternatives ranks first. Each hit gives path:line, its header, its label, the source URL and the lines around it; the files past the slots follow as path:line, for `read`. A pattern hitting most of the scope is refused, with the sub-scopes it hits.\n\nA label is who said it, their base reliability, the date, and the effective reliability: base plus age, lower is better, 10 at most and as good as nothing. A call's base is its speaker's, so its label gives the age term to add to it instead.\n\nBase by role: {}.\nBase by person, whatever their role: {}.",
				roles.join(" · "),
				people.join(" · ")
			),
		),
		(
			"read",
			"One chapter of a capture (a `###` under `## transcript` or `## summary`) or one section of a note, whole, with every line numbered.\n\nNever a whole file.".to_owned(),
		),
		(
			"guide",
			format!(
				"One guide, whole: how to decide in one area, as what matters, how to decide, and what to do when it goes wrong.\n\nLines ending `// inferred` are drafted, not yet vetted. The guides:\n{guides}"
			),
		),
	];
	writeln!(out, "pub(crate) static DOCS: ev_lib::api::Docs = &[").unwrap();
	for (handler, doc) in docs {
		writeln!(out, "\t(\"playbook_web::tools::{handler}\", {doc:?}),").unwrap();
	}
	writeln!(out, "];").unwrap();

	let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	fs::write(out_dir.join("corpus.rs"), out).unwrap();
}

/// `README.md`s are indices and format rules, not knowledge. `knowledge/references/cases/` states no source and no date.
fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
	for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
		let path = entry.unwrap().path();
		if path.ends_with("knowledge/references/cases") {
			continue;
		}
		if path.is_dir() {
			walk(&path, files);
		} else if path.extension().is_some_and(|e| e == "md") && path.file_name().unwrap() != "README.md" {
			files.push(path);
		}
	}
}

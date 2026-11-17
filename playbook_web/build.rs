//! Compiles the corpus and bakes it and the operations' docs into the binary: the server reads nothing from disk per
//! request, and the image carries only what is listed here.

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
	let knowledge = root.join("knowledge");
	println!("cargo:rerun-if-changed={}", knowledge.display());
	let index = corpus::compile(&root).unwrap_or_else(|errors| panic!("knowledge/ does not compile:\n{}", errors.join("\n")));
	for pending in &index.pending {
		println!("cargo:warning={pending}: no `content.md` reads it yet");
	}

	// r[impl knowledge.image]
	let mut files = vec![knowledge.join("README.md")];
	walk(&knowledge.join("structured"), &mut files);
	walk(&knowledge.join("references"), &mut files);
	files.sort();

	let mut out = String::from("pub static FILES: &[(&str, &str)] = &[\n");
	let mut pages = Vec::new();
	for f in &files {
		let rel = f.strip_prefix(&root).unwrap().to_str().expect("corpus paths are utf-8");
		if let Some(page) = rel.strip_prefix("knowledge/structured/") {
			let topic = fs::read_to_string(f).unwrap();
			let topic = topic.lines().next().and_then(|l| l.strip_prefix("# ")).expect("`corpus::compile` holds a page to its opening line");
			pages.push(format!("- {}: {topic}", page.trim_end_matches(".md")));
		}
		writeln!(out, "\t({rel:?}, include_str!({:?})),", f.display().to_string()).unwrap();
	}
	out.push_str("];\n");
	let guide = format!(
		"One page of the manual, whole: what we do in an area.\n\nEach step or decision in the order it is met, and the evidence under it, every citation given as who, how they know, when, and what that is worth today. `people`: who has run what, and how to reach them. The pages:\n{}",
		pages.join("\n")
	);
	writeln!(out, "const GUIDE_DESCRIPTION: &str = {guide:?};").unwrap();
	// `document()` takes each handler's summary, then its description after a blank line
	let docs = [
		(
			"search",
			"Case-insensitive regex over the service-arb playbook.\n\nEach kind of source has its own slots, so none crowds out another: the manual (knowledge/structured), the course (lessons and the recordings they link), calls, videos, texts. Within a kind, a passage holding the rarer of the pattern's alternatives ranks first. Each hit gives path:line, its header, its label, the source URL and the lines around it; the files past the slots follow as path:line, for `read`. A pattern hitting most of the scope is refused, with the sub-scopes it hits.\n\nA label is who backs the passage, how they know (ran it, says so, heard it), when the information is from, and `effective`: what that is worth today, 0 to 1, higher is better. `unvetted`: nobody of ours has checked the claim. `unscored`: nothing says who speaks.".to_owned(),
		),
		(
			"read",
			"One chapter of a capture or a digest (a `###`), or one section of a manual page, whole, with every line numbered.\n\nNever a whole file.".to_owned(),
		),
		("guide", guide),
	];
	writeln!(out, "pub(crate) static DOCS: ev_lib::api::Docs = &[").unwrap();
	for (handler, doc) in docs {
		writeln!(out, "\t(\"playbook_web::tools::{handler}\", {doc:?}),").unwrap();
	}
	out.push_str("];\n");

	let cited = |line: usize, b: &corpus::Backing| format!("Cited {{ line: {line}, who: {:?}, how: {:?}, trust: {:?}, day: {} }}", b.who, b.how, b.trust, b.day);
	out.push_str("pub static CLAIMS: &[(&str, &[Claim])] = &[\n");
	for (path, claims) in index.claims.iter().filter(|(p, _)| files.iter().any(|f| f.ends_with(p.as_str()))) {
		writeln!(out, "\t({path:?}, &[").unwrap();
		for c in claims {
			let backing: Vec<String> = c.backing.iter().map(|(line, b)| cited(*line, b)).collect();
			writeln!(out, "\t\tClaim {{ from: {}, to: {}, vetted: {}, backing: &[{}] }},", c.lines.start, c.lines.end, c.vetted, backing.join(", ")).unwrap();
		}
		out.push_str("\t]),\n");
	}
	out.push_str("];\npub static SPEAKERS: &[(&str, Cited)] = &[\n");
	for (path, capture) in &index.captures {
		if let Some(s) = &capture.speaker {
			writeln!(out, "\t({path:?}, {}),", cited(0, s)).unwrap();
		}
	}
	out.push_str("];\n");

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

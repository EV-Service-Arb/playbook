//! Compiles the corpus and bakes it and the connect pages' stylesheet into the binary: the server reads nothing
//! from disk per request, and the image carries only what is listed here.

use std::{
	fmt::Write,
	fs,
	path::{Path, PathBuf},
	process::Command,
};

use sha2::{Digest, Sha256};

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
	writeln!(
		out,
		"pub const GUIDE_DESCRIPTION: &str = {:?};",
		format!(
			"One page of the manual, whole: what we do in an area, each step or decision in the order it is met, and the evidence under it, every citation given as who, how they know, when, and what that is worth today. `people`: who has run what, and how to reach them. The pages:\n{}",
			pages.join("\n")
		)
	)
	.unwrap();

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
	stylesheet(&root.join("playbook_web/src/pages.rs"), &out_dir);
}

/// Tailwind over the kit's class inventory and `pages.rs`, under a name that changes with it, so it is cached for good.
/// The inventory and tokens are written out because tailwind can neither scan nor `@import` a crate unpacked from crates.io.
fn stylesheet(pages: &Path, out: &Path) {
	println!("cargo:rerun-if-changed={}", pages.display());
	fs::write(out.join("uikit-classes.txt"), ev_lib_classes::CLASS_INVENTORY).unwrap();
	fs::write(out.join("tokens.css"), ev_lib_classes::TOKENS_CSS).unwrap();
	let entry = format!(
		"@import \"tailwindcss\";\n@import \"./tokens.css\";\n@source \"./uikit-classes.txt\";\n@source {:?};\n",
		pages.display().to_string()
	);
	fs::write(out.join("entry.css"), entry).unwrap();
	let css = out.join("connect.css");
	let status = Command::new("tailwindcss")
		.arg("-i")
		.arg(out.join("entry.css"))
		.arg("-o")
		.arg(&css)
		.arg("--minify")
		.status()
		.expect("tailwindcss (v4) is on PATH: the dev shell and the nix build carry it");
	assert!(status.success(), "tailwindcss failed on {}", out.join("entry.css").display());
	let hash = Sha256::digest(fs::read(&css).unwrap());
	let hash: String = hash[..8].iter().map(|b| format!("{b:02x}")).collect();
	fs::write(
		out.join("stylesheet.rs"),
		format!(
			"pub(crate) const CSS: &str = include_str!({:?});\npub(crate) const CSS_FILE: &str = \"connect.{hash}.css\";\n",
			css.display().to_string()
		),
	)
	.unwrap();
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

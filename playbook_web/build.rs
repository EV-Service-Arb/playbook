//! Bakes the corpus and the connect pages' stylesheet into the binary: the server reads nothing from
//! disk per request, and the image carries only what is listed here (docs/ARCHITECTURE.md, Invariants).

use std::{
	fmt::Write,
	fs,
	path::{Path, PathBuf},
	process::Command,
};

use sha2::{Digest, Sha256};

const INSTRUCTIONS_MAX: usize = 2048; // Claude Code truncates server instructions past this

fn main() {
	let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).parent().expect("playbook_web/ sits in the workspace root").to_path_buf();
	let (files, decay_days) = playbook::tree(&root);
	let mut dirs: Vec<&str> = files.iter().map(|(p, _)| p.split('/').next().unwrap()).collect();
	dirs.dedup();
	for dir in dirs.into_iter().chain(["scripts/cite-check.rs"]) {
		println!("cargo:rerun-if-changed={}", root.join(dir).display());
	}
	let skill = root.join("skill/service-arb/SKILL.md");
	println!("cargo:rerun-if-changed={}", skill.display());

	let mut out = String::from("pub static FILES: &[(&str, &str)] = &[\n");
	let mut sections = Vec::new();
	for (rel, _) in &files {
		if let Some(name) = rel.strip_prefix("skill/service-arb/sections/") {
			sections.push(name.trim_end_matches(".md").to_owned());
		}
		writeln!(out, "\t({rel:?}, include_str!({:?})),", root.join(rel).display().to_string()).unwrap();
	}
	out.push_str("];\n");

	// the image carries no scripts, so the decay the skill links to is inlined
	writeln!(out, "pub const DECAY_DAYS: f64 = {decay_days:?};").unwrap();
	let skill = fs::read_to_string(&skill).unwrap();
	let instructions = format!("{}\n\n{}", section(&skill, "# service-arb"), section(&skill, "## Answering"));
	let decay_link = "[`DECAY_DAYS`](../../scripts/cite-check.rs)";
	assert!(instructions.contains(decay_link), "SKILL.md's `## Answering` no longer links {decay_link}");
	let instructions = instructions.replace(decay_link, &decay_days.to_string());
	assert!(instructions.len() <= INSTRUCTIONS_MAX, "SKILL.md's hook + `## Answering` is {} chars, over {INSTRUCTIONS_MAX}", instructions.len());
	writeln!(out, "pub const INSTRUCTIONS: &str = {instructions:?};").unwrap();
	writeln!(out, "pub const GUIDE_DESCRIPTION: &str = {:?};", format!(
		"How to decide, for one area: what matters, how to choose, what to do when it goes wrong. Read the matching one before advising rather than quoting. Sections: {}.",
		sections.join(", ")
	))
	.unwrap();

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
	let entry = format!("@import \"tailwindcss\";\n@import \"./tokens.css\";\n@source \"./uikit-classes.txt\";\n@source {:?};\n", pages.display().to_string());
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
		format!("pub(crate) const CSS: &str = include_str!({:?});\npub(crate) const CSS_FILE: &str = \"connect.{hash}.css\";\n", css.display().to_string()),
	)
	.unwrap();
}

/// The text under `header`, up to the next header.
fn section<'a>(md: &'a str, header: &str) -> &'a str {
	let start = md.find(&format!("\n{header}\n")).unwrap_or_else(|| panic!("SKILL.md has no `{header}`")) + header.len() + 2;
	let rest = &md[start..];
	rest[..rest.find("\n#").unwrap_or(rest.len())].trim()
}

#!/usr/bin/env -S cargo -Zscript -q
---cargo
[package]
edition = "2024"

[dependencies]
jiff = "0.2"
---

//! `./scripts/cite-check.rs [<root>]` — hold every block under `structured/` to
//! `structured/README.md`: each citation resolves to a capture in `ref/`, carries that capture's date,
//! stays inside its duration, and sits in its run by effective reliability; no bullet goes unbacked.
//! The `## claims` of every capture in `ref/` too: a topic per chapter it draws on, and one citation
//! per claim, into the capture itself. `<root>` defaults to the checkout this runs in.
//!
//! Prints every failure as `path:line: why` and exits non-zero if there was one.

use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
};

use jiff::civil::Date;

const ISSUE_COMMENT: &str = "https://github.com/Service-Arb/playbook/issues/";
const SECTIONS: [&str; 3] = ["## info", "## facts", "## plays"];
const MODES: [&str; 3] = ["tested", "claims", "heard"];
const DECAY_DAYS: f64 = 180.;
const WORST: f64 = 10.;

/// What a capture in `ref/` says about itself.
struct Capture {
	path: PathBuf,
	date: Date,
	duration: Option<u64>,
}

struct Citation {
	line: usize,
	base: u8,
	date: Date,
}

fn main() {
	let root = match std::env::args().nth(1) {
		Some(dir) => PathBuf::from(dir),
		None => repo_root(),
	};
	let today = jiff::Zoned::now().date();
	let captures = captures(&root.join("ref"));

	let mut blocks = Vec::new();
	md_files(&root.join("structured"), &mut blocks);
	blocks.retain(|p| p.file_name().expect("walked files have names") != "README.md");
	assert!(!blocks.is_empty(), "no blocks under {}/structured", root.display());

	let mut errors = Vec::new();
	for path in &blocks {
		let shown = path.strip_prefix(&root).expect("walked from root").display().to_string();
		let text = std::fs::read_to_string(path).expect("walked files are readable");
		for (line, why) in check(&text, &captures, today) {
			errors.push(format!("{shown}:{line}: {why}"));
		}
	}
	let mut digests = 0;
	for (key, capture) in &captures {
		let text = std::fs::read_to_string(&capture.path).expect("walked files are readable");
		if !text.contains("\n## claims\n") {
			continue;
		}
		digests += 1;
		let shown = capture.path.strip_prefix(&root).expect("walked from root").display().to_string();
		for (line, why) in claims(&text, key, &captures) {
			errors.push(format!("{shown}:{line}: {why}"));
		}
	}
	for e in &errors {
		eprintln!("{e}");
	}
	if !errors.is_empty() {
		eprintln!("{} citation errors across {} blocks and {digests} captures' claims", errors.len(), blocks.len());
		std::process::exit(1);
	}
	eprintln!("{} blocks and {digests} captures' claims hold their citations", blocks.len());
}

fn check(text: &str, captures: &BTreeMap<String, Capture>, today: Date) -> Vec<(usize, String)> {
	let mut errors = Vec::new();
	let mut section = None;
	let mut unbacked: Option<usize> = None;
	let mut run: Vec<Citation> = Vec::new();
	let mut fenced = false;

	for (i, raw) in text.lines().enumerate() {
		let line = i + 1;
		let trimmed = raw.trim();
		if trimmed.starts_with("```") {
			fenced = !fenced;
			continue;
		}
		if fenced {
			continue;
		}

		let cited = match link_only(trimmed) {
			Some((label, url)) => match citation(line, label, url, captures) {
				Ok(c) => {
					run.push(c);
					true
				}
				Err(why) => {
					errors.push((line, why));
					true
				}
			},
			None => false,
		};
		if !cited {
			sorted(&run, today, &mut errors);
			run.clear();
		}
		if cited || trimmed.starts_with("// unsourced:") {
			if trimmed == "// unsourced:" {
				errors.push((line, "`// unsourced:` names nobody".into()));
			}
			unbacked = None;
			continue;
		}

		if raw.starts_with('#') {
			if let Some(b) = unbacked.take() {
				errors.push((b, "bullet has no citation or `// unsourced: <who>` before the section ends".into()));
			}
			match SECTIONS.iter().position(|s| *s == raw) {
				Some(pos) if section.is_none_or(|prev| pos > prev) => section = Some(pos),
				Some(_) => errors.push((line, "sections go info, facts, plays, each at most once".into())),
				None => errors.push((line, format!("a block's only headers are {SECTIONS:?}"))),
			}
			continue;
		}
		if !trimmed.is_empty() && section.is_none() {
			errors.push((line, "text above the first section".into()));
			section = Some(0); // reported once; what follows is read as `## info`
		}
		if raw.starts_with("- ") && unbacked.is_none() {
			unbacked = Some(line);
		}
	}
	sorted(&run, today, &mut errors);
	if let Some(b) = unbacked {
		errors.push((b, "bullet has no citation or `// unsourced: <who>` before the section ends".into()));
	}
	errors
}

/// A capture's `## claims`: `### <topic> <stamp>` naming its chapters in order, under each a bullet
/// `<who>, <mode>: <claim>` per claim and its one citation, into this capture.
fn claims(text: &str, own: &str, captures: &BTreeMap<String, Capture>) -> Vec<(usize, String)> {
	let mut errors = Vec::new();
	let chapters = chapters(text);
	let mut next = 0; // the chapters a topic may still name start here
	let mut section = "";
	let mut topic = false;
	let mut bullet: Option<(usize, usize)> = None; // its line, and the citations under it so far
	let close = |bullet: &mut Option<(usize, usize)>, errors: &mut Vec<(usize, String)>| {
		if let Some((line, n)) = bullet.take().filter(|(_, n)| *n != 1) {
			errors.push((line, format!("a claim carries exactly one citation, and this one {n}")));
		}
	};
	for (i, raw) in text.lines().enumerate() {
		let line = i + 1;
		if let Some(s) = raw.strip_prefix("## ") {
			close(&mut bullet, &mut errors);
			section = s;
			continue;
		}
		if section != "claims" || raw.trim().is_empty() {
			continue;
		}
		if let Some(header) = raw.strip_prefix("### ") {
			close(&mut bullet, &mut errors);
			topic = true;
			match header.rsplit_once(' ').and_then(|(name, stamp)| Some((stamp_secs(stamp)?, name))) {
				Some((at, name)) if let Some(i) = chapters[next..].iter().position(|&(t, c)| t == at && c == name) => next += i + 1,
				Some(_) => errors.push((line, "names no chapter after the topic above it; a topic is a chapter's name and time".into())),
				None => errors.push((line, format!("`### {header}` is not `### <topic> <stamp>`"))),
			}
			continue;
		}
		if !topic {
			errors.push((line, "claims sit under a `### <topic> <stamp>`".into()));
			topic = true; // reported once
		}
		if let Some(claim) = raw.strip_prefix("- ") {
			close(&mut bullet, &mut errors);
			bullet = Some((line, 0));
			let mode = claim.split_once(": ").and_then(|(said, _)| said.rsplit_once(", ")).map(|(_, mode)| mode);
			if !mode.is_some_and(|m| MODES.contains(&m)) {
				errors.push((line, format!("a claim opens `<who>, <{}>: `", MODES.join("|"))));
			}
			continue;
		}
		match (link_only(raw.trim()), &mut bullet) {
			(Some((label, url)), Some((_, n))) => {
				*n += 1;
				if source_key(url).as_deref() != Some(own) {
					errors.push((line, format!("cites `{url}`, and a claim cites the capture it sits in")));
				} else if let Err(why) = citation(line, label, url, captures) {
					errors.push((line, why));
				}
			}
			_ => errors.push((line, "under a topic, only claims and their citations".into())),
		}
	}
	close(&mut bullet, &mut errors);
	errors
}

/// `(secs, title)` of a capture's chapters: `### [<stamp>](url) title` in a call's transcript, `- [<stamp>](url) title` in a video's `## chapters`.
fn chapters(text: &str) -> Vec<(u64, &str)> {
	let mut section = "";
	let mut out = Vec::new();
	for raw in text.lines() {
		if let Some(s) = raw.strip_prefix("## ") {
			section = s;
			continue;
		}
		let entry = match section {
			"transcript" => raw.strip_prefix("### ["),
			"chapters" => raw.strip_prefix("- ["),
			_ => None,
		};
		if let Some((stamp, rest)) = entry.and_then(|e| e.split_once("](")) {
			let at = stamp_secs(stamp).unwrap_or_else(|| panic!("chapter `{raw}` has no stamp; call-pull.rs --check or yt-pull.rs wrote it"));
			out.push((at, rest.split_once(") ").map_or("", |(_, title)| title.trim())));
		}
	}
	out
}

/// `1:02:27`, `03:34` or `0:00` as seconds.
fn stamp_secs(stamp: &str) -> Option<u64> {
	stamp.split(':').try_fold(0, |acc, p| Some(acc * 60 + p.parse::<u64>().ok()?))
}

/// `[label](url)` and nothing else on the line.
fn link_only(line: &str) -> Option<(&str, &str)> {
	let rest = line.strip_prefix('[')?.strip_suffix(')')?;
	let (label, url) = rest.split_once("](")?;
	(!label.contains(']') && !url.contains(' ')).then_some((label, url))
}

fn citation(line: usize, label: &str, url: &str, captures: &BTreeMap<String, Capture>) -> Result<Citation, String> {
	let parsed = label
		.strip_prefix('r')
		.and_then(|l| l.split_once(' '))
		.and_then(|(b, d)| Some((b.parse::<u8>().ok().filter(|b| *b <= 7)?, d.parse::<Date>().ok()?)));
	let Some((base, date)) = parsed else {
		return Err(format!("a link alone on a line is a citation, and `{label}` is not `r<0-7> <YYYY-MM-DD>`"));
	};

	if url.starts_with(ISSUE_COMMENT) {
		// a test's result lives on github and not in ref/, so there is no capture to hold its date to
		if !url.contains("#issuecomment-") {
			return Err("an `r0` cites the issue comment that closed the test, not the issue".into());
		}
		if base != 0 {
			return Err(format!("an issue comment is a test of our own, so `r0`, not `r{base}`"));
		}
		return Ok(Citation { line, base, date });
	}
	if base == 0 {
		return Err(format!("`r0` is a test of our own, cited by its comment under {ISSUE_COMMENT}"));
	}

	let key = source_key(url).ok_or_else(|| format!("`{url}` is no source this checker knows"))?;
	let capture = captures.get(&key).ok_or_else(|| format!("`{url}` has no capture in ref/"))?;
	if key.starts_with("skool:") && base != 1 {
		return Err(format!("skool course material is `r1`, not `r{base}`"));
	}
	if date != capture.date {
		return Err(format!("dated {date}, but the capture is dated {}", capture.date));
	}
	if let Some(t) = param(url, "t").or_else(|| url.split_once("#t=").map(|(_, t)| t)) {
		let t: u64 = t.trim_end_matches('s').parse().map_err(|_| format!("`t={t}` is not a second"))?;
		match capture.duration {
			Some(d) if t > d => return Err(format!("`t={t}` is past the capture's end at {d}s")),
			Some(_) => {}
			None => return Err("`t=` on a capture that has no duration".into()),
		}
	}
	Ok(Citation { line, base, date })
}

/// A run leads with its best effective reliability, and a tie goes to the newer one.
fn sorted(run: &[Citation], today: Date, errors: &mut Vec<(usize, String)>) {
	let effective = |c: &Citation| {
		let age = (today - c.date).get_days() as f64;
		assert!(age >= 0., "cited {} is in the future", c.date);
		(c.base as f64 + age / DECAY_DAYS).min(WORST)
	};
	for pair in run.windows(2) {
		let (a, b) = (&pair[0], &pair[1]);
		let order = effective(a).total_cmp(&effective(b)).then(b.date.cmp(&a.date));
		if order.is_gt() {
			errors.push((b.line, format!("outranks the citation above it ({:.2} vs {:.2} effective)", effective(b), effective(a))));
		}
	}
}

/// The identity a url shares with the `source:` of its capture.
fn source_key(url: &str) -> Option<String> {
	let after = |prefix: &str| url.split_once(prefix).map(|(_, rest)| rest.split(['?', '#', '&', '/']).next().expect("split yields one").to_string());
	if let Some(id) = after("loom.com/share/") {
		return Some(format!("loom:{id}"));
	}
	if let Some(id) = after("fathom.video/share/") {
		return Some(format!("fathom:{id}"));
	}
	if let Some(id) = after("chatgpt.com/share/") {
		return Some(format!("chatgpt:{id}"));
	}
	if let Some(id) = after("drive.google.com/file/d/") {
		return Some(format!("drive:{id}"));
	}
	if let Some(id) = after("player.vimeo.com/video/") {
		return Some(format!("vimeo:{id}"));
	}
	if let Some(id) = after("vocaroo.com/") {
		return Some(format!("vocaroo:{id}"));
	}
	if let Some(id) = after("youtu.be/") {
		return Some(format!("youtube:{id}"));
	}
	if url.contains("youtube.com/watch") {
		return Some(format!("youtube:{}", param(url, "v")?));
	}
	if url.contains("skool.com/") && url.contains("/classroom") {
		return Some(format!("skool:{}", param(url, "md")?));
	}
	None
}

fn param<'a>(url: &'a str, name: &str) -> Option<&'a str> {
	let query = url.split_once('?')?.1.split('#').next().expect("split yields one");
	query.split('&').find_map(|kv| kv.strip_prefix(name)?.strip_prefix('='))
}

/// Every capture under `ref/` that states a source this checker can cite, by that source.
fn captures(ref_dir: &Path) -> BTreeMap<String, Capture> {
	let mut files = Vec::new();
	md_files(ref_dir, &mut files);
	let mut out = BTreeMap::new();
	for path in files {
		let text = std::fs::read_to_string(&path).expect("walked files are readable");
		let field = |name: &str| text.lines().find_map(|l| l.strip_prefix("- ")?.strip_prefix(name)?.strip_prefix(": "));
		let Some(key) = field("source").and_then(|s| source_key(s.trim_matches(['<', '>']))) else {
			continue; // a registry or index file, which states no citable source
		};
		let stamp = ["recorded", "uploaded", "updated", "pulled"]
			.iter()
			.find_map(|f| field(f))
			.unwrap_or_else(|| panic!("{} states a source but no date", path.display()));
		let date: Date = stamp[..10].parse().unwrap_or_else(|e| panic!("{}: `{stamp}` does not open on a date: {e}", path.display()));
		let duration = field("duration").map(|d| {
			let parts: Vec<u64> = d.split(':').map(|p| p.parse().unwrap_or_else(|e| panic!("{}: duration `{d}`: {e}", path.display()))).collect();
			parts.iter().fold(0, |acc, p| acc * 60 + p)
		});
		if out.insert(key.clone(), Capture { path: path.clone(), date, duration }).is_some() {
			panic!("{key} is captured twice, second at {}", path.display());
		}
	}
	out
}

fn md_files(dir: &Path, out: &mut Vec<PathBuf>) {
	for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
		let path = entry.expect("dir entries are readable").path();
		if path.is_dir() {
			md_files(&path, out);
		} else if path.extension().is_some_and(|e| e == "md") {
			out.push(path);
		}
	}
	out.sort();
}

fn repo_root() -> PathBuf {
	let mut dir = std::env::current_dir().expect("a process has a cwd");
	loop {
		if dir.join(".git").exists() {
			return dir;
		}
		if !dir.pop() {
			panic!("run this from inside the playbook checkout");
		}
	}
}

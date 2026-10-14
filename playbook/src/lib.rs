//! The playbook's search, read and guide, over `skill/`, `structured/` and `ref/`: baked into the
//! member server, and read off the checkout by the `playbook` binary, so both rank the same way.

use std::{cmp::Reverse, fmt::Write, fs, path::Path, sync::LazyLock};

use regex::{Regex, RegexBuilder};
use regex_syntax::ast::{Ast, parse::Parser};

const PER_SOURCE: usize = 2;
const CHUNK: usize = 2_000; // chars of blocks a capture's chunk packs, so a long chapter is not one bag of words
const MIN_MATCH: usize = 2; // a lone letter is noise, not relevance
const BROAD: f64 = 0.4; // a pattern hitting this share of the files in scope is an enumeration
const OVERFLOW: usize = 5; // files per kind listed past its slots, for the agent to `read`
const WORST: f64 = 10.;
const LINE_CAP: usize = 700;
const STRUCTURED_LINES: usize = 12;
const REF_BLOCKS: usize = 2; // paragraphs or bullets shown per chunk
const BLOCK_LINES: usize = 8;
const CHAPTER_CAP: usize = 24_000;

/// how many files of which kinds one search shows: reliability decides across kinds, relevance within one
const SLOTS: [(&[Kind], usize); 5] = [(&[Kind::Approved, Kind::Suggested], 6), (&[Kind::Course], 2), (&[Kind::Call], 3), (&[Kind::Video], 2), (&[Kind::Text], 1)];

static CITATION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*\[r(\d) (\d{4}-\d{2}-\d{2})\]\([^)]+\)\s*$").unwrap());

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
	Approved,
	Suggested,
	/// a lesson, or a capture a lesson links
	Course,
	/// a recording of speech other than a video
	Call,
	Video,
	Text,
}

struct Doc {
	path: &'static str,
	kind: Kind,
	lines: Vec<&'static str>,
	/// per line, the index of the header it sits under
	chapter: Vec<Option<usize>>,
	/// per line, where the chunk `search` scores starts: a bullet of a note, blocks of a capture's chapter
	chunk: Vec<usize>,
	/// per line, whether it reads what a recording showed rather than what was said
	screen: Vec<bool>,
	/// per line, the `## ` section it sits in
	section: Vec<&'static str>,
	/// a capture's own date; structured notes date each claim instead
	date: Option<&'static str>,
	source: Option<&'static str>,
}

pub struct Corpus {
	docs: Vec<Doc>,
	sections: Vec<(&'static str, &'static str)>,
	decay_days: f64,
}

pub struct Found {
	pub text: String,
	pub ids: Vec<String>,
}

/// `(path, text)` of every file the corpus reads under `root`, and cite-check.rs's `DECAY_DAYS`.
pub fn tree(root: &Path) -> (Vec<(String, String)>, f64) {
	// `README.md`s are indices and format rules, not knowledge. `ref/cases/` states no source and no date
	fn walk(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
		for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
			let path = entry.unwrap().path();
			if path.ends_with("ref/cases") {
				continue;
			}
			if path.is_dir() {
				walk(&path, files);
			} else if path.extension().is_some_and(|e| e == "md") && path.file_name().unwrap() != "README.md" {
				files.push(path);
			}
		}
	}
	let mut paths = Vec::new();
	for dir in ["skill/service-arb/sections", "structured/approved", "structured/suggested", "ref"] {
		walk(&root.join(dir), &mut paths);
	}
	paths.sort();
	let files = paths
		.into_iter()
		.map(|p| {
			let text = fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
			(p.strip_prefix(root).unwrap().to_str().expect("corpus paths are utf-8").to_owned(), text)
		})
		.collect();
	let cite_check = fs::read_to_string(root.join("scripts/cite-check.rs")).expect("the checkout holds scripts/cite-check.rs");
	let decay_days = cite_check
		.lines()
		.find_map(|l| l.strip_prefix("const DECAY_DAYS: f64 = ")?.strip_suffix(".;"))
		.expect("cite-check.rs states `const DECAY_DAYS: f64 = <n>.;`")
		.parse()
		.expect("an integer, per the pattern above");
	(files, decay_days)
}

impl Corpus {
	/// `files` as `tree` reads them
	pub fn new(files: &[(&'static str, &'static str)], decay_days: f64) -> Self {
		let sections = files.iter().filter_map(|&(p, t)| Some((p.strip_prefix("skill/service-arb/sections/")?.strip_suffix(".md")?, t))).collect();
		let mut docs: Vec<Doc> = files
			.iter()
			.filter(|(p, _)| !p.starts_with("skill/"))
			.map(|&(path, text)| {
				let lines: Vec<&str> = text.lines().collect();
				let kind = match path {
					p if p.starts_with("structured/approved/") => Kind::Approved,
					p if p.starts_with("structured/suggested/") => Kind::Suggested,
					p if p.starts_with("ref/skool_") && p.contains("/course/") => Kind::Course,
					p if p.starts_with("ref/youtube/") => Kind::Video,
					p if p.starts_with("ref/") && lines.contains(&"## transcript") => Kind::Call,
					p if p.starts_with("ref/") => Kind::Text,
					p => unreachable!("`tree` reads only structured/ and ref/ besides skill/: {p}"),
				};
				let mut current = None;
				let chapter = lines
					.iter()
					.enumerate()
					.map(|(i, l)| {
						if l.starts_with('#') {
							current = Some(i);
						}
						current
					})
					.collect();
				let mut current = "";
				let section: Vec<&str> = lines
					.iter()
					.map(|l| {
						if l.starts_with("# ") || l.starts_with("## ") {
							current = l;
						}
						current
					})
					.collect();
				let screen = section.iter().map(|s| path.ends_with("/shown.md") || *s == "## shown").collect();
				let chunk = match kind {
					Kind::Approved | Kind::Suggested => {
						let mut current = 0;
						(0..lines.len())
							.map(|i| {
								if lines[i].starts_with('#') || lines[i].starts_with("- ") {
									current = i;
								}
								current
							})
							.collect()
					}
					_ => {
						// a claim of the digest is a chunk to itself, like a note's bullet
						let mut chunk = chunks(&lines);
						let mut current = 0;
						for i in (0..lines.len()).filter(|&i| section[i] == "## claims") {
							if lines[i].starts_with('#') || lines[i].starts_with("- ") {
								current = i;
							}
							chunk[i] = current;
						}
						chunk
					}
				};
				let meta = |key: &str| lines.iter().find_map(|l| l.strip_prefix(key));
				let date = ["- recorded: ", "- uploaded: ", "- updated: ", "- pulled: "].iter().find_map(|k| meta(k)).map(|d| &d[..10]);
				let source = meta("- source: ").map(|s| s.trim_matches(['<', '>']));
				Doc { path, kind, lines, chapter, chunk, screen, section, date, source }
			})
			.collect();
		let lessons: Vec<&str> = files.iter().filter(|(p, _)| p.starts_with("ref/skool_") && p.contains("/course/")).map(|(_, t)| *t).collect();
		for d in docs.iter_mut().filter(|d| matches!(d.kind, Kind::Call | Kind::Video | Kind::Text)) {
			let within = d.lines.iter().find_map(|l| l.strip_prefix("- within: <")?.split_once('>')).map(|(u, _)| u);
			let linked = |url: &str| {
				let url = url.split('?').next().unwrap();
				lessons.iter().any(|t| t.contains(url))
			};
			if d.source.into_iter().chain(within).any(linked) {
				d.kind = Kind::Course;
			}
		}
		// `shown.md` describes its recording, so it carries the recording's kind, date and source
		for i in 0..docs.len() {
			if let Some(dir) = docs[i].path.strip_suffix("/shown.md") {
				let capture = format!("{dir}.md");
				let parent = docs.iter().find(|d| d.path == capture).unwrap_or_else(|| panic!("{} has no capture beside it", docs[i].path));
				(docs[i].kind, docs[i].date, docs[i].source) = (parent.kind, parent.date, parent.source);
			}
		}
		for d in docs.iter().filter(|d| d.kind > Kind::Suggested) {
			let date = d.date.unwrap_or_else(|| panic!("{} states no date", d.path));
			assert!(date.as_bytes()[4] == b'-' && date.as_bytes()[7] == b'-', "{}: `{date}` is not a date", d.path);
			if d.kind == Kind::Video {
				let channel = d.path.split('/').nth(2).unwrap();
				assert!(channel == "eric", "ref/youtube/{channel}: no base reliability for the channel; give it a row in docs/ARCHITECTURE.md#reliability and arms in `Doc::reliability` and `Doc::effective`");
			}
		}
		Self { docs, sections, decay_days }
	}

	pub fn search(&self, pattern: &str, scope: Option<&str>, screen: bool) -> Result<Found, String> {
		let compile = |p: &str| RegexBuilder::new(p).case_insensitive(true).size_limit(1 << 20).build().map_err(|e| e.to_string());
		let re = compile(pattern)?;
		if re.is_match("") {
			return Err("the pattern matches the empty string. Search for words.".into());
		}
		// each alternated term is scored on its own, so a chunk holding a rare one outranks one holding many common ones
		let branches: Vec<Regex> = match &Parser::new().parse(pattern).map_err(|e| e.to_string())? {
			Ast::Alternation(alt) => alt.asts.iter().map(|a| compile(&a.to_string())).collect::<Result<_, _>>()?,
			_ => vec![re.clone()],
		};
		let scope = scope.unwrap_or("");
		if !(scope.is_empty() || scope.starts_with("structured") || scope.starts_with("ref")) {
			return Err("scope is a path prefix under `structured/` or `ref/`".into());
		}
		let in_scope: Vec<&Doc> = self.docs.iter().filter(|d| d.path.starts_with(scope) && (0..d.lines.len()).any(|i| d.screen[i] == screen)).collect();
		if in_scope.is_empty() {
			return Err(match screen {
				true => format!("nothing under `{scope}` reads a screen"),
				false => format!("nothing sits under `{scope}`"),
			});
		}

		struct Hit {
			start: usize,
			/// every matching line of the chunk, with how many branches it holds
			lines: Vec<(usize, usize)>,
			branches: Vec<bool>,
			score: f64,
		}
		struct File<'a> {
			doc: &'a Doc,
			hits: Vec<Hit>,
			score: f64,
			effective: f64,
		}
		let mut files: Vec<File> = Vec::new();
		let mut candidates = 0;
		for &doc in &in_scope {
			candidates += (0..doc.lines.len()).filter(|&i| doc.chunk[i] == i && doc.screen[i] == screen).count();
			let mut hits: Vec<Hit> = Vec::new();
			for (i, line) in doc.lines.iter().enumerate().filter(|&(i, _)| doc.screen[i] == screen) {
				if !re.find_iter(line).any(|m| m.len() >= MIN_MATCH) {
					continue;
				}
				let on_line: Vec<bool> = branches.iter().map(|b| b.find_iter(line).any(|m| m.len() >= MIN_MATCH)).collect();
				let hit = match hits.iter().position(|h| h.start == doc.chunk[i]) {
					Some(h) => &mut hits[h],
					None => {
						hits.push(Hit { start: doc.chunk[i], lines: vec![], branches: vec![false; branches.len()], score: 0. });
						hits.last_mut().unwrap()
					}
				};
				hit.lines.push((i, on_line.iter().filter(|b| **b).count()));
				hit.branches.iter_mut().zip(on_line).for_each(|(h, l)| *h |= l);
			}
			if !hits.is_empty() {
				files.push(File { doc, hits, score: 0., effective: 0. });
			}
		}
		if in_scope.len() >= 5 && files.len() as f64 > in_scope.len() as f64 * BROAD {
			return Err(format!("the pattern hits {} of the {} files in scope. Narrow it.", files.len(), in_scope.len()));
		}
		if files.is_empty() {
			return Ok(Found {
				text: "no match. If rephrasing finds nothing either, the corpus does not cover this: say so.".into(),
				ids: vec![],
			});
		}

		// BM25's idf with term frequency saturated to whether the term is there: no reward for length or repetition
		let n = candidates as f64;
		let idf: Vec<f64> = (0..branches.len())
			.map(|b| {
				let df = files.iter().flat_map(|f| &f.hits).filter(|h| h.branches[b]).count() as f64;
				(1. + (n - df + 0.5) / (df + 0.5)).ln()
			})
			.collect();
		let today = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64 / 86_400;
		for f in &mut files {
			for h in &mut f.hits {
				h.score = h.branches.iter().zip(&idf).filter(|(b, _)| **b).map(|(_, idf)| idf).sum();
			}
			f.score = f.hits.iter().map(|h| h.score).fold(0., f64::max);
			// the digest's claims, then its summary, before the words as said: a claim carries its own second
			let doc = f.doc;
			f.hits.sort_by(|a, b| doc.digested(a.start).cmp(&doc.digested(b.start)).then(b.score.total_cmp(&a.score)));
			f.hits.truncate(PER_SOURCE);
			f.effective = f.doc.effective(f.hits[0].start, today, self.decay_days);
		}
		files.sort_by(|a, b| {
			a.doc.kind.cmp(&b.doc.kind)
				.then(b.score.total_cmp(&a.score))
				.then(a.effective.total_cmp(&b.effective))
				.then(b.doc.date.cmp(&a.doc.date))
		});

		let mut groups: Vec<(Vec<&File>, usize)> = SLOTS.iter().map(|(kinds, slots)| (files.iter().filter(|f| kinds.contains(&f.doc.kind)).collect(), *slots)).collect();
		// slots a ref kind leaves empty go to the others, the more reliable kind first
		let mut spare: usize = groups[1..].iter().map(|(of_kind, slots)| slots.saturating_sub(of_kind.len())).sum();
		for (of_kind, slots) in &mut groups[1..] {
			let more = spare.min(of_kind.len().saturating_sub(*slots));
			(*slots, spare) = (*slots + more, spare - more);
		}
		let (mut shown, mut overflow, mut dropped): (Vec<&File>, Vec<&File>, usize) = (Vec::new(), Vec::new(), 0);
		for (of_kind, slots) in groups {
			shown.extend(of_kind.iter().take(slots));
			overflow.extend(of_kind.iter().skip(slots).take(OVERFLOW));
			dropped += of_kind.len().saturating_sub(slots + OVERFLOW);
		}
		// structured and the course lead; the rest follow by what their kind is worth
		let leading = shown.iter().take_while(|f| f.doc.kind <= Kind::Course).count();
		shown[leading..].sort_by(|a, b| a.effective.total_cmp(&b.effective).then(b.doc.date.cmp(&a.doc.date)));

		let mut text = String::new();
		let mut ids = Vec::new();
		for file in shown {
			let doc = file.doc;
			for hit in &file.hits {
				let first = hit.lines[0].0;
				let end = doc.chapter_end(first);
				let blocks: Vec<(usize, usize)> = match doc.kind {
					Kind::Approved | Kind::Suggested => {
						let from = if doc.lines[hit.start].starts_with('#') { first } else { hit.start };
						let to = (from + 1..end).find(|&i| doc.chunk[i] != hit.start).unwrap_or(end);
						vec![(from, to.min(from + STRUCTURED_LINES))]
					}
					_ => {
						let mut body: Vec<(usize, usize)> = hit.lines.iter().copied().filter(|(l, _)| !doc.lines[*l].starts_with('#')).collect();
						body.sort_by_key(|&(l, branches)| (Reverse(branches), l));
						let mut lines: Vec<usize> = body.iter().take(REF_BLOCKS).map(|&(l, _)| l).collect();
						if lines.is_empty() {
							// a chunk that matched only in its header shows its opening paragraph
							lines.push((first..end).find(|&i| !doc.lines[i].starts_with('#') && !doc.lines[i].trim().is_empty()).unwrap_or(first));
						}
						let mut blocks: Vec<(usize, usize)> = lines.into_iter().map(|l| doc.block(l, end)).collect();
						blocks.sort();
						blocks.dedup();
						blocks
					}
				};
				let at = blocks[0].0;
				writeln!(text, "{}:{} · {} · {}", doc.path, at + 1, doc.header(first), doc.reliability(first)).unwrap();
				if let Some(source) = doc.source {
					writeln!(text, "source: {source}").unwrap();
				}
				for (from, to) in blocks {
					for i in from..to {
						writeln!(text, "{:>5} | {}", i + 1, clip(doc.lines[i], &re)).unwrap();
					}
				}
				text.push('\n');
				ids.push(format!("{}:{}", doc.path, at + 1));
			}
		}
		if !overflow.is_empty() {
			writeln!(text, "also matched, best first within each kind; `read` what the answer needs:").unwrap();
			for f in overflow {
				let first = f.hits[0].lines[0].0;
				writeln!(text, "{}:{} · {} · {}", f.doc.path, first + 1, f.doc.header(first), f.doc.reliability(first)).unwrap();
			}
		}
		if dropped > 0 {
			writeln!(text, "{dropped} more files matched. Narrow the pattern, or pass a `scope`, to reach them.").unwrap();
		}
		Ok(Found { text, ids })
	}

	pub fn read(&self, path: &str, header: &str, line: Option<usize>) -> Result<Found, String> {
		let doc = self.docs.iter().find(|d| d.path == path).ok_or_else(|| format!("no file `{path}`; take paths from `search`"))?;
		let start = match line {
			Some(l) => {
				let l = l.checked_sub(1).filter(|l| *l < doc.lines.len()).ok_or_else(|| format!("{path} has {} lines", doc.lines.len()))?;
				doc.chapter[l].ok_or_else(|| format!("line {} of {path} sits above its first header", l + 1))?
			}
			None => {
				let want = header.to_lowercase();
				let found: Vec<usize> = (0..doc.lines.len()).filter(|&i| doc.lines[i].starts_with('#') && doc.lines[i].to_lowercase().contains(&want)).collect();
				match found[..] {
					[one] => one,
					[] => return Err(format!("no header in {path} holds `{header}`. Its headers:\n{}", doc.headers())),
					_ => {
						let list: Vec<String> = found.iter().map(|&i| format!("line {}: {}", i + 1, doc.lines[i])).collect();
						return Err(format!("`{header}` is ambiguous in {path}; pass the `line` of one:\n{}", list.join("\n")));
					}
				}
			}
		};
		let end = doc.chapter_end(start);
		let mut text = format!("{path}:{} · {}\n", start + 1, doc.reliability(start));
		if let Some(source) = doc.source {
			writeln!(text, "source: {source}").unwrap();
		}
		for i in start..end {
			writeln!(text, "{:>5} | {}", i + 1, doc.lines[i]).unwrap();
			if text.len() > CHAPTER_CAP {
				writeln!(text, "… the chapter runs to line {end}. `search` inside it, with scope `{path}`, for the rest.").unwrap();
				break;
			}
		}
		Ok(Found { text, ids: vec![format!("{path}:{}", start + 1)] })
	}

	pub fn guide(&self, section: &str) -> Result<Found, String> {
		self.sections
			.iter()
			.find(|(s, _)| *s == section)
			.map(|(_, text)| Found { text: text.to_string(), ids: vec![section.to_owned()] })
			.ok_or_else(|| format!("no section `{section}`. Sections: {}.", self.sections.iter().map(|(s, _)| *s).collect::<Vec<_>>().join(", ")))
	}
}

impl Doc {
	/// the line after the last one of the chapter holding `line`
	fn chapter_end(&self, line: usize) -> usize {
		(line + 1..self.lines.len()).find(|&i| self.lines[i].starts_with('#')).unwrap_or(self.lines.len())
	}

	/// the paragraph or bullet holding `line`, within a chapter ending at `end`
	fn block(&self, line: usize, end: usize) -> (usize, usize) {
		let opens = |i: usize| self.lines[i].starts_with("- ") || self.lines[i - 1].trim().is_empty() || self.lines[i - 1].starts_with('#');
		let from = (1..=line).rev().find(|&i| opens(i)).unwrap_or(0);
		let to = (line + 1..end).find(|&i| self.lines[i].trim().is_empty() || self.lines[i].starts_with("- ") || self.lines[i].starts_with('#')).unwrap_or(end);
		(from, to.min(from + BLOCK_LINES).max(line + 1))
	}

	fn headers(&self) -> String {
		self.lines.iter().enumerate().filter(|(_, l)| l.starts_with('#')).map(|(i, l)| format!("line {}: {l}", i + 1)).collect::<Vec<_>>().join("\n")
	}

	/// how far from the digest `line` sits: its claims, its summary, the rest, then the transcript
	fn digested(&self, line: usize) -> u8 {
		match self.section[line] {
			"## claims" => 0,
			"## summary" => 1,
			"## transcript" => 3,
			_ => 2,
		}
	}

	fn header(&self, line: usize) -> &str {
		self.chapter[line].map_or("", |c| self.lines[c])
	}

	/// what a claim at `line` is worth, and when it was said
	fn reliability(&self, line: usize) -> String {
		let date = || self.date.expect("asserted on load");
		match self.kind {
			Kind::Approved | Kind::Suggested => self.citation(line).map_or("unsourced".into(), |(r, d)| format!("r{r} {d}")),
			_ if let Some((r, d)) = self.claim(line) => format!("r{r} {d}"),
			_ if self.screen[line] => format!("a reading of the screen · {}", date()),
			Kind::Course => format!("r1 course · {}", date()),
			Kind::Video => format!("r2 Eric · {}", date()),
			Kind::Call => format!("r2–7 by who speaks · {}", date()),
			Kind::Text => format!("unrated · {}", date()),
		}
	}

	fn citation(&self, line: usize) -> Option<(u8, &'static str)> {
		(line..self.chapter_end(line)).find_map(|i| CITATION.captures(self.lines[i])).map(|c| (c[1].parse().unwrap(), c.get(2).unwrap().as_str()))
	}

	/// the citation of the digest's claim at `line`
	fn claim(&self, line: usize) -> Option<(u8, &'static str)> {
		let bullet = self.section[line] == "## claims" && self.lines[line].starts_with("- ");
		bullet.then(|| self.citation(line).unwrap_or_else(|| panic!("{}:{}: a claim with no citation; run cite-check.rs", self.path, line + 1)))
	}

	/// docs/ARCHITECTURE.md#reliability: base plus age, lower is better; a call is held at its best speaker's base
	fn effective(&self, line: usize, today: i64, decay_days: f64) -> f64 {
		let (base, date) = match self.kind {
			Kind::Approved | Kind::Suggested => match self.citation(line) {
				Some((r, d)) => (r as f64, d),
				None => return WORST,
			},
			_ if let Some((r, d)) = self.claim(line) => (r as f64, d),
			Kind::Course => (1., self.date.unwrap()),
			Kind::Video | Kind::Call => (2., self.date.unwrap()),
			Kind::Text => return WORST,
		};
		(base + (today - days(date)) as f64 / decay_days).min(WORST)
	}
}

/// Packs a capture's blocks (paragraphs, bullets) into chunks of at most `CHUNK` chars; a header always
/// opens one, and a block over `CHUNK` stands alone.
fn chunks(lines: &[&str]) -> Vec<usize> {
	let opens = |i: usize| i == 0 || !lines[i].trim().is_empty() && (lines[i].starts_with('#') || lines[i].starts_with("- ") || lines[i - 1].trim().is_empty() || lines[i - 1].starts_with('#'));
	let blocks: Vec<usize> = (0..lines.len()).filter(|&i| opens(i)).chain([lines.len()]).collect();
	let mut chunk = vec![0; lines.len()];
	let (mut start, mut len, mut count) = (0, 0, 0);
	for w in blocks.windows(2) {
		let (from, to) = (w[0], w[1]);
		let size: usize = lines[from..to].iter().map(|l| l.len() + 1).sum();
		if lines[from].starts_with('#') || len > 0 && len + size > CHUNK {
			(start, len, count) = (from, 0, 0);
		}
		(len, count) = (len + size, count + 1);
		assert!(len <= CHUNK || count == 1, "the chunk at line {} runs past CHUNK over several blocks", start + 1);
		chunk[from..to].fill(start);
	}
	chunk
}

/// days since 1970-01-01 of a `YYYY-MM-DD`, after Howard Hinnant's `days_from_civil`
fn days(date: &str) -> i64 {
	let n = |r: std::ops::Range<usize>| date[r].parse::<i64>().expect("dates are asserted on load");
	let (m, d) = (n(5..7), n(8..10));
	let y = n(0..4) - (m <= 2) as i64;
	let era = y.div_euclid(400);
	let yoe = y - era * 400;
	let doy = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
	era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
}

/// A transcript paragraph is one line and can run to kilobytes; keep the part around the match.
fn clip(line: &str, re: &Regex) -> String {
	if line.len() <= LINE_CAP {
		return line.to_owned();
	}
	let (from, to) = match re.find(line) {
		Some(m) => (m.start().saturating_sub(LINE_CAP / 2), (m.end() + LINE_CAP / 2).min(line.len())),
		None => (0, LINE_CAP),
	};
	let from = (0..=from).rev().find(|&i| line.is_char_boundary(i)).unwrap();
	let to = (to..=line.len()).find(|&i| line.is_char_boundary(i)).unwrap();
	format!("{}{}{}", if from > 0 { "…" } else { "" }, &line[from..to], if to < line.len() { "…" } else { "" })
}

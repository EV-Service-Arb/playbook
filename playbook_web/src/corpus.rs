use std::{cmp::Reverse, fmt::Write, sync::LazyLock};

use regex::{Regex, RegexBuilder};
use regex_syntax::ast::{Ast, parse::Parser};

include!(concat!(env!("OUT_DIR"), "/corpus.rs"));

const PER_SOURCE: usize = 2;
const CHUNK: usize = 2_000; // chars of blocks a capture's chunk packs, so a long chapter is not one bag of words
const MIN_MATCH: usize = 2; // a lone letter is noise, not relevance
const BROAD: f64 = 0.4; // a pattern hitting this share of the files in scope is an enumeration
const OVERFLOW: usize = 5; // files per kind listed past its slots, for the agent to `read`
const LINE_CAP: usize = 700;
const STRUCTURED_LINES: usize = 12;
const REF_BLOCKS: usize = 2; // paragraphs or bullets shown per chunk
const BLOCK_LINES: usize = 8;
const CHAPTER_CAP: usize = 24_000;

/// how many files of which kinds one search shows: trust decides across kinds, relevance within one
const SLOTS: [(Kind, usize); 5] = [(Kind::Manual, 6), (Kind::Course, 2), (Kind::Call, 3), (Kind::Video, 2), (Kind::Text, 1)];

const PEOPLE: &str = "knowledge/README.md";

/// A backed bullet of a page or a digest, as `corpus::compile` resolved it.
pub struct Claim {
	from: usize,
	to: usize,
	vetted: bool,
	backing: &'static [Cited],
}
pub struct Cited {
	line: usize,
	who: &'static str,
	how: &'static str,
	trust: f64,
	day: i64,
}
impl Cited {
	// r[impl trust.effective]
	fn effective(&self, today: i64) -> f64 {
		self.trust * corpus::gamma(today - self.day)
	}

	// r[impl trust.arithmetic]
	fn label(&self, today: i64) -> String {
		format!("{} {} · {} · effective {:.2}", self.who, self.how, corpus::date(self.day), self.effective(today))
	}
}

pub struct Corpus {
	docs: Vec<Doc>,
}
impl Corpus {
	fn load() -> Self {
		let mut docs: Vec<Doc> = FILES
			.iter()
			.filter(|(p, _)| *p != PEOPLE)
			.map(|&(path, text)| {
				let lines: Vec<&str> = text.lines().collect();
				let speaker = SPEAKERS.iter().find(|(p, _)| *p == path).map(|(_, s)| s);
				let kind = match path {
					p if p.starts_with("knowledge/structured/") => Kind::Manual,
					_ if speaker.is_some_and(|s| s.who == "course") => Kind::Course,
					p if p.starts_with("knowledge/references/youtube/") => Kind::Video,
					p if p.starts_with("knowledge/references/") && lines.contains(&"## transcript") => Kind::Call,
					p if p.starts_with("knowledge/references/") => Kind::Text,
					p => unreachable!("build.rs bakes only knowledge/: {p}"),
				};
				let claims = CLAIMS.iter().find(|(p, _)| *p == path).map_or(&[][..], |(_, c)| c);
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
				let mut screen = path.ends_with("/shown.md");
				let screen = lines
					.iter()
					.map(|l| {
						if l.starts_with("# ") || l.starts_with("## ") {
							screen = path.ends_with("/shown.md") || *l == "## shown";
						}
						screen
					})
					.collect();
				let chunk = match kind == Kind::Manual || path.ends_with("/content.md") {
					true => {
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
					false => chunks(&lines),
				};
				let meta = |key: &str| lines.iter().find_map(|l| l.strip_prefix(key));
				let date = ["- recorded: ", "- uploaded: ", "- updated: ", "- pulled: "].iter().find_map(|k| meta(k)).map(|d| &d[..10]);
				let source = meta("- source: ").map(|s| s.trim_matches(['<', '>']));
				Doc {
					path,
					kind,
					lines,
					chapter,
					chunk,
					screen,
					date,
					source,
					claims,
					speaker,
					digest: None,
				}
			})
			.collect();
		// a reading beside a capture describes that recording, so it carries the recording's kind, date and source
		for i in 0..docs.len() {
			let Some((dir, reading)) = docs[i].path.rsplit_once('/').filter(|(_, f)| ["shown.md", "content.md"].contains(f)) else {
				continue;
			};
			let capture = format!("{dir}.md");
			let parent = docs.iter().position(|d| d.path == capture).unwrap_or_else(|| panic!("{} has no capture beside it", docs[i].path));
			(docs[i].kind, docs[i].date, docs[i].source) = (docs[parent].kind, docs[parent].date, docs[parent].source);
			if reading == "content.md" {
				docs[parent].digest = Some(docs[i].path);
			}
		}
		for d in docs.iter().filter(|d| d.kind > Kind::Manual) {
			let date = d.date.unwrap_or_else(|| panic!("{} states no date", d.path));
			assert!(corpus::days(date).is_some(), "{}: `{date}` is not a date", d.path);
		}
		Self { docs }
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
		if !(scope.is_empty() || scope.starts_with("knowledge")) {
			return Err("scope is a path prefix under `knowledge/structured/` or `knowledge/references/`".into());
		}
		let in_scope: Vec<&Doc> = self
			.docs
			.iter()
			.filter(|d| d.path.starts_with(scope) && (0..d.lines.len()).any(|i| d.screen[i] == screen))
			.collect();
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
			effective: Option<f64>,
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
						hits.push(Hit {
							start: doc.chunk[i],
							lines: vec![],
							branches: vec![false; branches.len()],
							score: 0.,
						});
						hits.last_mut().unwrap()
					}
				};
				hit.lines.push((i, on_line.iter().filter(|b| **b).count()));
				hit.branches.iter_mut().zip(on_line).for_each(|(h, l)| *h |= l);
			}
			if !hits.is_empty() {
				files.push(File { doc, hits, effective: None });
			}
		}
		let broad = |hit: usize, of: usize| of >= 5 && hit as f64 > of as f64 * BROAD;
		if broad(files.len(), in_scope.len()) {
			// the directories a level below the scope; below `knowledge/structured/` and `knowledge/references/` from the top, as those narrow nothing
			let depth = scope.split('/').filter(|s| !s.is_empty()).count().max(2);
			let sub = |path: &'static str| path.match_indices('/').nth(depth).map(|(i, _)| &path[..i]);
			let mut subs: Vec<(&str, usize, usize)> = Vec::new(); // sub-scope, files hit, files in it
			for d in &in_scope {
				if let Some(sub) = sub(d.path) {
					match subs.iter_mut().find(|s| s.0 == sub) {
						Some(s) => s.2 += 1,
						None => subs.push((sub, 0, 1)),
					}
				}
			}
			for f in &files {
				if let Some(sub) = sub(f.doc.path) {
					subs.iter_mut().find(|s| s.0 == sub).expect("a hit is in scope").1 += 1;
				}
			}
			subs.retain(|s| s.1 > 0);
			subs.sort_by_key(|s| Reverse(s.1));
			let mut text = format!("the pattern hits {} of the {} files in scope. Narrow it", files.len(), in_scope.len());
			match subs.is_empty() {
				true => text.push('.'),
				false => {
					text.push_str(", or pass one of these as `scope`:");
					for (sub, hit, of) in subs {
						let also = if broad(hit, of) { ", too broad as well" } else { "" };
						text.push_str(&format!("\n- {sub}: {hit} of {of}{also}"));
					}
				}
			}
			return Err(text);
		}
		if files.is_empty() {
			return Ok(Found {
				text: "no match.".into(),
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
		let today = crate::now() / 86_400;
		for f in &mut files {
			for h in &mut f.hits {
				h.score = h.branches.iter().zip(&idf).filter(|(b, _)| **b).map(|(_, idf)| idf).sum();
			}
			f.hits.sort_by(|a, b| b.score.total_cmp(&a.score));
			f.hits.truncate(PER_SOURCE);
			f.effective = f.doc.effective(f.hits[0].start, today);
		}
		files.sort_by(|a, b| {
			a.doc
				.kind
				.cmp(&b.doc.kind)
				.then(b.hits[0].score.total_cmp(&a.hits[0].score))
				.then(b.effective.partial_cmp(&a.effective).expect("a trust is never NaN"))
				.then(b.doc.date.cmp(&a.doc.date))
		});

		let mut groups: Vec<(Vec<&File>, usize)> = SLOTS
			.iter()
			.map(|(kind, slots)| (files.iter().filter(|f| f.doc.kind == *kind).collect(), *slots))
			.collect();
		// slots a ref kind leaves empty go to the others, the more trusted kind first
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
		// the manual and the course lead; the rest follow by what each is worth, the unscored last
		let leading = shown.iter().take_while(|f| f.doc.kind <= Kind::Course).count();
		shown[leading..].sort_by(|a, b| b.effective.partial_cmp(&a.effective).expect("a trust is never NaN").then(b.doc.date.cmp(&a.doc.date)));

		let mut text = String::new();
		let mut ids = Vec::new();
		for file in shown {
			let doc = file.doc;
			for hit in &file.hits {
				let first = hit.lines[0].0;
				let end = doc.chapter_end(first);
				let blocks: Vec<(usize, usize)> = match doc.kind == Kind::Manual {
					true => {
						let from = if doc.lines[hit.start].starts_with('#') { first } else { hit.start };
						let to = (from + 1..end).find(|&i| doc.chunk[i] != hit.start).unwrap_or(end);
						vec![(from, to.min(from + STRUCTURED_LINES))]
					}
					false => {
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
				writeln!(text, "{}:{} · {} · {}", doc.path, at + 1, doc.header(first), doc.label(first, today)).unwrap();
				if let Some(source) = doc.source {
					writeln!(text, "source: {source}").unwrap();
				}
				for (from, to) in blocks {
					for i in from..to {
						writeln!(text, "{:>5} | {}", i + 1, clip(&doc.line(i, today), &re)).unwrap();
					}
				}
				text.push('\n');
				ids.push(format!("{}:{}", doc.path, at + 1));
			}
		}
		if !overflow.is_empty() {
			writeln!(text, "also matched, best first within each kind:").unwrap();
			for f in overflow {
				let first = f.hits[0].lines[0].0;
				writeln!(text, "{}:{} · {} · {}", f.doc.path, first + 1, f.doc.header(first), f.doc.label(first, today)).unwrap();
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
				let found: Vec<usize> = (0..doc.lines.len())
					.filter(|&i| doc.lines[i].starts_with('#') && doc.lines[i].to_lowercase().contains(&want))
					.collect();
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
		let today = crate::now() / 86_400;
		let mut text = format!("{path}:{} · {}\n", start + 1, doc.label(start, today));
		if let Some(source) = doc.source {
			writeln!(text, "source: {source}").unwrap();
		}
		for i in start..end {
			writeln!(text, "{:>5} | {}", i + 1, doc.line(i, today)).unwrap();
			if text.len() > CHAPTER_CAP {
				writeln!(text, "… the chapter runs to line {end}. `search` inside it, with scope `{path}`, for the rest.").unwrap();
				break;
			}
		}
		Ok(Found {
			text,
			ids: vec![format!("{path}:{}", start + 1)],
		})
	}

	pub fn guide(&self, area: &str) -> Result<Found, String> {
		let found = |text: String| Found { text, ids: vec![area.to_owned()] };
		if area == "people" {
			let readme = FILES.iter().find(|(p, _)| *p == PEOPLE).expect("build.rs bakes it").1;
			let table: Vec<&str> = readme.lines().skip_while(|l| !l.starts_with("| who ")).take_while(|l| l.starts_with('|')).collect();
			return Ok(found(table.join("\n")));
		}
		let path = format!("knowledge/structured/{area}.md");
		let doc = self.docs.iter().find(|d| d.path == path).ok_or_else(|| format!("no page `{area}`. {GUIDE_DESCRIPTION}"))?;
		let today = crate::now() / 86_400;
		Ok(found((0..doc.lines.len()).map(|i| doc.line(i, today)).collect::<Vec<_>>().join("\n")))
	}
}

pub struct Found {
	pub text: String,
	pub ids: Vec<String>,
}

pub static CORPUS: LazyLock<Corpus> = LazyLock::new(Corpus::load);

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum Kind {
	Manual,
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
	/// a capture's own date; the manual dates each claim instead
	date: Option<&'static str>,
	source: Option<&'static str>,
	claims: &'static [Claim],
	/// who speaks, where the source fixes it
	speaker: Option<&'static Cited>,
	/// the `content.md` that reads this recording
	digest: Option<&'static str>,
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
		let to = (line + 1..end)
			.find(|&i| self.lines[i].trim().is_empty() || self.lines[i].starts_with("- ") || self.lines[i].starts_with('#'))
			.unwrap_or(end);
		(from, to.min(from + BLOCK_LINES).max(line + 1))
	}

	fn headers(&self) -> String {
		self.lines
			.iter()
			.enumerate()
			.filter(|(_, l)| l.starts_with('#'))
			.map(|(i, l)| format!("line {}: {l}", i + 1))
			.collect::<Vec<_>>()
			.join("\n")
	}

	fn header(&self, line: usize) -> &str {
		self.chapter[line].map_or("", |c| self.lines[c])
	}

	/// the innermost backed bullet holding `line`
	fn claim(&self, line: usize) -> Option<&'static Claim> {
		self.claims.iter().filter(|c| (c.from..c.to).contains(&line)).max_by_key(|c| c.from)
	}

	/// what backs `line` best today
	fn best(&self, line: usize, today: i64) -> Option<&'static Cited> {
		let by_worth = |a: &&Cited, b: &&Cited| a.effective(today).total_cmp(&b.effective(today));
		match self.claim(line) {
			Some(c) => c.backing.iter().max_by(by_worth),
			None => self.speaker.filter(|_| !self.screen[line] && !self.path.ends_with("/content.md")),
		}
	}

	fn effective(&self, line: usize, today: i64) -> Option<f64> {
		self.best(line, today).map(|c| c.effective(today))
	}

	// r[impl trust.arithmetic]
	/// who backs `line`, how they know, since when, and what that is worth today; or that nothing scores it
	fn label(&self, line: usize, today: i64) -> String {
		let date = || self.date.expect("asserted on load");
		match (self.best(line, today), self.claim(line)) {
			(Some(cited), Some(claim)) => format!("{}{}", cited.label(today), if claim.vetted { "" } else { " · unvetted" }),
			(Some(cited), None) => cited.label(today),
			(None, _) if self.kind == Kind::Manual => "unscored: a heading".into(),
			(None, _) if self.screen[line] => format!("a reading of the screen · {} · unscored", date()),
			(None, _) => match self.digest {
				Some(digest) => format!("as said · {} · unscored: who speaks, and what it is worth, is in {digest}", date()),
				None => format!("as said · {} · unscored", date()),
			},
		}
	}

	/// the line as a reader gets it: a citation gives what it is worth instead of what it is computed from
	fn line(&self, i: usize, today: i64) -> String {
		let cited = self.claims.iter().flat_map(|c| c.backing).find(|c| c.line == i);
		match cited {
			Some(c) => {
				let indent = &self.lines[i][..self.lines[i].len() - self.lines[i].trim_start().len()];
				let at = self.lines[i].trim_end_matches("-->").rsplit(' ').next().filter(|t| t.starts_with("http") || t.starts_with('?') || t.starts_with('#'));
				format!("{indent}[{}{}]", c.label(today), at.map(|a| format!(" · {a}")).unwrap_or_default())
			}
			None => self.lines[i].to_owned(),
		}
	}
}

/// Packs a capture's blocks (paragraphs, bullets) into chunks of at most `CHUNK` chars; a header always
/// opens one, and a block over `CHUNK` stands alone.
fn chunks(lines: &[&str]) -> Vec<usize> {
	let opens =
		|i: usize| i == 0 || !lines[i].trim().is_empty() && (lines[i].starts_with('#') || lines[i].starts_with("- ") || lines[i - 1].trim().is_empty() || lines[i - 1].starts_with('#'));
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

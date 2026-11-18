#![doc = include_str!("../README.md")]

use std::{
	collections::{BTreeMap, BTreeSet},
	fmt, fs,
	ops::Range,
	path::{Path, PathBuf},
	str::FromStr,
};

use serde::{Deserialize, Serialize};
pub use v_utils::primitives::Timeframe;
use v_utils::primitives::{Certainty, TimeframeDesignator, percent::PercentU};

const DAY_MS: f64 = 86_400_000.;
const DIGEST_SECTIONS: [&str; 3] = ["## gist", "## chapters", "## open"];

type Errors = Vec<(usize, String)>;

/// What `compile` resolved, keyed by path from the repo root.
#[derive(Deserialize, Serialize)]
pub struct Index {
	/// every backed bullet of a manual page, a digest or a log
	pub claims: BTreeMap<String, Vec<Claim>>,
	/// every capture that states a source
	pub captures: BTreeMap<String, Capture>,
	/// what the citations of each page, digest and log decay at
	pub half_lives: BTreeMap<String, HalfLife>,
	/// recordings no `content.md` reads yet
	pub pending: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Claim {
	/// the bullet and everything nested under it, 0-based
	pub lines: Range<usize>,
	pub vetted: bool,
	/// every citation beneath the bullet, by its 0-based line
	pub backing: Vec<(usize, Backing)>,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize, strum::Display, strum::EnumString, strum::VariantNames)]
#[strum(serialize_all = "lowercase")]
pub enum How {
	Ran,
	Says,
	Heard,
}

// r[impl trust.init]
#[derive(Clone, Deserialize, Serialize)]
pub struct Backing {
	who: String,
	how: How,
	certainty: Certainty,
	/// `source(who) × P(certainty)`
	init: f64,
	/// when the information is from, in days since 1970-01-01
	day: i64,
	half_life: Timeframe,
	/// a url, or a digest's `?t=` into its own capture
	at: Option<String>,
}
impl Backing {
	// r[impl trust.norm]
	/// What it is worth on `today`, decaying at `half_life` where given, else at its module's.
	pub fn norm(&self, today: i64, half_life: Option<Timeframe>) -> f64 {
		self.init * gamma(today - self.day, half_life.unwrap_or(self.half_life))
	}

	// r[impl trust.arithmetic+2]
	/// `Dante ran 2σ · 400d · init 0.57 · norm 0.21 · <url>`
	pub fn label(&self, today: i64, half_life: Option<Timeframe>) -> String {
		let at = self.at.as_ref().map(|a| format!(" · {a}")).unwrap_or_default();
		format!(
			"{} {} {}σ · {} · init {:.2} · norm {:.2}{at}",
			self.who,
			self.how,
			*self.certainty,
			age(today - self.day),
			self.init,
			self.norm(today, half_life)
		)
	}
}

/// How fast a page's knowledge goes stale, and who says so.
#[derive(Clone, Deserialize, Serialize)]
pub struct HalfLife {
	pub span: Timeframe,
	/// the page whose `<!--half-life-->` it is; none: knowledge/README.md's default
	pub page: Option<String>,
}
impl fmt::Display for HalfLife {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.page {
			Some(page) => write!(f, "half-life {}, suggested by {page}", self.span),
			None => write!(f, "half-life {}, knowledge/README.md's default", self.span),
		}
	}
}

#[derive(Deserialize, Serialize)]
pub struct Capture {
	pub day: i64,
	/// where the source fixes who speaks: a lesson, a channel's video
	pub speaker: Option<Backing>,
	source: String,
	duration: Option<u64>,
	stamps: BTreeSet<u64>,
}

// r[impl trust.decay]
/// `age_days` as a reader gets it: `0d`, `400d`, `2w`.
pub fn age(age_days: i64) -> String {
	match age_days.max(0) {
		0 => "0d".into(), // a Timeframe is never empty
		n => Timeframe::from_naive(n as u64, TimeframeDesignator::Days).to_string(),
	}
}

/// Days since 1970-01-01 of a `YYYY-MM-DD`, after Howard Hinnant's `days_from_civil`.
pub fn days(date: &str) -> Option<i64> {
	let b = date.as_bytes();
	if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
		return None;
	}
	let n = |r: Range<usize>| date[r].parse::<i64>().ok();
	let (m, d) = (n(5..7)?, n(8..10)?);
	if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
		return None;
	}
	let y = n(0..4)? - (m <= 2) as i64;
	let era = y.div_euclid(400);
	let yoe = y - era * 400;
	let doy = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
	Some(era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468)
}

/// Checks everything under `<root>/knowledge/` and resolves it, or says what is wrong as `path:line: why`.
pub fn compile(root: &Path) -> Result<Index, Vec<String>> {
	let knowledge = root.join("knowledge");
	let rel = |p: &Path| p.strip_prefix(root).expect("walked from root").to_str().expect("corpus paths are utf-8").to_owned();
	let read = |p: &Path| fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
	let tables = Tables::parse(&read(&knowledge.join("README.md")));

	let mut errors = Vec::new();
	let mut files = Vec::new();
	walk(&knowledge.join("references"), &mut files);
	let texts: Vec<(String, String)> = files.iter().map(|p| (rel(p), read(p))).collect();
	let lessons: Vec<&str> = texts.iter().filter(|(p, _)| lesson(p)).map(|(_, t)| t.as_str()).collect();

	let mut captures = BTreeMap::new();
	let mut by_source: BTreeMap<String, String> = BTreeMap::new();
	for (path, text) in &texts {
		let field = |name: &str| text.lines().find_map(|l| l.strip_prefix("- ")?.strip_prefix(name)?.strip_prefix(": "));
		let Some(source) = field("source").map(|s| s.trim_matches(['<', '>'])) else {
			continue; // a registry, an index or a reading beside a capture: none states a source
		};
		let key = source_key(source).unwrap_or_else(|| panic!("{path}: `{source}` is on no platform `source_key` knows"));
		let stamp = ["recorded", "uploaded", "updated", "pulled"]
			.iter()
			.find_map(|f| field(f))
			.unwrap_or_else(|| panic!("{path} states a source but no date"));
		let day = stamp.get(..10).and_then(days).unwrap_or_else(|| panic!("{path}: `{stamp}` does not open on a date"));
		let duration = field("duration").map(|d| {
			d.split(':')
				.fold(0, |acc, p| acc * 60 + p.parse::<u64>().unwrap_or_else(|e| panic!("{path}: duration `{d}`: {e}")))
		});
		let within = field("within").and_then(|w| w.strip_prefix('<')?.split_once('>')).map(|(u, _)| u);
		let taught = [Some(source), within].into_iter().flatten().any(|u| {
			let u = u.split('?').next().expect("split yields one");
			lessons.iter().any(|t| t.contains(u))
		});
		let who = match path.strip_prefix("knowledge/references/youtube/").and_then(|p| p.split_once('/')) {
			_ if lesson(path) || taught => Some("Eric".to_owned()),
			_ if path.starts_with("knowledge/references/research/") => Some("chatgpt".to_owned()),
			Some((channel, _)) => Some(
				tables
					.channel(channel)
					.unwrap_or_else(|| panic!("{path}: no row of knowledge/README.md#people reaches youtube `{channel}`"))
					.to_owned(),
			),
			None => None,
		};
		// r[impl trust.certainty]
		let certainty = Certainty::try_new(if lesson(path) || taught { 2. } else { 1. }).expect("non-negative");
		let speaker = who.map(|who| Backing {
			init: tables.source(&who).unwrap_or_else(|| panic!("knowledge/README.md#people has no row `{who}`")) * *PercentU::from(certainty),
			who,
			how: How::Says,
			certainty,
			day,
			half_life: tables.half_life,
			at: None,
		});
		if let Some(first) = by_source.insert(key.clone(), path.clone()) {
			panic!("{key} is captured twice: {first} and {path}");
		}
		captures.insert(
			path.clone(),
			Capture {
				day,
				speaker,
				source: source.to_owned(),
				duration,
				stamps: {
					let shown = format!("{}/shown.md", path.strip_suffix(".md").expect("walked for .md"));
					let shown = texts.iter().find(|(p, _)| *p == shown).map_or("", |(_, t)| t.as_str());
					seconds(text).chain(seconds(shown)).collect()
				},
			},
		);
	}

	let ctx = Ctx {
		tables: &tables,
		captures: &captures,
		by_source: &by_source,
	};
	let mut claims = BTreeMap::new();
	let mut half_lives = BTreeMap::new();
	let mut pending = Vec::new();
	let default = HalfLife { span: tables.half_life, page: None };
	for (path, capture) in captures.iter().filter(|(_, c)| c.duration.is_some()) {
		let digest = format!("{}/content.md", path.strip_suffix(".md").expect("walked for .md"));
		match texts.iter().find(|(p, _)| *p == digest) {
			Some((_, text)) => {
				let (found, mut wrong) = ctx.digest(text, capture);
				stated_half_life(&text.lines().collect::<Vec<_>>(), false, &mut wrong);
				claims.insert(digest.clone(), found);
				half_lives.insert(digest.clone(), default.clone());
				errors.extend(wrong.into_iter().map(|(line, why)| format!("{digest}:{line}: {why}")));
			}
			None => pending.push(path.clone()),
		}
	}
	for (path, _) in texts.iter().filter(|(p, _)| p.ends_with("/content.md")) {
		let capture = format!("{}.md", path.strip_suffix("/content.md").expect("filtered on it"));
		if !captures.get(&capture).is_some_and(|c| c.duration.is_some()) {
			errors.push(format!("{path}:1: digests no recording: `{capture}` is none"));
		}
	}

	let mut pages = Vec::new();
	walk(&knowledge.join("structured"), &mut pages);
	walk(&knowledge.join("internal"), &mut pages);
	assert!(!pages.is_empty(), "no pages under {}/structured", knowledge.display());
	let pages: Vec<(String, String, PathBuf)> = pages.iter().map(|p| (rel(p), read(p), p.parent().expect("a page sits in a dir").to_owned())).collect();
	let mut stated = BTreeMap::new();
	for (path, text, _) in &pages {
		let mut wrong = Errors::new();
		stated.insert(path.as_str(), stated_half_life(&text.lines().collect::<Vec<_>>(), true, &mut wrong));
		errors.extend(wrong.into_iter().map(|(line, why)| format!("{path}:{line}: {why}")));
	}
	for (path, text, dir) in &pages {
		// the nearest of `x.md`, then each page whose `x/` holds it
		let mut module = path.strip_suffix(".md").expect("walked for .md");
		let half_life = loop {
			if let Some(Some(span)) = stated.get(format!("{module}.md").as_str()) {
				break HalfLife {
					span: *span,
					page: Some(format!("{module}.md")),
				};
			}
			match module.rsplit_once('/') {
				Some((up, _)) => module = up,
				None => break default.clone(),
			}
		};
		let (found, wrong) = ctx.page(text, dir, half_life.span);
		errors.extend(wrong.into_iter().map(|(line, why)| format!("{path}:{line}: {why}")));
		claims.insert(path.clone(), found);
		half_lives.insert(path.clone(), half_life);
	}

	let mut logs = Vec::new();
	walk(&knowledge.join("log"), &mut logs);
	for log in &logs {
		let text = read(log);
		let lines: Vec<&str> = text.lines().collect();
		let mut wrong = Errors::new();
		stated_half_life(&lines, false, &mut wrong);
		let found = ctx.bullets(&lines, 0..lines.len(), None, tables.half_life, false, &mut wrong);
		errors.extend(wrong.into_iter().map(|(line, why)| format!("{}:{line}: {why}", rel(log))));
		claims.insert(rel(log), found);
		half_lives.insert(rel(log), default.clone());
	}

	// r[impl trust.derived+2]
	for (path, text) in texts.iter().map(|(p, t)| (p, t.as_str())).chain(pages.iter().map(|(p, t, _)| (p, t.as_str()))) {
		for (i, l) in text.lines().enumerate() {
			if l.contains(" · norm ") || l.starts_with("# ") && l.contains(" · half-life ") {
				errors.push(format!(
					"{path}:{}: a label `playbook view` rendered: write what it renders from, as knowledge/README.md#citation has it",
					i + 1
				));
			}
		}
	}

	match errors.is_empty() {
		true => Ok(Index {
			claims,
			captures,
			half_lives,
			pending,
		}),
		false => Err(errors),
	}
}

/// What `age_days` leaves of a trust: ½ at one half-life.
fn gamma(age_days: i64, half_life: Timeframe) -> f64 {
	let age = age_days.max(0); // a capture dated in a timezone ahead of the clock is a day in its future
	1. / (1. + age as f64 / (half_life.0 as f64 / DAY_MS))
}

/// A page's own `<!--half-life <timeframe>-->`, right under its title; nowhere else does one reach anything.
fn stated_half_life(lines: &[&str], page: bool, errors: &mut Errors) -> Option<Timeframe> {
	let mut found = None;
	for (i, l) in lines.iter().enumerate() {
		let Some(span) = l.trim().strip_prefix("<!--half-life").and_then(|r| r.strip_suffix("-->")) else {
			continue;
		};
		if !page || i != 1 {
			errors.push((i + 1, "a half-life sits on the line right under a page's title, and reaches nothing elsewhere".into()));
			continue;
		}
		match Timeframe::from_str(span.trim()) {
			Ok(t) if t.0 > 0 => found = Some(t),
			Ok(_) => errors.push((i + 1, "a half-life is longer than nothing".into())),
			Err(e) => errors.push((i + 1, format!("`{}` is no timeframe: {e}", span.trim()))),
		}
	}
	found
}

struct Ctx<'a> {
	tables: &'a Tables,
	captures: &'a BTreeMap<String, Capture>,
	by_source: &'a BTreeMap<String, String>,
}
impl Ctx<'_> {
	fn page(&self, text: &str, dir: &Path, half_life: Timeframe) -> (Vec<Claim>, Errors) {
		let lines: Vec<&str> = text.lines().collect();
		let mut errors = Errors::new();
		if !lines.first().is_some_and(|l| l.starts_with("# ")) {
			errors.push((1, "a page opens `# <topic>`".into()));
		}
		let first_section = lines.iter().position(|l| l.starts_with("## ")).unwrap_or(lines.len());
		if !lines[1.min(first_section)..first_section].iter().any(|l| !l.trim().is_empty() && !l.starts_with("<!--half-life")) {
			errors.push((1, "a page states what we do before its first `## `".into()));
		}
		let open = lines.iter().position(|l| *l == "## open");
		if let Some(o) = open
			&& let Some(after) = lines[o + 1..].iter().position(|l| l.starts_with('#'))
		{
			errors.push((o + after + 2, "`## open` is a page's last section".into()));
		}
		for (i, l) in lines.iter().enumerate() {
			if let Some(link) = l.trim().strip_prefix("resolved: ")
				&& !link
					.rsplit_once("](")
					.and_then(|(_, u)| u.strip_suffix(')'))
					.is_some_and(|u| u.contains("log/resolved/") && dir.join(u).is_file())
			{
				errors.push((i + 1, "`resolved:` links its record in `knowledge/log/resolved/`, which must exist".into()));
			}
		}
		let open = open.unwrap_or(lines.len());
		let mut claims = self.bullets(&lines, 0..open, None, half_life, true, &mut errors);
		claims.extend(self.bullets(&lines, open..lines.len(), None, half_life, false, &mut errors));
		(claims, errors)
	}

	// r[impl knowledge.digest.first]
	fn digest(&self, text: &str, capture: &Capture) -> (Vec<Claim>, Errors) {
		let lines: Vec<&str> = text.lines().collect();
		let mut errors = Errors::new();
		if !lines.first().is_some_and(|l| l.starts_with("# ")) {
			errors.push((1, "a digest opens `# <the capture's title>`".into()));
		}
		for field in ["- of: ", "- digested by: "] {
			if !lines.iter().any(|l| l.starts_with(field)) {
				errors.push((1, format!("a digest states `{field}`")));
			}
		}
		let sections: Vec<(usize, usize)> = lines
			.iter()
			.enumerate()
			.filter(|(_, l)| l.starts_with("## ") || l.starts_with("# ") && lines[0] != **l)
			.map(|(i, l)| (i, DIGEST_SECTIONS.iter().position(|s| s == l).unwrap_or(usize::MAX)))
			.collect();
		for w in sections.windows(2) {
			if w[1].1 <= w[0].1 {
				errors.push((w[1].0 + 1, format!("a digest's sections are {DIGEST_SECTIONS:?}, in that order")));
			}
		}
		for (i, s) in &sections {
			if *s == usize::MAX {
				errors.push((i + 1, format!("a digest's sections are {DIGEST_SECTIONS:?}")));
			}
		}
		let at = |s: usize| sections.iter().find(|(_, pos)| *pos == s).map(|(i, _)| *i);
		let (Some(_), Some(chapters)) = (at(0), at(1)) else {
			errors.push((1, "a digest has `## gist` and `## chapters`".into()));
			return (vec![], errors);
		};
		let end = sections.iter().map(|(i, _)| *i).find(|i| *i > chapters).unwrap_or(lines.len());

		let mut last = None;
		for (i, l) in lines.iter().enumerate().take(end).skip(chapters + 1).filter(|(_, l)| l.starts_with('#')) {
			let parsed = l
				.strip_prefix("### [")
				.and_then(|r| r.split_once("]("))
				.and_then(|(_, r)| r.split_once(')'))
				.filter(|(url, title)| source_key(url) == source_key(&capture.source) && !title.trim().is_empty())
				.and_then(|(url, _)| seconds(url).next());
			match parsed {
				None => errors.push((i + 1, "a chapter is `### [<stamp>](<the capture's url at its second>) <topic>`".into())),
				Some(t) if !capture.stamps.contains(&t) => errors.push((i + 1, format!("the capture has no stamp at {t}s"))),
				Some(t) if last.is_some_and(|l| t < l) => errors.push((i + 1, "chapters go in the order they were said".into())),
				Some(t) => last = Some(t),
			}
		}
		let claims = self.bullets(&lines, chapters + 1..end, Some(capture), self.tables.half_life, true, &mut errors);
		(claims, errors)
	}

	/// The bullets of `within`, each with every citation beneath it. Inside a digest the capture is `given`. `cited`: every
	/// top-level bullet cites.
	fn bullets(&self, lines: &[&str], within: Range<usize>, given: Option<&Capture>, half_life: Timeframe, cited: bool, errors: &mut Errors) -> Vec<Claim> {
		struct Open {
			claim: Claim,
			indent: usize,
		}
		let mut done: Vec<(usize, Claim)> = Vec::new();
		let mut stack: Vec<Open> = Vec::new();
		let mut fenced = false;
		let mut refused = BTreeSet::new(); // bullets whose citation is already reported
		let close = |stack: &mut Vec<Open>, done: &mut Vec<(usize, Claim)>, to: usize, at: usize| {
			while stack.last().is_some_and(|o| o.indent >= to) {
				let mut o = stack.pop().expect("checked");
				o.claim.lines.end = at;
				done.push((o.indent, o.claim));
			}
		};
		for i in within.clone() {
			let raw = lines[i];
			let trimmed = raw.trim();
			if trimmed.starts_with("```") {
				fenced = !fenced;
			}
			if fenced || trimmed.is_empty() {
				continue;
			}
			let indent = raw.len() - raw.trim_start().len();
			if raw.starts_with('#') {
				close(&mut stack, &mut done, 0, i);
				continue;
			}
			if let Some(text) = trimmed.strip_prefix("- ") {
				close(&mut stack, &mut done, indent, i);
				stack.push(Open {
					claim: Claim {
						lines: i..i,
						vetted: !text.starts_with("(?)"),
						backing: vec![],
					},
					indent,
				});
				continue;
			}
			close(&mut stack, &mut done, indent, i);
			let Some(cite) = Cite::parse(trimmed, given.is_some()) else { continue };
			match cite.and_then(|c| self.resolve(c, given, half_life)) {
				Err(why) => {
					errors.push((i + 1, why));
					refused.extend(stack.iter().map(|o| o.claim.lines.start));
				}
				Ok(_) if stack.is_empty() => errors.push((i + 1, "a citation sits under the bullet it backs".into())),
				Ok(b) => stack.iter_mut().for_each(|o| o.claim.backing.push((i, b.clone()))),
			}
		}
		close(&mut stack, &mut done, 0, within.end);
		for (indent, claim) in &done {
			if cited && *indent == 0 && claim.backing.is_empty() && !refused.contains(&claim.lines.start) {
				errors.push((claim.lines.start + 1, "nothing beneath this bullet cites a source".into()));
			}
		}
		let mut claims: Vec<Claim> = done.into_iter().map(|(_, c)| c).filter(|c| !c.backing.is_empty()).collect();
		claims.sort_by_key(|c| (c.lines.start, c.lines.end));
		claims
	}

	// r[impl trust.age]
	fn resolve(&self, cite: Cite, given: Option<&Capture>, half_life: Timeframe) -> Result<Backing, String> {
		let how = cite.how;
		let backing = |who: &str, day: i64| {
			let source = self.tables.source(who).ok_or_else(|| format!("`{who}` has no row in knowledge/README.md#people"))?;
			Ok(Backing {
				who: who.to_owned(),
				how,
				certainty: cite.certainty,
				init: source * *PercentU::from(cite.certainty),
				day,
				half_life,
				at: cite.at.map(str::to_owned),
			})
		};
		let stated = cite.date.map(|d| days(d).ok_or_else(|| format!("`{d}` is not a date"))).transpose()?;
		let Some(at) = cite.at else {
			let day = stated.ok_or("a citation of nothing recorded still dates it")?;
			return match cite.who.is_empty() {
				true => Err("a citation of nothing recorded names who says so".into()),
				false => backing(&cite.who, day),
			};
		};
		if at.contains("github.com/") && at.contains("/playbook/issues/") {
			if !at.contains("#issuecomment-") {
				return Err("a test of our own cites the comment that closed its issue, not the issue".into());
			}
			if cite.who != "us" || how != How::Ran {
				return Err("an issue comment is a test of our own: `us ran`".into());
			}
			return backing("us", stated.ok_or("a test of our own is dated")?);
		}
		let capture = match given {
			Some(c) if at.starts_with('?') || at.starts_with('#') => c,
			Some(_) => return Err("a digest cites its own capture: `?t=<second>`".into()),
			None => {
				let key = source_key(at).ok_or_else(|| format!("`{at}` is on no platform this checker knows"))?;
				let path = self.by_source.get(&key).ok_or_else(|| format!("`{at}` has no capture in knowledge/references/"))?;
				&self.captures[path]
			}
		};
		let day = match (stated, given) {
			(Some(d), _) if d > capture.day => return Err("dated after its capture".into()),
			(Some(d), _) => d,
			(None, Some(_)) => capture.day,
			(None, None) => return Err("a manual citation states its date".into()),
		};
		match (seconds(at).next(), capture.duration) {
			(Some(t), Some(d)) if t > d => return Err(format!("`t={t}` is past the capture's end at {d}s")),
			(Some(t), Some(_)) if given.is_some() && !capture.stamps.contains(&t) => return Err(format!("the capture has no stamp at {t}s")),
			(Some(_), None) => return Err("`t=` on a capture that has no duration".into()),
			(None, _) if given.is_some() => return Err("a digest cites the second: `?t=<second>`".into()),
			_ => {}
		}
		match (cite.who.is_empty(), &capture.speaker) {
			(false, _) => backing(&cite.who, day),
			(true, Some(s)) => backing(&s.who, day),
			(true, None) => Err("names nobody, and the source fixes no speaker".into()),
		}
	}
}

/// `<!--[<date>] [<who>] <how> <n>σ [<url>|?t=<second>]-->`
struct Cite<'a> {
	date: Option<&'a str>,
	who: String,
	how: How,
	certainty: Certainty,
	at: Option<&'a str>,
}
impl<'a> Cite<'a> {
	/// `None`: the line is no citation. In a digest one ends on `?t=`; elsewhere it opens on a date.
	fn parse(line: &'a str, in_digest: bool) -> Option<Result<Self, String>> {
		let inner = line.strip_prefix("<!--")?.strip_suffix("-->")?;
		let mut tokens: Vec<&str> = inner.split_whitespace().collect();
		let dated = tokens.first().is_some_and(|t| t.len() == 10 && t.as_bytes()[4] == b'-' && t.as_bytes()[0].is_ascii_digit());
		let placed = tokens.last().is_some_and(|t| t.starts_with("http") || t.starts_with("?t=") || t.starts_with("#t="));
		if !(dated || in_digest && placed) {
			return None;
		}
		let date = dated.then(|| tokens.remove(0));
		let at = placed.then(|| tokens.pop().expect("checked"));
		let certainty = match tokens.pop().and_then(|t| t.strip_suffix('σ')) {
			Some(n) => match n.parse::<f64>().map_err(|e| e.to_string()).and_then(|n| Certainty::try_new(n).map_err(|e| e.to_string())) {
				Ok(c) => c,
				Err(e) => return Some(Err(format!("`{n}σ` is no certainty: {e}"))),
			},
			None => return Some(Err("a citation says how sure its speaker is, as `<n>σ`, before its url".into())),
		};
		let Some(how) = tokens.pop().and_then(|h| How::from_str(h).ok()) else {
			let hows: &[&str] = <How as strum::VariantNames>::VARIANTS;
			return Some(Err(format!("a citation says how its speaker knows, one of {hows:?}, before its certainty")));
		};
		Some(Ok(Self {
			date,
			who: tokens.join(" "),
			how,
			certainty,
			at,
		}))
	}
}

/// The trust tables of `knowledge/README.md`.
struct Tables {
	half_life: Timeframe,
	/// who, their trust, how to reach them
	people: Vec<(String, f64, String)>,
}
impl Tables {
	fn parse(readme: &str) -> Self {
		let mut half_life = None;
		let mut people: Vec<(String, f64, String)> = Vec::new();
		let mut header: Vec<&str> = Vec::new();
		for l in readme.lines() {
			let Some(row) = l.strip_prefix('|') else {
				header.clear();
				continue;
			};
			let cells: Vec<&str> = row.split('|').map(str::trim).collect();
			if header.is_empty() {
				header = cells;
				continue;
			}
			if cells[0].starts_with("---") {
				continue;
			}
			match header[..2] {
				["default", "value"] => match cells[0] {
					"half-life" => half_life = Some(Timeframe::from_str(cells[1]).unwrap_or_else(|e| panic!("knowledge/README.md#trust: half-life `{}`: {e}", cells[1]))),
					d => panic!("knowledge/README.md#trust: no default `{d}`"),
				},
				["who", "trust"] => {
					let trust = cells[1]
						.parse()
						.ok()
						.or_else(|| people.iter().find(|p| p.0 == cells[1]).map(|p| p.1))
						.unwrap_or_else(|| panic!("knowledge/README.md#people: `{}` trusts `{}`, which is neither a number nor a row above it", cells[0], cells[1]));
					assert!((0. ..=1.).contains(&trust), "knowledge/README.md#people: `{}` is trusted {trust}, outside [0, 1]", cells[0]);
					people.push((cells[0].to_owned(), trust, cells[2].to_owned()));
				}
				_ => {}
			}
		}
		assert!(!people.is_empty(), "knowledge/README.md has the people table");
		Self {
			half_life: half_life.expect("knowledge/README.md#trust states the default half-life"),
			people,
		}
	}

	fn source(&self, who: &str) -> Option<f64> {
		self.people.iter().find(|p| p.0 == who).map(|p| p.1)
	}

	fn channel(&self, handle: &str) -> Option<&str> {
		let reach = format!("youtube `{handle}`");
		self.people.iter().find(|p| p.2.contains(&reach)).map(|p| p.0.as_str())
	}
}

fn lesson(path: &str) -> bool {
	path.starts_with("knowledge/references/skool_") && path.contains("/course/")
}

/// The identity a url shares with the `source:` of its capture.
fn source_key(url: &str) -> Option<String> {
	let param = |name: &str| url.split_once('?')?.1.split('#').next()?.split('&').find_map(|kv| kv.strip_prefix(name)?.strip_prefix('='));
	if url.contains("youtube.com/watch") {
		return Some(format!("youtube:{}", param("v")?));
	}
	if url.contains("skool.com/") && url.contains("/classroom") {
		return Some(format!("skool:{}", param("md")?));
	}
	[
		("loom.com/share/", "loom"),
		("fathom.video/share/", "fathom"),
		("chatgpt.com/share/", "chatgpt"),
		("youtu.be/", "youtube"),
		("drive.google.com/file/d/", "drive"),
		("docs.google.com/document/d/", "doc"),
		("player.vimeo.com/video/", "vimeo"),
		("vocaroo.com/", "vocaroo"),
	]
	.iter()
	.find_map(|(prefix, platform)| {
		let id = url.split_once(prefix)?.1.split(['?', '#', '&', '/']).next()?;
		Some(format!("{platform}:{id}"))
	})
}

/// Every second a text links: the `<n>` of each `?t=<n>`, `&t=<n>` and `#t=<n>`.
fn seconds(text: &str) -> impl Iterator<Item = u64> + '_ {
	text.match_indices("t=")
		.filter(|(i, _)| *i > 0 && matches!(text.as_bytes()[i - 1], b'?' | b'&' | b'#'))
		.filter_map(|(i, _)| {
			let digits = &text[i + 2..];
			digits[..digits.find(|c: char| !c.is_ascii_digit()).unwrap_or(digits.len())].parse().ok()
		})
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
	let Ok(entries) = fs::read_dir(dir) else {
		return; // `knowledge/internal/` is absent from the member image's sources
	};
	for entry in entries {
		let path = entry.expect("dir entries are readable").path();
		if path.is_dir() {
			walk(&path, out);
		} else if path.extension().is_some_and(|e| e == "md") && path.file_name().is_some_and(|n| n != "README.md") {
			out.push(path);
		}
	}
	out.sort();
}

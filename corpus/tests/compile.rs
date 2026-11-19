//! What `compile` admits and what it refuses, on a tree of one capture, one digest, two pages and a log.

use std::{fs, path::PathBuf};

const README: &str = include_str!("../../knowledge/README.md");
const CAPTURE: &str = "# Group call

- source: <https://www.loom.com/share/abc>
- recorded: 2026-09-25T00:01:52.046Z
- duration: 00:10:00

## transcript

### [00:32](https://www.loom.com/share/abc?t=32)

said

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

post the job where contractors already look

## where

- (?) facebook marketplace finds one within the hour
  - Dante found contractor in 10m on Facebook marketplace
    <!--2026-09-25 Dante ran 2σ https://www.loom.com/share/abc?t=353-->
";
const PARENT: &str = "# contractors

who does the work
";
const LOG: &str = "# 2026-10-01 call

## contractors/sourcing.md

- facebook marketplace
  <!--2026-09-25 Dante ran 2σ https://www.loom.com/share/abc?t=353-->

## not kept

- a story
";

/// Compiles the fixture with each `(from, to)` applied to every file but the capture; the errors, or `ok`.
#[track_caller]
fn check(name: &str, edits: &[(&str, &str)], expect: &str) {
	let root: PathBuf = std::env::temp_dir().join(format!("corpus-{}-{name}", std::process::id()));
	let write = |path: &str, text: &str| {
		let path = root.join("knowledge").join(path);
		fs::create_dir_all(path.parent().unwrap()).unwrap();
		fs::write(path, edits.iter().fold(text.to_owned(), |t, (from, to)| t.replace(from, to))).unwrap();
	};
	write("README.md", README);
	fs::create_dir_all(root.join("knowledge/references/loom/call")).unwrap();
	fs::write(root.join("knowledge/references/loom/call/src.md"), CAPTURE).unwrap();
	write("references/loom/call/content.md", DIGEST);
	write("structured/contractors.md", PARENT);
	write("structured/contractors/sourcing.md", PAGE);
	write("log/digest/2026-10-01-call.md", LOG);
	let got = match corpus::compile(&root) {
		Ok(index) => format!(
			"ok: {} claims, {} pending, {}",
			index.claims.values().map(Vec::len).sum::<usize>(),
			index.pending.len(),
			index.half_lives["knowledge/structured/contractors/sourcing.md"]
		),
		Err(errors) => errors.join("\n"),
	};
	fs::remove_dir_all(&root).unwrap();
	assert!(got.contains(expect), "{name}: wanted `{expect}` in:\n{got}");
}

#[test]
fn compile() {
	// r[verify knowledge.digest.first]
	check("sound", &[], "ok: 4 claims, 0 pending");
	// r[verify trust.derived+2]
	check(
		"a written score",
		&[("<!--2026-09-25 Dante ran 2σ https", "<!--r3 2026-09-25 https")],
		"sourcing.md:7: nothing beneath this bullet cites a source",
	);
	check(
		"a pasted label",
		&[(
			"<!--2026-09-25 Dante ran 2σ https://www.loom.com/share/abc?t=353-->",
			"[Dante ran, 95% sure · 54d · init 0.57 · norm 0.56 · https://www.loom.com/share/abc?t=353]",
		)],
		"sourcing.md:9: a label `playbook view` rendered",
	);
	check("nobody's row", &[("Dante ran", "Dnate ran")], "`Dnate` has no row in knowledge/README.md#people");
	check("no how", &[("Dante ran 2σ https", "Dante 2σ https")], "says how its speaker knows");
	// r[verify trust.certainty]
	check("no certainty", &[("ran 2σ https", "ran https")], "says how sure its speaker is");
	check("no certainty in a digest", &[("ran 2σ ?t=", "ran ?t=")], "content.md:15: a citation says how sure");
	check("a certainty of nothing", &[("ran 2σ https", "ran xσ https")], "`xσ` is no certainty");
	check("a negative certainty", &[("ran 2σ https", "ran -1σ https")], "`-1σ` is no certainty");
	// r[verify trust.age]
	check("from the future", &[("<!--2026-09-25 Dante", "<!--2026-09-26 Dante")], "sourcing.md:9: dated after its capture");
	check("older than said", &[("<!--2026-09-25 Dante", "<!--2026-03-01 Dante")], "ok: 4 claims");
	check("past the end", &[("abc?t=353-->", "abc?t=3530-->")], "`t=3530` is past the capture's end at 600s");
	check("a stamp of its own", &[("ran 2σ ?t=353", "ran 2σ ?t=354")], "content.md:15: the capture has no stamp at 354s");
	check(
		"a chapter of its own",
		&[("abc?t=353) Dante", "abc?t=400) Dante")],
		"content.md:12: the capture has no stamp at 400s",
	);
	check(
		"another capture",
		&[(
			"- (?) facebook",
			"- (?) elsewhere\n  <!--2026-09-25 Dante ran 2σ https://www.loom.com/share/zzz?t=1-->\n- (?) facebook",
		)],
		"has no capture in knowledge/references/",
	);
	check("never recorded", &[("ran 2σ https://www.loom.com/share/abc?t=353-->", "says 1σ-->")], "ok: 4 claims");
	check(
		"no lead",
		&[("post the job where contractors already look\n", "")],
		"a page states what we do before its first `## `",
	);
	check(
		"a log cites as the manual does",
		&[("- facebook marketplace\n  <!--2026-09-25 Dante ran 2σ", "- facebook marketplace\n  <!--2026-09-25 Dante ran")],
		"2026-10-01-call.md:6: a citation says how sure",
	);

	// r[verify trust.norm+2]
	check("the default half-life", &[], "half-life 1y, knowledge/README.md's default");
	check(
		"a page's own",
		&[("# sourcing\n", "# sourcing\n<!--half-life 5y-->\n")],
		"half-life 5y, suggested by knowledge/structured/contractors/sourcing.md",
	);
	check(
		"its module's",
		&[("# contractors\n", "# contractors\n<!--half-life 5y-->\n")],
		"half-life 5y, suggested by knowledge/structured/contractors.md",
	);
	check(
		"the nearest",
		&[("# contractors\n", "# contractors\n<!--half-life 5y-->\n"), ("# sourcing\n", "# sourcing\n<!--half-life 3M-->\n")],
		"half-life 3M, suggested by knowledge/structured/contractors/sourcing.md",
	);
	check(
		"a half-life is no lead",
		&[("# sourcing\n", "# sourcing\n<!--half-life 5y-->\n"), ("post the job where contractors already look\n", "")],
		"before its first `## `",
	);
	check(
		"a malformed half-life",
		&[("# sourcing\n", "# sourcing\n<!--half-life 5x-->\n")],
		"sourcing.md:2: `5x` is no timeframe",
	);
	check(
		"no half-life at all",
		&[("# sourcing\n", "# sourcing\n<!--half-life 0d-->\n")],
		"sourcing.md:2: a half-life is longer than nothing",
	);
	check(
		"a half-life out of reach",
		&[("## where\n", "<!--half-life 5y-->\n## where\n")],
		"sourcing.md:5: a half-life sits on the line right under a page's title",
	);
	check(
		"a half-life in a log",
		&[("## not kept\n", "<!--half-life 5y-->\n## not kept\n")],
		"2026-10-01-call.md:8: a half-life sits",
	);
}

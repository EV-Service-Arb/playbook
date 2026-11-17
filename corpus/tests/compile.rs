//! What `compile` admits and what it refuses, on a tree of one capture, one digest and one page.

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

- of: [call.md](../call.md)
- digested by: a test, 2026-10-01

## gist

two members on their week

## chapters

### [05:53](https://www.loom.com/share/abc?t=353) Dante finds a contractor

- found a plumber on facebook marketplace in ten minutes
  <!--Dante ran ?t=353-->
";
const PAGE: &str = "# sourcing

post the job where contractors already look

## where

- (?) facebook marketplace finds one within the hour
  - Dante found contractor in 10m on Facebook marketplace
    <!--2026-09-25 Dante ran https://www.loom.com/share/abc?t=353-->
";

/// Compiles the fixture with `from` replaced by `to` in its page and its digest; the errors, or `ok`.
#[track_caller]
fn check(name: &str, from: &str, to: &str, expect: &str) {
	let root: PathBuf = std::env::temp_dir().join(format!("corpus-{}-{name}", std::process::id()));
	let write = |path: &str, text: &str| {
		let path = root.join("knowledge").join(path);
		fs::create_dir_all(path.parent().unwrap()).unwrap();
		fs::write(path, text).unwrap();
	};
	write("README.md", README);
	write("references/loom/call.md", CAPTURE);
	write("references/loom/call/content.md", &DIGEST.replace(from, to));
	write("structured/contractors/sourcing.md", &PAGE.replace(from, to));
	let got = match corpus::compile(&root) {
		Ok(index) => format!("ok: {} claims, {} pending", index.claims.values().map(Vec::len).sum::<usize>(), index.pending.len()),
		Err(errors) => errors.join("\n"),
	};
	fs::remove_dir_all(&root).unwrap();
	assert!(got.contains(expect), "{name}: wanted `{expect}` in:\n{got}");
}

#[test]
fn compile() {
	// r[verify knowledge.digest.first]
	check("sound", "", "", "ok: 3 claims, 0 pending");
	// r[verify trust.derived]
	check(
		"a written score",
		"<!--2026-09-25 Dante ran",
		"<!--r3 2026-09-25",
		"sourcing.md:7: nothing beneath this bullet cites a source",
	);
	check("nobody's row", "Dante ran", "Dnate ran", "`Dnate` has no row in knowledge/README.md#people");
	check("no how", "Dante ran https", "Dante https", "says how its speaker knows");
	// r[verify trust.age]
	check("from the future", "<!--2026-09-25 Dante", "<!--2026-09-26 Dante", "sourcing.md:9: dated after its capture");
	check("older than said", "<!--2026-09-25 Dante", "<!--2026-03-01 Dante", "ok: 3 claims");
	check("past the end", "abc?t=353-->", "abc?t=3530-->", "`t=3530` is past the capture's end at 600s");
	check("a stamp of its own", "ran ?t=353", "ran ?t=354", "content.md:15: the capture has no stamp at 354s");
	check("a chapter of its own", "abc?t=353) Dante", "abc?t=400) Dante", "content.md:12: the capture has no stamp at 400s");
	check(
		"another capture",
		"- (?) facebook",
		"- (?) elsewhere\n  <!--2026-09-25 Dante ran https://www.loom.com/share/zzz?t=1-->\n- (?) facebook",
		"has no capture in knowledge/references/",
	);
	check("never recorded", "ran https://www.loom.com/share/abc?t=353-->", "says-->", "ok: 3 claims");
	check("no lead", "post the job where contractors already look\n", "", "a page states what we do before its first `## `");
}

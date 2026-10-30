//! Answers `playbook/eval.json` the way a member's client does, on this tree and on the last
//! release, and has an agent judge each answer against the case's expectations. Each side answers
//! with the `plugin/` its rev ships; a rev with none shipped the server alone.

use std::{
	io::{Read as _, Write as _},
	net::{TcpListener, TcpStream},
	num::NonZeroUsize,
	path::{Path, PathBuf},
	process::{Child, Command, Stdio},
	sync::atomic::{AtomicUsize, Ordering},
	time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const ANSWER_MODEL: &str = "claude-sonnet-5-5";
const JUDGE_MODEL: &str = "claude-sonnet-5-5";
const PANEL_ASSERTION_KEYS: &str = "eval:WGZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmY="; // the Ed25519 base point; nobody signs in here
/// What a side takes from its info rev; the rest of its tree is the system rev's.
const INFO: [&str; 3] = ["structured/approved", "structured/suggested", "ref"];
const OUT: &str = "/tmp/playbook-eval";

/// Whether any case scored lower on this tree than on the old side.
pub fn run(root: &Path, against: Option<String>, info: Option<String>, jobs: NonZeroUsize, ids: Vec<String>) -> bool {
	assert!(
		std::env::var_os("CLAUDE_CODE_OAUTH_TOKEN").is_some(),
		"CLAUDE_CODE_OAUTH_TOKEN is unset; `claude setup-token` makes one"
	);
	let cases: Vec<Case> =
		serde_json::from_str(&std::fs::read_to_string(root.join("playbook/eval.json")).expect("playbook/eval.json is in the tree")).expect("playbook/eval.json is a list of cases");
	for id in &ids {
		assert!(cases.iter().any(|c| &c.id == id), "playbook/eval.json has no case {id}");
	}
	let cases: Vec<Case> = cases.into_iter().filter(|c| ids.is_empty() || ids.contains(&c.id)).collect();

	// shortcut: untracked files are not in `stash create`, so a new file is evaluated only once committed
	let new = match git(root, &["stash", "create"]) {
		stash if stash.is_empty() => git(root, &["rev-parse", "HEAD"]),
		stash => stash,
	};
	let old = against.unwrap_or_else(|| git(root, &["describe", "--tags", "--abbrev=0", "--match", "v*", "--exclude", "*-*", &format!("{new}^")]));
	let info = info.unwrap_or_else(|| new.clone());
	let sides = [
		Side {
			name: "new",
			system: new,
			info: info.clone(),
		},
		Side { name: "old", system: old, info },
	];
	for s in &sides {
		println!("{}: system {}, info {}", s.name, s.system, s.info);
	}

	let out = PathBuf::from(OUT).join(SystemTime::now().duration_since(UNIX_EPOCH).expect("the clock is past 1970").as_secs().to_string());
	std::fs::create_dir_all(&out).unwrap_or_else(|e| panic!("{}: {e}", out.display()));
	let bins: Vec<PathBuf> = sides.iter().map(|s| build(root, s)).collect();
	let servers: Vec<Server> = sides.iter().zip(&bins).map(|(s, bin)| serve(bin, s.name, &cases, &out)).collect();

	let todo: Vec<(usize, usize)> = (0..cases.len()).flat_map(|c| (0..sides.len()).map(move |s| (c, s))).collect();
	let next = AtomicUsize::new(0);
	let mut answered: Vec<Vec<Option<Answered>>> = cases.iter().map(|_| sides.iter().map(|_| None).collect()).collect();
	std::thread::scope(|scope| {
		let workers: Vec<_> = (0..jobs.get().min(todo.len()))
			.map(|_| {
				scope.spawn(|| {
					let mut done = Vec::new();
					//LOOP: bounded by `todo`, each index handed out once
					while let Some(&(c, s)) = todo.get(next.fetch_add(1, Ordering::Relaxed)) {
						done.push((c, s, ask(&cases[c], &sides[s], &servers[s], &out)));
					}
					done
				})
			})
			.collect();
		for w in workers {
			for (c, s, a) in w.join().unwrap_or_else(|e| std::panic::resume_unwind(e)) {
				answered[c][s] = Some(a);
			}
		}
	});
	let answered: Vec<Vec<Answered>> = answered
		.into_iter()
		.map(|a| a.into_iter().map(|a| a.expect("every (case, side) is in `todo`")).collect())
		.collect();
	drop(servers);

	let width = cases.iter().fold("case".len(), |w, c| w.max(c.id.len()));
	println!("\n{:width$}  side  skill  turns  tokens in/out  answer $  calls", "case");
	for (c, a) in cases.iter().zip(&answered) {
		for (s, a) in sides.iter().zip(a) {
			let calls: Vec<String> = a.calls.iter().map(|(tool, n)| format!("{tool} {n}")).collect();
			println!(
				"{:width$}  {:4}  {:5}  {:5}  {:>6}k/{:<5}  {:8.2}  {}",
				c.id,
				s.name,
				if a.skill { "✓" } else { "✗" },
				a.turns,
				a.tokens_in / 1000,
				format!("{}k", a.tokens_out / 1000),
				a.answer_usd,
				calls.join(" · ")
			);
		}
	}
	println!("\n{:width$}  new  old", "case");
	let mut regressed = false;
	for (c, a) in cases.iter().zip(&answered) {
		let [new, old] = [&a[0], &a[1]].map(|a| a.verdicts.iter().filter(|v| v.met).count());
		let total = c.expect.len();
		let mark = match new < old {
			true => {
				regressed = true;
				format!("  ✗ {} {}", out.join(&c.id).join("new.md").display(), out.join(&c.id).join("old.md").display())
			}
			false => String::new(),
		};
		println!("{:width$}  {new}/{total}  {old}/{total}{mark}", c.id);
		for (i, e) in c.expect.iter().enumerate() {
			if !a[0].verdicts[i].met && !a[1].verdicts[i].met {
				println!("{:width$}    missed on both: {e}", "");
			}
		}
	}
	println!("\n${:.2}, answers in {}", answered.iter().flatten().map(|a| a.usd).sum::<f64>(), out.display());
	regressed
}

#[derive(Deserialize)]
struct Case {
	id: String,
	question: String,
	expect: Vec<String>,
}

#[derive(Deserialize)]
struct Verdict {
	met: bool,
	why: String,
}

struct Side {
	name: &'static str,
	system: String,
	info: String,
}

struct Answered {
	verdicts: Vec<Verdict>,
	usd: f64,
	answer_usd: f64,
	skill: bool,
	turns: u64,
	/// every tool the answer called, by name past its server prefix, and how often
	calls: Vec<(String, usize)>,
	tokens_in: u64,
	tokens_out: u64,
}

struct Server {
	child: Child,
	url: String,
	/// what the side's rev ships members, `None` before it shipped one
	plugin: Option<PathBuf>,
}

impl Drop for Server {
	fn drop(&mut self) {
		// a panic here, mid-unwind, would abort before the other server is killed
		if let Err(e) = self.child.kill().and_then(|()| self.child.wait().map(drop)) {
			eprintln!("killing the server at {}: {e}", self.url);
		}
	}
}

/// `target/eval/<side>/`: the system rev's tree with the info rev's corpus in it, built. Returns the binary.
fn build(root: &Path, side: &Side) -> PathBuf {
	let dir = root.join("target/eval").join(side.name);
	if dir.exists() {
		std::fs::remove_dir_all(&dir).expect("target/eval/ is ours");
	}
	std::fs::create_dir_all(&dir).expect("target/ is writable");
	extract(root, &side.system, &dir, &[]);
	for d in INFO {
		std::fs::remove_dir_all(dir.join(d)).unwrap_or_else(|e| panic!("{} has no {d}: {e}", side.system));
	}
	extract(root, &side.info, &dir, &INFO);

	// shared, so the two sides build their dependencies once
	let target = root.join("target/eval/cargo");
	let status = Command::new("cargo")
		.args(["build", "-p", "playbook_web", "--manifest-path"])
		.arg(dir.join("Cargo.toml"))
		.env("CARGO_TARGET_DIR", &target)
		.status()
		.expect("cargo runs");
	assert!(status.success(), "building playbook_web at {} failed", side.system);
	// both sides build to the same path, so each is moved out before the next
	let bin = dir.join("server");
	std::fs::copy(target.join("debug/playbook_web"), &bin).expect("cargo just built it");
	bin
}

fn extract(root: &Path, rev: &str, dir: &Path, paths: &[&str]) {
	// LFS objects would otherwise be fetched in full, and the server reads only markdown
	let mut archive = Command::new("git")
		.args(["archive", rev, "--"])
		.args(paths)
		.env("GIT_LFS_SKIP_SMUDGE", "1")
		.current_dir(root)
		.stdout(Stdio::piped())
		.spawn()
		.expect("git runs");
	let tar = Command::new("tar")
		.arg("-x")
		.current_dir(dir)
		.stdin(archive.stdout.take().expect("piped"))
		.status()
		.expect("tar runs");
	assert!(archive.wait().expect("git was spawned").success() && tar.success(), "extracting {rev} into {}", dir.display());
}

/// Runs the side's server from its own dir, so its `mcp.db` lands there, and gives each case a member.
fn serve(bin: &Path, side: &str, cases: &[Case], out: &Path) -> Server {
	let port = TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).expect("a free port").port();
	let url = format!("http://127.0.0.1:{port}");
	let log = std::fs::File::create(out.join(format!("{side}-server.log"))).expect("the run dir was just made");
	let dir = bin.parent().expect("built into its side's dir");
	let child = Command::new(bin)
		.env("PORT", port.to_string())
		.env("PUBLIC_URL", format!("{url}/playbook_mcp"))
		.env("PANEL_ASSERTION_KEYS", PANEL_ASSERTION_KEYS)
		.current_dir(dir)
		.stdout(log.try_clone().expect("a file handle clones"))
		.stderr(log)
		.spawn()
		.expect("the server was just built");
	let plugin = Some(dir.join("plugin")).filter(|p| p.exists());
	let mut server = Server { child, url, plugin };

	// /health is routed only once `app()` has made the schema
	let start = Instant::now();
	//LOOP: bounded by the timeout
	while !healthy(port) {
		if let Some(status) = server.child.try_wait().expect("the child is ours") {
			panic!("the {side} server exited with {status}; see {side}-server.log");
		}
		assert!(start.elapsed() < Duration::from_secs(30), "the {side} server is not healthy after 30s");
		std::thread::sleep(Duration::from_millis(100));
	}
	let db = rusqlite::Connection::open(dir.join("mcp.db")).expect("the server made it");
	for c in cases {
		// one member a case, so no case eats another's daily budget
		db.execute(
			"INSERT INTO tokens (hash, kind, sub, email, client_id, expires) VALUES (?1, 'access', ?2, ?3, 'eval', ?4)",
			(URL_SAFE_NO_PAD.encode(Sha256::digest(token(c))), &c.id, format!("{}@eval", c.id), i64::MAX),
		)
		.unwrap_or_else(|e| panic!("the {side} server's `tokens` changed shape: {e}"));
	}
	server.url.push_str("/playbook_mcp");
	server
}

fn healthy(port: u16) -> bool {
	let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) else {
		return false; // not listening yet
	};
	stream.write_all(b"GET /health HTTP/1.0\r\n\r\n").expect("the server accepted");
	let mut res = String::new();
	stream.read_to_string(&mut res).expect("the server answers in utf-8");
	res.starts_with("HTTP/1.0 200") || res.starts_with("HTTP/1.1 200")
}

fn token(c: &Case) -> String {
	format!("eval-{}", c.id)
}

fn ask(c: &Case, side: &Side, server: &Server, out: &Path) -> Answered {
	let cfg = |role: &str| out.join(".claude").join(format!("{}-{}-{role}", c.id, side.name));
	let authorization = format!("Bearer {}", token(c));
	let args: Vec<String> = match &server.plugin {
		Some(plugin) => {
			// the plugin as shipped, but for where its server is and whose token it carries
			let copy = cfg("plugin");
			let status = Command::new("cp").arg("-rT").arg(plugin).arg(&copy).status().expect("cp runs");
			assert!(status.success(), "copying {} to {}", plugin.display(), copy.display());
			let read = |p: &Path| -> Value {
				serde_json::from_str(&std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
			};
			let name = read(&copy.join(".claude-plugin/plugin.json"))["name"].as_str().expect("a plugin names itself").to_owned();
			let mut mcp = read(&copy.join(".mcp.json"));
			let servers = mcp["mcpServers"].as_object_mut().expect("the plugin's .mcp.json holds `mcpServers`");
			assert_eq!(servers.len(), 1, "the {} side's plugin connects one server", side.name);
			let (key, entry) = servers.iter_mut().next().expect("asserted one");
			entry["url"] = json!(server.url);
			entry["headers"] = json!({ "Authorization": authorization });
			let tools = format!("mcp__plugin_{name}_{key}");
			std::fs::write(copy.join(".mcp.json"), mcp.to_string()).expect("the copy was just made");
			// a member's client has the Skill tool; nothing else of Claude Code's would answer from the playbook
			[
				"--plugin-dir",
				copy.to_str().expect("utf-8 paths"),
				"--tools",
				"Skill",
				"--allowedTools",
				&format!("Skill,{tools}"),
			]
			.map(String::from)
			.into()
		}
		None => {
			let mcp = json!({ "mcpServers": { "service-arb": { "type": "http", "url": server.url, "headers": { "Authorization": authorization } } } });
			["--strict-mcp-config", "--mcp-config", &mcp.to_string(), "--tools", "", "--allowedTools", "mcp__service-arb"]
				.map(String::from)
				.into()
		}
	};
	let args: Vec<&str> = ["--model", ANSWER_MODEL].into_iter().chain(args.iter().map(String::as_str)).collect();
	let (answer, called) = claude(&cfg("answer"), out, &c.question, &args);
	let text = answer["result"].as_str().expect("a result is a string");
	let mut calls: Vec<(String, usize)> = Vec::new();
	for tool in &called {
		let tool = tool.rsplit("__").next().expect("rsplit yields at least one part").to_owned();
		match calls.iter_mut().find(|(t, _)| *t == tool) {
			Some((_, n)) => *n += 1,
			None => calls.push((tool, 1)),
		}
	}
	let usage = |k: &str| answer["usage"][k].as_u64().unwrap_or_else(|| panic!("claude reports `usage.{k}`"));
	let skill = calls.iter().any(|(t, _)| t == "Skill");

	let expectations: String = c.expect.iter().map(|e| format!("- {e}\n")).collect();
	let prompt = format!(
		"A member asked a question and got the answer below. For each expectation, in order, decide whether the answer meets it.\n\n## question\n\n{}\n\n## answer\n\n{text}\n\n## expectations\n\n{expectations}",
		c.question
	);
	let schema = json!({
		"type": "object",
		"properties": { "verdicts": { "type": "array", "items": { "type": "object", "properties": { "met": { "type": "boolean" }, "why": { "type": "string" } }, "required": ["met", "why"] } } },
		"required": ["verdicts"],
	});
	let (judged, _) = claude(&cfg("judge"), out, &prompt, &["--model", JUDGE_MODEL, "--tools", "", "--json-schema", &schema.to_string()]);
	let verdicts: Vec<Verdict> = serde_json::from_value(judged["structured_output"]["verdicts"].clone()).expect("the schema holds the judge to this shape");
	assert_eq!(
		verdicts.len(),
		c.expect.len(),
		"{} {}: the judge returned a verdict count unlike the expectations'",
		c.id,
		side.name
	);

	let met = verdicts.iter().filter(|v| v.met).count();
	let lines: String = c
		.expect
		.iter()
		.zip(&verdicts)
		.map(|(e, v)| format!("- {} {e} — {}\n", if v.met { "✓" } else { "✗" }, v.why))
		.collect();
	let md = format!(
		"# {} ({}): {}\n\n- system: {} · info: {}\n- score: {met}/{}\n- skill: {} · turns: {} · calls: {}\n\n## expectations\n\n{lines}\n## answer\n\n{text}\n",
		c.id,
		side.name,
		c.question,
		side.system,
		side.info,
		c.expect.len(),
		skill,
		answer["num_turns"],
		called.join(" "),
	);
	let dir = out.join(&c.id);
	std::fs::create_dir_all(&dir).expect("the run dir was just made");
	std::fs::write(dir.join(format!("{}.md", side.name)), md).expect("the run dir was just made");

	let usd = |v: &Value| v["total_cost_usd"].as_f64().expect("claude reports its cost");
	Answered {
		usd: usd(&answer) + usd(&judged),
		answer_usd: usd(&answer),
		skill,
		turns: answer["num_turns"].as_u64().expect("claude reports its turns"),
		tokens_in: usage("input_tokens") + usage("cache_creation_input_tokens") + usage("cache_read_input_tokens"),
		tokens_out: usage("output_tokens"),
		calls,
		verdicts,
	}
}

/// `claude -p` with nothing of this machine's setup: no CLAUDE.md, skills, hooks, plugins or MCP
/// servers but what `args` name. Returns its result, and the name of every tool it called.
fn claude(cfg: &Path, cwd: &Path, prompt: &str, args: &[&str]) -> (Value, Vec<String>) {
	std::fs::create_dir_all(cfg).expect("the run dir was just made");
	let env = |k: &str| std::env::var_os(k).unwrap_or_else(|| panic!("{k} is unset"));
	let mut child = Command::new("claude")
		.args(["-p", "--output-format", "stream-json", "--verbose", "--no-session-persistence"])
		.args(args)
		.env_clear()
		.env("PATH", env("PATH"))
		.env("HOME", env("HOME"))
		.env("CLAUDE_CODE_OAUTH_TOKEN", env("CLAUDE_CODE_OAUTH_TOKEN"))
		.env("CLAUDE_CONFIG_DIR", cfg)
		.env("ENABLE_CLAUDEAI_MCP_SERVERS", "false") // the account's own connectors, which a member's are not
		.current_dir(cwd)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect("claude is on PATH");
	child.stdin.take().expect("piped").write_all(prompt.as_bytes()).expect("claude reads its prompt");
	let res = child.wait_with_output().expect("claude was spawned");
	let events: Vec<Value> = String::from_utf8_lossy(&res.stdout)
		.lines()
		.map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("claude printed a line that is no json ({e}): {l}\n{}", String::from_utf8_lossy(&res.stderr))))
		.collect();
	let tools = events
		.iter()
		.filter(|e| e["type"] == "assistant")
		.flat_map(|e| e["message"]["content"].as_array().expect("an assistant message has content"))
		.filter(|b| b["type"] == "tool_use")
		.map(|b| b["name"].as_str().expect("a tool_use names its tool").to_owned())
		.collect();
	let result = events
		.into_iter()
		.rfind(|e| e["type"] == "result")
		.unwrap_or_else(|| panic!("claude ended with no result: {}", String::from_utf8_lossy(&res.stderr)));
	assert!(result["is_error"] == false, "claude failed: {}", result["result"]);
	(result, tools)
}

fn git(root: &Path, args: &[&str]) -> String {
	let out = Command::new("git").args(args).current_dir(root).output().expect("git runs");
	assert!(out.status.success(), "git {}: {}", args.join(" "), String::from_utf8_lossy(&out.stderr));
	String::from_utf8(out.stdout).expect("git prints utf-8").trim().to_owned()
}

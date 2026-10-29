#![feature(default_field_values)]
use std::{
	path::{Path, PathBuf},
	process::Command,
};

use ask_llm::Pick;
use clap::{Parser, Subcommand};
use v_utils::io::{ConfirmResult, confirmation};

mod call;
mod chatgpt;
mod cite_check;
mod doc;
mod eval;
mod photo_metadata;
mod skool;
mod watch;
mod yt;

/// Past this, `--execute` asks before spending.
const CONFIRM_USD: f64 = 1.;
/// ¢ per minute of recording, by the `ask_llm` `Footage` `watch` reads it as.
const SCREEN_CENTS_PER_MIN: f64 = 0.1;
const PHONE_CENTS_PER_MIN: f64 = 0.6;

#[derive(Parser)]
struct Cli {
	#[command(subcommand)]
	command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
	/// Capture each source we have no capture of yet into `ref/`: the given links, or where none, every one `ref/README.md` links.
	Pull {
		#[command(subcommand)]
		source: Source,
	},
	/// Hold the tree to its shape; exits non-zero where it breaks.
	Check {
		#[command(subcommand)]
		what: Check,
	},
	/// Print the path of each capture's recording, fetched beside the capture first where none is kept.
	Recording {
		#[arg(required = true)]
		captures: Vec<PathBuf>,
	},
	/// Write `<capture>/shown.md`, what the recording shows that its speech does not, for each capture nobody has
	/// watched yet; every one under `ref/` where none are given.
	Watch {
		#[command(flatten)]
		frames: Frames,
		captures: Vec<PathBuf>,
	},
	/// Report the metadata of every photo under a directory, as JSON.
	PhotoMetadata(photo_metadata::Options),
	/// Redo a machine transcription of every call capture under the paths. Prints each with what it
	/// would cost; only `--execute` runs it.
	ReTranscribe {
		#[command(subcommand)]
		what: What,
	},
	/// Answer playbook/eval.json through a member's client on this tree, uncommitted edits included, and
	/// on the last release; exits 1 when a case scores lower here.
	Eval {
		/// the old side's system rev, the last release where unset
		#[arg(long)]
		against: Option<String>,
		/// the rev both sides take `structured/{approved,suggested}` and `ref/` from, this tree's where unset
		#[arg(long)]
		info: Option<String>,
		/// answers in flight at once; each is a `claude` process of ~240 MB
		#[arg(long, default_value = "8")]
		jobs: std::num::NonZeroUsize,
		/// cases to run, all where none
		ids: Vec<String>,
	},
}

#[derive(Subcommand)]
enum Source {
	/// loom and fathom share pages; drive folders, vimeo and vocaroo through what the registry says of them
	Call { urls: Vec<String> },
	/// google docs, as their markdown export
	Doc { urls: Vec<String> },
	/// chatgpt shares, by link or id
	Chatgpt { urls: Vec<String> },
	/// every group's classroom, and the calls feed into the registry
	Skool,
	/// youtube channels, through the index `ref/youtube/README.md`
	Yt {
		#[command(subcommand)]
		step: Yt,
	},
}

#[derive(Subcommand)]
enum Yt {
	/// List every video of the channels `ref/README.md` links, in `ref/youtube/README.md`.
	Sync,
	/// Capture the given videos, or every one that list has unticked.
	Transcribe {
		#[command(flatten)]
		frames: Frames,
		urls: Vec<String>,
	},
}

#[derive(Subcommand)]
enum Check {
	/// every call capture, against `docs/ARCHITECTURE.md`'s "Call captures"
	Calls,
	/// every citation in `structured/`, against its capture in `ref/`
	Cites,
}

#[derive(Subcommand)]
enum What {
	/// whisper's transcript, re-pulled from the kept recording
	Audio(Targets),
	/// `shown.md`, re-watched
	Picture {
		#[command(flatten)]
		frames: Frames,
		#[command(flatten)]
		targets: Targets,
	},
}

#[derive(clap::Args)]
struct Targets {
	#[arg(required = true)]
	paths: Vec<PathBuf>,
	#[arg(long)]
	execute: bool,
}

/// Which frames of a recording are read.
#[derive(clap::Args)]
struct Frames {
	/// wherever the picture changes, not only where the speech says something is shown
	#[arg(long, conflicts_with = "every")]
	legacy: bool,
	/// least seconds between frames
	#[arg(long, default_value = "0.5")]
	every: f64,
}

impl From<Frames> for Pick {
	fn from(f: Frames) -> Self {
		match f.legacy {
			true => Pick::Changes,
			false => Pick::Likely { every: f.every },
		}
	}
}

fn main() {
	let root = repo_root();
	let rt = || tokio::runtime::Runtime::new().expect("a runtime builds");
	match Cli::parse().command {
		Cmd::Pull { source } => match source {
			Source::Call { urls } => call::pull(&root, &urls),
			Source::Doc { urls } => doc::pull(&root, &urls),
			Source::Chatgpt { urls } => chatgpt::pull(&root, &urls),
			Source::Skool => rt().block_on(skool::pull(&root)),
			Source::Yt { step: Yt::Sync } => rt().block_on(yt::sync(&root)),
			Source::Yt {
				step: Yt::Transcribe { frames, urls },
			} => rt().block_on(yt::transcribe(&root, frames.into(), &urls)),
		},
		Cmd::Check { what: Check::Calls } => eprintln!("{} call captures hold their shape", call::all(&root).len()),
		Cmd::Check { what: Check::Cites } => cite_check::run(&root),
		Cmd::Recording { captures } =>
			for media in call::recordings(&root, &captures) {
				println!("{}", media.display());
			},
		Cmd::Watch { frames, captures } => watch::run(&root, frames.into(), &captures),
		Cmd::PhotoMetadata(options) => photo_metadata::run(options),
		Cmd::ReTranscribe { what } => re_transcribe(&root, what),
		Cmd::Eval { against, info, jobs, ids } =>
			if eval::run(&root, against, info, jobs, ids) {
				std::process::exit(1);
			},
	}
}

/// The fields of a capture this reads.
struct Capture {
	path: PathBuf,
	source: String,
	minutes: f64,
	transcribed_by: String,
	read_by: String,
}

fn re_transcribe(root: &Path, what: What) {
	let (pick, Targets { paths, execute }) = match what {
		What::Audio(targets) => (None, targets),
		What::Picture { frames, targets } => (Some(Pick::from(frames)), targets),
	};
	let all = call::all(root);
	let captures: Vec<Capture> = paths
		.iter()
		.flat_map(|p| {
			let p = std::fs::canonicalize(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
			let under: Vec<&PathBuf> = all.iter().filter(|c| c.starts_with(&p)).collect();
			assert!(!under.is_empty(), "{} holds no call capture", p.display());
			under
		})
		.map(|p| read(p))
		.collect();
	let media: Vec<Option<PathBuf>> = match pick {
		Some(_) => call::recordings(root, &captures.iter().map(|c| c.path.clone()).collect::<Vec<_>>())
			.into_iter()
			.map(Some)
			.collect(),
		None => captures.iter().map(|_| None).collect(),
	};

	let mut todo = Vec::new();
	let mut usd = 0.;
	for (c, media) in captures.iter().zip(&media) {
		let rel = c.path.strip_prefix(root).expect("captures are found under the root").display();
		match plan(c, pick.zip(media.as_deref())) {
			Ok((cost, why)) => {
				println!("{:>8} {rel} — {why}", format!("${cost:.3}"));
				usd += cost;
				todo.push(c);
			}
			Err(skip) => println!("{:>8} {rel} — {skip}", "skip"),
		}
	}
	println!("{:>8} {} of {} captures", format!("${usd:.2}"), todo.len(), captures.len());
	if !execute || todo.is_empty() {
		return;
	}
	if usd > CONFIRM_USD && !matches!(confirmation(&format!("spend ~${usd:.2}?")).flush_blocking(), ConfirmResult::Yes) {
		return;
	}
	match pick {
		Some(pick) => {
			for c in &todo {
				let kept = c.path.with_extension("");
				if kept.join("shown.md").exists() {
					std::fs::remove_file(kept.join("shown.md")).expect("the capture's dir is ours");
				}
				if kept.join("frames").exists() {
					std::fs::remove_dir_all(kept.join("frames")).expect("the capture's dir is ours");
				}
			}
			watch::run(root, pick, &todo.iter().map(|c| c.path.clone()).collect::<Vec<_>>());
		}
		None => {
			// the kept recording waits where `call::pull` stages a fetch, which it then finds already there
			for c in &todo {
				let id = c.source.trim_end_matches('/').rsplit('/').next().expect("rsplit yields at least once");
				let staged = root.join("tmp/call-pull").join(id);
				assert!(!staged.exists(), "{} is already staged — a pull was cut short; move it back beside its capture", staged.display());
				std::fs::create_dir_all(staged.parent().expect("joined above")).expect("tmp/ is writable");
				std::fs::rename(c.path.with_extension(""), &staged).expect("tmp/ is on the same filesystem as ref/");
				std::fs::remove_file(&c.path).expect("the capture was just read");
			}
			// a drive, vimeo or vocaroo recording is named by the registry, so only a registry run can pull it
			call::pull(root, &[]);
		}
	}
}

/// `Ok((usd, what it is))` for a capture this would redo, `Err(why not)` for one it leaves. `picture`
/// is how a picture is picked, and its recording.
fn plan(c: &Capture, picture: Option<(Pick, &Path)>) -> Result<(f64, String), String> {
	match picture {
		None => {
			if !c.transcribed_by.starts_with("whisper") {
				return Err(format!("transcribed by {}, which a re-pull fetches again rather than transcribes", c.transcribed_by));
			}
			if c.read_by.starts_with("`/") {
				return Err("digested, and a re-pull drops the digest".into());
			}
			Ok((0., format!("{:.0} min through local whisper", c.minutes)))
		}
		Some((pick, media)) => {
			let video = Command::new("ffprobe")
				.args(["-v", "error", "-select_streams", "v", "-show_entries", "stream=index", "-of", "csv=p=0"])
				.arg(media)
				.output()
				.expect("ffprobe runs");
			assert!(video.status.success(), "ffprobe could not read {}", media.display());
			if video.stdout.iter().all(u8::is_ascii_whitespace) {
				return Err("audio only, nothing shown".into());
			}
			let (kind, rate) = match c.source.contains("drive.google.com") {
				true => ("phone clip", PHONE_CENTS_PER_MIN),
				false => ("screen", SCREEN_CENTS_PER_MIN),
			};
			// the pick decides what is read, so its cost is known only once it ran
			let at = match pick {
				Pick::Changes => "",
				Pick::Likely { .. } => "the legacy rate, ",
			};
			Ok((c.minutes * rate / 100., format!("{:.1} min of {kind} at {at}{rate}¢/min", c.minutes)))
		}
	}
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

fn read(path: &Path) -> Capture {
	let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
	let field = |key: &str| {
		text.lines()
			.find_map(|l| l.strip_prefix(&format!("- {key}: ")))
			.unwrap_or_else(|| panic!("{} states no `{key}:` — run `playbook check calls`", path.display()))
			.to_string()
	};
	let secs = field("duration")
		.split(':')
		.fold(0u64, |acc, p| acc * 60 + p.parse::<u64>().unwrap_or_else(|e| panic!("{}: duration: {e}", path.display())));
	Capture {
		path: path.to_path_buf(),
		source: field("source").trim_matches(['<', '>']).to_string(),
		minutes: secs as f64 / 60.,
		transcribed_by: field("transcribed by"),
		read_by: field("read by"),
	}
}

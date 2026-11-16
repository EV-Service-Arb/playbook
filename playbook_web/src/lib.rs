//! The playbook's `guides/`, `knowledge/structured/{approved,suggested}/` and `knowledge/references/` text, served in
//! slices to the Service-Arb members holding `sa:playbook:mcp:use`, behind the conductor. See `README.md`.

mod corpus;
mod tools;

use std::{
	path::{Path, PathBuf},
	sync::{Arc, LazyLock, Mutex},
};

use axum::{Router, extract::FromRef, routing::get};
use sa_auth::{Gate, Service, Verifier};

/// The conductor's `/mcp` instructions are composed of each document's description; Codex and ChatGPT read their first 512 characters
const INSTRUCTIONS: &str = "The service-arb playbook: dated, sourced knowledge on running Google Business Profiles into subcontracted trade work, used through the `service-arb` skill, which https://github.com/EV-Service-Arb/playbook installs.";

pub struct Config {
	/// the conductor's public keys, for the assertion on every request
	pub assertion_keys: sa_auth::Keys,
	pub db: PathBuf,
	pub daily_bytes: i64,
}

pub fn app(config: Config) -> Router {
	LazyLock::force(&corpus::CORPUS); // a malformed capture fails the start, not a member's call
	let state = State {
		verifier: Verifier::new(config.assertion_keys, Service::Playbook),
		db: Arc::new(Mutex::new(open(&config.db))),
		daily_bytes: config.daily_bytes,
	};
	let routes = tools::routes();
	let mut document = ev_lib::api::document("playbook", env!("CARGO_PKG_VERSION"), "/", &routes, corpus::DOCS, None);
	document.info.description = Some(INSTRUCTIONS.into());
	routes
		.iter()
		.fold(Router::new(), |r, route| r.route(route.path, route.handler()))
		.route("/openapi.json", ev_lib::api::serve(&document, Gate::USER.or_conductor()))
		.route("/health", get(|| async { "ok" }))
		.with_state(state)
}

fn open(path: &Path) -> rusqlite::Connection {
	let db = rusqlite::Connection::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
	db.pragma_update(None, "journal_mode", "WAL").unwrap();
	db.busy_timeout(std::time::Duration::from_secs(5)).unwrap();
	db.execute_batch("BEGIN IMMEDIATE").unwrap(); // a start that dies mid-migration leaves the old schema whole
	match db.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0)).unwrap() {
		0 => {
			let va_sso: bool = db.query_row("SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE name = 'clients')", [], |r| r.get(0)).unwrap();
			assert!(!va_sso, "{} is va_sso's, from before concierge; no server migrates it any more", path.display());
		}
		1 => db.execute_batch(include_str!("oauth_to_assertion.sql")).unwrap(),
		2 => {}
		v => panic!("{} is at schema version {v}, newer than this server", path.display()),
	}
	db.execute_batch(include_str!("schema.sql")).unwrap();
	db.execute_batch("COMMIT").unwrap();
	db
}

#[derive(Clone)]
struct State {
	verifier: Verifier,
	db: Arc<Mutex<rusqlite::Connection>>, // shortcut: one lock for every query; a pool if calls ever queue on it
	daily_bytes: i64,
}

impl FromRef<State> for Verifier {
	fn from_ref(s: &State) -> Self {
		s.verifier.clone()
	}
}

fn now() -> i64 {
	std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64
}

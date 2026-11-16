//! `search`, `read` and `guide`: what an agent reaches through the conductor's `/mcp`, each answered as text and
//! logged against the asker's daily budget.

use axum::{
	extract::State as Axum,
	http::{Method, StatusCode},
};
use ev_lib::api::{ErrorCode, JsonSchema, Params, Refusal, Route};
use sa_auth::{Gate, Mcp, User};
use sa_core::agent::Tool;
use serde::{Deserialize, Serialize};

use crate::{
	State,
	corpus::{CORPUS, Found},
	now,
};

const DAY: i64 = 24 * 60 * 60;

#[derive(Clone, Copy, Serialize, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Code {
	/// The corpus holds nothing for these arguments; the error says what would.
	Unanswerable,
	/// Today's bytes are spent; the budget resets at 00:00 UTC.
	OverBudget,
}

impl ErrorCode for Code {
	fn status(self) -> StatusCode {
		match self {
			Self::Unanswerable => StatusCode::UNPROCESSABLE_ENTITY,
			Self::OverBudget => StatusCode::TOO_MANY_REQUESTS,
		}
	}
}

pub(crate) fn routes() -> Vec<Route<State, Gate>> {
	vec![
		Route::new(Method::GET, "/search", Gate::USER.needs(Mcp::Use), search),
		Route::new(Method::GET, "/read", Gate::USER.needs(Mcp::Use), read),
		Route::new(Method::GET, "/guide", Gate::USER.needs(Mcp::Use), guide),
	]
}

#[derive(Deserialize, JsonSchema, Serialize)]
struct SearchArgs {
	/// a regex, case-insensitive; each `|` alternative is scored on its own
	pattern: String,
	/// a path prefix to search under, e.g. `knowledge/references/loom` or `knowledge/structured/approved/profile`
	scope: Option<String>,
	/// search what recordings showed on screen instead of what was said
	screen: Option<bool>,
}

#[derive(Deserialize, JsonSchema, Serialize)]
struct ReadArgs {
	/// a path as `search` returned it
	path: String,
	/// part of the header line of the chapter or section to read, e.g. `Selling calls` or `## facts`
	header: String,
	/// a line number inside the chapter, when the header text repeats in the file
	line: Option<usize>,
}

#[derive(Deserialize, JsonSchema, Serialize)]
struct GuideArgs {
	/// one of the guides listed
	area: String,
}

async fn search(Axum(state): Axum<State>, Tool: Tool, user: User, Params(args): Params<SearchArgs>) -> Result<String, Refusal<Code>> {
	state.answer(&user, "search", &args, || CORPUS.search(&args.pattern, args.scope.as_deref(), args.screen.unwrap_or(false)))
}

async fn read(Axum(state): Axum<State>, Tool: Tool, user: User, Params(args): Params<ReadArgs>) -> Result<String, Refusal<Code>> {
	state.answer(&user, "read", &args, || CORPUS.read(&args.path, &args.header, args.line))
}

async fn guide(Axum(state): Axum<State>, Tool: Tool, user: User, Params(args): Params<GuideArgs>) -> Result<String, Refusal<Code>> {
	state.answer(&user, "guide", &args, || CORPUS.guide(&args.area))
}

impl State {
	/// Refused once `user` has spent today's bytes; the call that crosses the budget is answered.
	fn answer(&self, user: &User, tool: &str, args: &impl Serialize, found: impl FnOnce() -> Result<Found, String>) -> Result<String, Refusal<Code>> {
		let today = now() - now() % DAY;
		let spent: i64 = self
			.db
			.lock()
			.unwrap()
			.query_row("SELECT COALESCE(SUM(bytes), 0) FROM calls WHERE sub = ?1 AND at >= ?2", (&user.sub, today), |r| r.get(0))
			.unwrap();
		if spent >= self.daily_bytes {
			return Err(Refusal::new(
				Code::OverBudget,
				format!("{} has used today's {} bytes; the budget resets at 00:00 UTC", user.email, self.daily_bytes),
			));
		}
		let (text, ids, error) = match found() {
			Ok(Found { text, ids }) => (text, ids, false),
			Err(e) => (e, vec![], true),
		};
		self.db
			.lock()
			.unwrap()
			.execute(
				"INSERT INTO calls (at, sub, email, tool, args, ids, bytes, error) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
				(now(), &user.sub, &user.email, tool, serde_json::to_string(args).unwrap(), ids.join(" "), text.len() as i64, error),
			)
			.expect("the calls table is created on start");
		match error {
			false => Ok(text),
			true => Err(Refusal::new(Code::Unanswerable, text)),
		}
	}
}

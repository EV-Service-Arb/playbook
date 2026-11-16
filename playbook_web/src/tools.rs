use std::sync::Arc;

use axum::http::request::Parts;
use rmcp::{
	ErrorData as McpError, RoleServer, ServerHandler,
	handler::server::{router::tool::ToolRouter, wrapper::Parameters},
	model::*,
	schemars,
	service::RequestContext,
	tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};

use crate::{
	Member, State,
	corpus::{CORPUS, Found, GUIDE_DESCRIPTION},
	now,
};

/// Codex and ChatGPT read the first 512 characters; claude.ai none
const INSTRUCTIONS: &str = "The service-arb playbook: dated, sourced knowledge on running Google Business Profiles into subcontracted trade work, used through the `service-arb` skill, which https://github.com/EV-Service-Arb/playbook installs.";

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
pub struct SearchArgs {
	/// a regex, case-insensitive; each `|` alternative is scored on its own
	pattern: String,
	/// a path prefix to search under, e.g. `knowledge/references/loom` or `knowledge/structured/profile`
	scope: Option<String>,
	/// search what recordings showed on screen instead of what was said
	screen: Option<bool>,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
pub struct ReadArgs {
	/// a path as `search` returned it
	path: String,
	/// part of the header line of the chapter or section to read, e.g. `Selling calls`
	header: String,
	/// a line number inside the chapter, when the header text repeats in the file
	line: Option<usize>,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
pub struct GuideArgs {
	/// one of the pages listed, or `people`
	area: String,
}

#[derive(Clone)]
pub struct Playbook {
	state: Arc<State>,
	tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Playbook {
	pub fn new(state: Arc<State>) -> Self {
		Self {
			state,
			tool_router: Self::tool_router(),
		}
	}

	#[tool(
		description = "Case-insensitive regex over the service-arb playbook. Each kind of source has its own slots, so none crowds out another: the manual (knowledge/structured), the course (lessons and the recordings they link), calls, videos, texts. Within a kind, a passage holding the rarer of the pattern's alternatives ranks first. Each hit gives path:line, its header, its label, the source URL and the lines around it; the files past the slots follow as path:line, for `read`. A pattern hitting most of the scope is refused, with the sub-scopes it hits.\n\nA label is who backs the passage, how they know (ran it, says so, heard it), when the information is from, and `effective`: what that is worth today, 0 to 1, higher is better. `unvetted`: nobody of ours has checked the claim. `unscored`: nothing says who speaks.",
		annotations(read_only_hint = true, open_world_hint = false))]
	async fn search(&self, Parameters(args): Parameters<SearchArgs>, ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
		Ok(self.answer(&ctx, "search", &args, CORPUS.search(&args.pattern, args.scope.as_deref(), args.screen.unwrap_or(false))))
	}

	#[tool(
		description = "One chapter of a capture or a digest (a `###`), or one section of a manual page, whole, with every line numbered. Never a whole file.",
		annotations(read_only_hint = true, open_world_hint = false)
	)]
	async fn read(&self, Parameters(args): Parameters<ReadArgs>, ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
		Ok(self.answer(&ctx, "read", &args, CORPUS.read(&args.path, &args.header, args.line)))
	}

	#[tool(description = GUIDE_DESCRIPTION, annotations(read_only_hint = true, open_world_hint = false))]
	async fn guide(&self, Parameters(args): Parameters<GuideArgs>, ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
		Ok(self.answer(&ctx, "guide", &args, CORPUS.guide(&args.area)))
	}

	fn answer(&self, ctx: &RequestContext<RoleServer>, tool: &str, args: &impl Serialize, found: Result<Found, String>) -> CallToolResult {
		let parts = ctx.extensions.get::<Parts>().expect("served over streamable http only");
		let Member { sub, email } = parts.extensions.get::<Member>().expect("the guard admits members only");
		let (text, ids, error) = match found {
			Ok(Found { text, ids }) => (text, ids, false),
			Err(e) => (e, vec![], true),
		};
		self.state
			.db
			.lock()
			.unwrap()
			.execute(
				"INSERT INTO calls (at, sub, email, tool, args, ids, bytes, error) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
				(now(), sub, email, tool, serde_json::to_string(args).unwrap(), ids.join(" "), text.len() as i64, error),
			)
			.expect("the calls table is created on start");
		match error {
			false => CallToolResult::success(vec![ContentBlock::text(text)]),
			true => CallToolResult::error(vec![ContentBlock::text(text)]),
		}
	}
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Playbook {
	fn get_info(&self) -> ServerConfig {
		ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
			.with_server_info(Implementation::from_build_env())
			.with_protocol_version(ProtocolVersion::V_2026_07_28)
			.with_instructions(INSTRUCTIONS)
	}
}

use std::sync::Arc;

use axum::http::request::Parts;
use rmcp::{
	ErrorData as McpError, RoleServer, ServerHandler,
	handler::server::{
		router::{prompt::PromptRouter, tool::ToolRouter},
		wrapper::Parameters,
	},
	model::*,
	prompt, prompt_handler, prompt_router, schemars,
	service::RequestContext,
	tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};

use crate::{
	Member, State,
	corpus::{ANSWERING, CORPUS, Found, GUIDE_DESCRIPTION, INSTRUCTIONS},
	now,
};

/// claude.ai drops a connector's `instructions`, so the rules that matter most ride here too
const SEARCH_DESCRIPTION: &str = "Case-insensitive regex over the service-arb playbook. Each kind of source has its own slots, so none crowds out another: structured/approved and structured/suggested notes, the course (lessons and the recordings they link), calls, Eric's videos, texts. Within a kind, a passage holding the rarer of your alternatives ranks first. Each hit gives path:line, its header, the date and reliability, the source URL and the lines around it; the files past the slots follow as path:line, for `read`. Search the words a speaker would say, alternated: `review(s)? (a|per) day|reviews? daily`. The course outranks any call. A member's failure story is an anecdote, not a rule. Newest wins only between equal sources. When nothing answers, say the corpus does not cover it.";

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
pub struct SearchArgs {
	/// a regex, case-insensitive. Alternate synonyms and likely mis-transcriptions with `|`
	pattern: String,
	/// a path prefix to search under, e.g. `ref/loom` or `structured/approved/profile`
	scope: Option<String>,
	/// search what recordings showed on screen instead of what was said
	screen: Option<bool>,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
pub struct ReadArgs {
	/// a path as `search` returned it
	path: String,
	/// part of the header line of the chapter or section to read, e.g. `Selling calls` or `## facts`
	header: String,
	/// a line number inside the chapter, when the header text repeats in the file
	line: Option<usize>,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
pub struct GuideArgs {
	section: String,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema)]
pub struct AskArgs {
	question: String,
}

#[derive(Clone)]
pub struct Playbook {
	state: Arc<State>,
	tool_router: ToolRouter<Self>,
	prompt_router: PromptRouter<Self>,
}

#[tool_router]
impl Playbook {
	pub fn new(state: Arc<State>) -> Self {
		Self { state, tool_router: Self::tool_router(), prompt_router: Self::prompt_router() }
	}

	#[tool(description = SEARCH_DESCRIPTION, annotations(read_only_hint = true, open_world_hint = false))]
	async fn search(&self, Parameters(args): Parameters<SearchArgs>, ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
		Ok(self.answer(&ctx, "search", &args, CORPUS.search(&args.pattern, args.scope.as_deref(), args.screen.unwrap_or(false))))
	}

	#[tool(
		description = "One chapter of a capture (a `###` under `## transcript` or `## summary`) or one section of a note, whole, with every line numbered. Never a whole file.",
		annotations(read_only_hint = true, open_world_hint = false)
	)]
	async fn read(&self, Parameters(args): Parameters<ReadArgs>, ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
		Ok(self.answer(&ctx, "read", &args, CORPUS.read(&args.path, &args.header, args.line)))
	}

	#[tool(description = GUIDE_DESCRIPTION, annotations(read_only_hint = true, open_world_hint = false))]
	async fn guide(&self, Parameters(args): Parameters<GuideArgs>, ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
		Ok(self.answer(&ctx, "guide", &args, CORPUS.guide(&args.section)))
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

/// a member's explicit way in, for clients that hide the tools until they decide to look
#[prompt_router]
impl Playbook {
	#[prompt(description = "Answer a question out of the service-arb playbook, never from general knowledge")]
	async fn ask(&self, Parameters(args): Parameters<AskArgs>) -> Vec<PromptMessage> {
		vec![PromptMessage::new_text(
			Role::User,
			format!("Answer out of the service-arb playbook, through its `guide`, `search` and `read` tools, never from general knowledge.\n\n{ANSWERING}\n\n---\n\n{}", args.question),
		)]
	}
}

#[tool_handler(router = self.tool_router)]
#[prompt_handler(router = self.prompt_router)]
impl ServerHandler for Playbook {
	fn get_info(&self) -> ServerConfig {
		ServerConfig::new(ServerCapabilities::builder().enable_tools().enable_prompts().build())
			.with_server_info(Implementation::from_build_env())
			.with_protocol_version(ProtocolVersion::V_2026_07_28)
			.with_instructions(INSTRUCTIONS)
	}
}

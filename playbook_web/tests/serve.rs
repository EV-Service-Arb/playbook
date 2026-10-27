//! Drives the server over streamable HTTP with a member's token, the way a member's client does.

use std::sync::{
	Arc, Mutex,
	atomic::{AtomicBool, AtomicI64, Ordering},
};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use evconcierge_projection::{EventSource, Member, Page, Pulled, ScopeEvent, ScopeEventBody, SourceError, UserEvent, UserEventKind};
use rmcp::{
	RoleClient, ServiceExt,
	model::{CallToolRequestParams, GetPromptRequestParams},
	service::RunningService,
	transport::{StreamableHttpClientTransport, streamable_http_client::StreamableHttpClientTransportConfig},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::watch;

const MEMBER: &str = "member@example.com";
const TOKEN: &str = "test-token";
const BASE: &str = "/playbook_mcp";
const PUBLIC_URL: &str = "https://sa.evinvest.ltd/playbook_mcp";
const PANEL: &str = "k1:BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=";
const STRANGER: &str = "k1:CAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg=";

#[tokio::test]
async fn read_returns_one_chapter() {
	serve("read", i64::MAX, Concierge::default(), async |url| {
		let client = connect(&url, TOKEN).await;
		for (path, header) in [("ref/loom/2026-09-18-group-call.md", "[03:34]"), ("structured/suggested/infra.md", "## facts")] {
			let (text, error) = call(&client, "read", json!({ "path": path, "header": header })).await;
			assert!(!error, "{text}");
			let headers = text.lines().filter(|l| l.split_once(" | ").is_some_and(|(_, l)| l.starts_with('#'))).count();
			assert_eq!(headers, 1, "{text}");
		}
	})
	.await;
}

/// Where tools stay hidden until the model decides to look (claude.ai), the instructions are what it decides
/// on, and `ask` is the member's explicit way in; both carry the answering rules, the decay inlined.
#[tokio::test]
async fn the_server_says_when_and_how_to_answer() {
	serve("prompt", i64::MAX, Concierge::default(), async |url| {
		let client = connect(&url, TOKEN).await;
		let instructions = client.peer_info().unwrap().instructions.clone().unwrap();
		assert!(instructions.contains("GBP") && instructions.contains("**sources**"), "{instructions}");
		let question = "how many reviews a day?";
		let res = client
			.get_prompt(GetPromptRequestParams::new("ask").with_arguments(json!({ "question": question }).as_object().unwrap().clone()))
			.await
			.unwrap();
		let text = res.messages.iter().map(|m| m.content.as_text().unwrap().text.as_str()).collect::<String>();
		assert!(text.ends_with(question) && text.contains("**sources**") && !text.contains("DECAY_DAYS"), "{text}");
	})
	.await;
}

/// The call that crosses the budget is answered; the request after it is refused until midnight UTC.
#[tokio::test]
async fn quota_overrun_is_429() {
	serve("quota", 1, Concierge::default(), async |url| {
		let client = connect(&url, TOKEN).await;
		call(&client, "guide", json!({ "section": "reviews" })).await;
		let res = reqwest::Client::new()
			.post(format!("{url}{BASE}"))
			.bearer_auth(TOKEN)
			.header("accept", "application/json, text/event-stream")
			.json(&json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" }))
			.send()
			.await
			.unwrap();
		assert_eq!(res.status(), 429);
		let retry: i64 = res.headers()["retry-after"].to_str().unwrap().parse().unwrap();
		assert!((1..=86_400).contains(&retry), "{retry}");
	})
	.await;
}

/// The panel forwards to the in-cluster address, so the Host it sends is that one, not `PUBLIC_URL`'s.
#[tokio::test]
async fn any_host_is_served() {
	serve("host", i64::MAX, Concierge::default(), async |url| {
		let res = reqwest::Client::new()
			.post(format!("{url}{BASE}"))
			.bearer_auth(TOKEN)
			.header("host", "playbook-web.personal:59082")
			.header("accept", "application/json, text/event-stream")
			.json(&json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "t", "version": "0" } } }))
			.send()
			.await
			.unwrap();
		assert_eq!(res.status(), 200, "{}", res.text().await.unwrap());
	})
	.await;
}

/// A client with nothing but the MCP url finds the authorization server through the path-inserted
/// metadata of RFC 9728 and RFC 8414.
#[tokio::test]
async fn no_token_leads_to_the_authorization_server() {
	serve("unauthorized", i64::MAX, Concierge::default(), async |url| {
		let http = reqwest::Client::new();
		let res = http.post(format!("{url}{BASE}")).json(&json!({})).send().await.unwrap();
		assert_eq!(res.status(), 401);
		let challenge = res.headers()["www-authenticate"].to_str().unwrap();
		assert_eq!(challenge, format!("Bearer resource_metadata=\"https://sa.evinvest.ltd/.well-known/oauth-protected-resource{BASE}\""));
		let resource: Value = http.get(format!("{url}/.well-known/oauth-protected-resource{BASE}")).send().await.unwrap().json().await.unwrap();
		assert_eq!(resource["resource"], PUBLIC_URL);
		assert_eq!(resource["authorization_servers"], json!([PUBLIC_URL]));
		let server: Value = http.get(format!("{url}/.well-known/oauth-authorization-server{BASE}")).send().await.unwrap().json().await.unwrap();
		assert_eq!(server["issuer"], PUBLIC_URL);
		assert_eq!(server["authorization_endpoint"], format!("{PUBLIC_URL}/authorize"));
		assert_eq!(server["token_endpoint"], format!("{PUBLIC_URL}/token"));
	})
	.await;
}

/// `/authorize` answers only a request the panel signed, for this very method and path, and
/// only a holder of `sa:playbook:mcp:use` gets the consent page; anyone else is sent to the panel
/// to ask for it, carrying the way back.
#[tokio::test]
async fn authorize_takes_the_panels_word_for_who_is_asking() {
	serve("assertion", i64::MAX, Concierge::default(), async |url| {
		let (client_id, _) = register(&url, "Claude Code").await;
		let authorize = authorize_url(&url, &client_id);
		let http = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();
		let get = |assertion: Option<String>| {
			let req = http.get(authorize.clone());
			async move { if let Some(a) = assertion { req.header(sa_auth::HEADER, a) } else { req }.send().await.unwrap() }
		};
		for bad in [None, Some("not.a.jws".into()), Some(assertion(STRANGER, "u1", "GET", true)), Some(assertion(PANEL, "u1", "POST", true))] {
			let res = get(bad).await;
			assert_eq!(res.status(), 401);
			assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");
		}
		assert_eq!(get(Some(assertion(PANEL, "u1", "GET", true))).await.status(), 200);

		let res = get(Some(assertion(PANEL, "u1", "GET", false))).await;
		assert_eq!(res.status(), 303);
		let to = reqwest::Url::parse("https://sa.evinvest.ltd").unwrap().join(res.headers()["location"].to_str().unwrap()).unwrap();
		assert_eq!(to.path(), "/access/");
		let q: std::collections::HashMap<_, _> = to.query_pairs().into_owned().collect();
		assert_eq!(q["need"], sa_auth::Mcp::Use.as_str());
		assert_eq!(q["continue"], format!("{}?{}", authorize.path(), authorize.query().unwrap()));
	})
	.await;
}

/// Cancel is a GET back to `/authorize`: the client learns `access_denied` at its own registered
/// redirect, whoever is signed in, and nothing is issued.
#[tokio::test]
async fn cancel_tells_the_client_access_denied() {
	serve("deny", i64::MAX, Concierge::default(), async |url| {
		let http = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();
		let (client_id, _) = register(&url, "Claude Code").await;
		let mut deny = authorize_url(&url, &client_id);
		deny.query_pairs_mut().append_pair("decision", "deny");
		for assertion in [None, Some(assertion(PANEL, "u1", "GET", false))] {
			let req = http.get(deny.clone());
			let res = if let Some(a) = assertion { req.header(sa_auth::HEADER, a) } else { req }.send().await.unwrap();
			assert_eq!(res.status(), 303);
			assert_eq!(res.headers()["location"], "http://localhost:64461/callback?error=access_denied&state=s1");
		}
		let mut stranger = deny.clone();
		stranger.query_pairs_mut().clear().extend_pairs(deny.query_pairs().map(|(k, v)| (k.clone(), if k == "redirect_uri" { "https://evil.example/cb".into() } else { v })));
		let res = http.get(stranger).send().await.unwrap();
		assert_eq!(res.status(), 400);
		assert!(res.headers().get("location").is_none());
	})
	.await;
}

/// Whatever goes wrong on the browser's side of the flow is a page saying what to do next, with
/// the reason in small print; the JSON endpoints stay JSON.
#[tokio::test]
async fn browser_errors_are_pages() {
	serve("errors", i64::MAX, Concierge::default(), async |url| {
		let http = reqwest::Client::new();
		let (client_id, _) = register(&url, "Claude Code").await;
		let mut unknown = authorize_url(&url, &client_id);
		unknown.query_pairs_mut().clear().append_pair("client_id", "nobody");
		let page = |res: reqwest::Response| async move {
			assert_eq!(res.status(), 400);
			assert_eq!(res.headers()["content-type"], "text/html; charset=utf-8");
			res.text().await.unwrap()
		};
		insta::assert_snapshot!("missing_params", redact(&page(http.get(unknown).send().await.unwrap()).await));
		let spent = http.post(format!("{url}{BASE}/authorize")).header(sa_auth::HEADER, assertion(PANEL, "u1", "POST", true)).form(&[("nonce", "made-up")]).send();
		insta::assert_snapshot!("spent_nonce", redact(&page(spent.await.unwrap()).await));
		let res = http.post(format!("{url}{BASE}/token")).form(&[("grant_type", "nope"), ("client_id", "x")]).send().await.unwrap();
		assert_eq!(res.headers()["content-type"], "application/json");
	})
	.await;
}

/// The code comes only from the consent form's POST, by the member it was shown to, once. A
/// native client's loopback redirect is registered without a port and matches whichever it
/// listens on (RFC 8252 §7.3), as Claude Code's client metadata does.
#[tokio::test]
async fn a_code_is_issued_only_on_consent() {
	serve("consent", i64::MAX, Concierge::default(), async |url| {
		let http = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();
		let (client_id, verifier) = register(&url, "<b>Claude</b> & co").await;
		let res = http.get(authorize_url(&url, &client_id)).header(sa_auth::HEADER, assertion(PANEL, "u1", "GET", true)).send().await.unwrap();
		assert_eq!(res.status(), 200);
		assert!(res.headers().get("location").is_none());
		let page = res.text().await.unwrap();
		insta::assert_snapshot!("consent", redact(&page));
		let nonce = page.split(r#"name="nonce" value=""#).nth(1).unwrap().split('"').next().unwrap().to_owned();
		let css = page.split(r#"rel="stylesheet" href=""#).nth(1).unwrap().split('"').next().unwrap();
		let res = http.get(format!("{url}{}", css.strip_prefix("https://sa.evinvest.ltd").unwrap())).send().await.unwrap();
		assert_eq!((res.status().as_u16(), &res.headers()["content-type"]), (200, &"text/css".parse::<reqwest::header::HeaderValue>().unwrap()));
		assert!(res.headers()["cache-control"].to_str().unwrap().contains("immutable"));

		let post = |sub: &str, nonce: &str| {
			http.post(format!("{url}{BASE}/authorize")).header(sa_auth::HEADER, assertion(PANEL, sub, "POST", true)).form(&[("nonce", nonce)]).send()
		};
		assert_eq!(post("u1", "made-up").await.unwrap().status(), 400);
		assert_eq!(post("u2", &nonce).await.unwrap().status(), 400);
		let res = post("u1", &nonce).await.unwrap();
		assert!(res.status().is_redirection(), "{}", res.status());
		assert_eq!(post("u1", &nonce).await.unwrap().status(), 400);

		let back = reqwest::Url::parse(res.headers()["location"].to_str().unwrap()).unwrap();
		assert_eq!(back.as_str().split('?').next(), Some("http://localhost:64461/callback"));
		assert_eq!(back.query_pairs().find(|(k, _)| k == "state").unwrap().1, "s1");
		let code = back.query_pairs().find(|(k, _)| k == "code").unwrap().1.into_owned();
		let tokens: Value = http
			.post(format!("{url}{BASE}/token"))
			.form(&[
				("grant_type", "authorization_code"),
				("client_id", &client_id),
				("code", &code),
				("code_verifier", &verifier),
				("redirect_uri", "http://localhost:64461/callback"),
			])
			.send()
			.await
			.unwrap()
			.json()
			.await
			.unwrap();
		let client = connect(&url, tokens["access_token"].as_str().unwrap()).await;
		let (text, error) = call(&client, "guide", json!({ "section": "reviews" })).await;
		assert!(!error, "{text}");
	})
	.await;
}

/// A refresh asks concierge's word, not `/authorize`'s: once the permission is gone or the account
/// suspended, it is refused and every token of the member stops working.
#[tokio::test]
async fn a_refresh_is_refused_once_concierge_withdraws_access() {
	let concierge = Concierge::default();
	for sub in ["u1", "u2"] {
		concierge.join(sub);
		concierge.grant(sub, &[sa_auth::Mcp::Use.as_str()]);
	}
	serve("withdraw", i64::MAX, concierge.clone(), async |url| {
		let withdrawals: [(&str, Withdraw); 2] = [("u1", |c, sub| c.grant(sub, &[])), ("u2", Concierge::suspend)];
		for (sub, withdraw) in withdrawals {
			concierge.synced().await;
			let (client_id, tokens) = sign_in(&url, sub).await;
			let res = refresh(&url, &client_id, &tokens).await;
			assert_eq!(res.status(), 200, "{sub}");
			let tokens: Value = res.json().await.unwrap();

			withdraw(&concierge, sub);
			concierge.synced().await;
			let res = refresh(&url, &client_id, &tokens).await;
			assert_eq!(res.status(), 400, "{sub}");
			assert_eq!(res.json::<Value>().await.unwrap()["error"], "invalid_grant", "{sub}");
			assert_eq!(ping(&url, tokens["access_token"].as_str().unwrap()).await, 401, "{sub}");
		}
	})
	.await;
}

/// While concierge cannot be heard from, a refresh is turned away without spending the token, which
/// works again once the projection catches up.
#[tokio::test]
async fn a_refresh_waits_while_concierge_is_unreachable() {
	let concierge = Concierge::default();
	concierge.join("u1");
	concierge.grant("u1", &[sa_auth::Mcp::Use.as_str()]);
	concierge.0.down.store(true, Ordering::SeqCst);
	serve("unreachable", i64::MAX, concierge.clone(), async |url| {
		let (client_id, tokens) = sign_in(&url, "u1").await;
		let res = refresh(&url, &client_id, &tokens).await;
		assert_eq!(res.status(), 503);
		assert_eq!(res.json::<Value>().await.unwrap()["error"], "temporarily_unavailable");

		concierge.0.down.store(false, Ordering::SeqCst);
		concierge.synced().await;
		assert_eq!(refresh(&url, &client_id, &tokens).await.status(), 200);
	})
	.await;
}

/// The consent flow end to end for `sub`: a fresh client, and the token response it ends with.
async fn sign_in(url: &str, sub: &str) -> (String, Value) {
	let http = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();
	let (client_id, verifier) = register(url, "Claude Code").await;
	let page = http.get(authorize_url(url, &client_id)).header(sa_auth::HEADER, assertion(PANEL, sub, "GET", true)).send().await.unwrap().text().await.unwrap();
	let nonce = page.split(r#"name="nonce" value=""#).nth(1).unwrap().split('"').next().unwrap();
	let res = http.post(format!("{url}{BASE}/authorize")).header(sa_auth::HEADER, assertion(PANEL, sub, "POST", true)).form(&[("nonce", nonce)]).send().await.unwrap();
	let back = reqwest::Url::parse(res.headers()["location"].to_str().unwrap()).unwrap();
	let code = back.query_pairs().find(|(k, _)| k == "code").unwrap().1.into_owned();
	let tokens = http
		.post(format!("{url}{BASE}/token"))
		.form(&[("grant_type", "authorization_code"), ("client_id", &client_id), ("code", &code), ("code_verifier", &verifier), ("redirect_uri", "http://localhost:64461/callback")])
		.send()
		.await
		.unwrap()
		.json()
		.await
		.unwrap();
	(client_id, tokens)
}

async fn refresh(url: &str, client_id: &str, tokens: &Value) -> reqwest::Response {
	reqwest::Client::new()
		.post(format!("{url}{BASE}/token"))
		.form(&[("grant_type", "refresh_token"), ("client_id", client_id), ("refresh_token", tokens["refresh_token"].as_str().unwrap())])
		.send()
		.await
		.unwrap()
}

/// the MCP endpoint's status for a ping with `token`
async fn ping(url: &str, token: &str) -> u16 {
	reqwest::Client::new()
		.post(format!("{url}{BASE}"))
		.bearer_auth(token)
		.header("accept", "application/json, text/event-stream")
		.json(&json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" }))
		.send()
		.await
		.unwrap()
		.status()
		.as_u16()
}

type Withdraw = fn(&Concierge, &str);

/// concierge's two streams as the projection pulls them, scope `sa` only, written by the test.
#[derive(Clone, Default)]
struct Concierge(Arc<Streams>);

struct Streams {
	scope: Mutex<Vec<ScopeEvent>>,
	users: Mutex<Vec<UserEvent>>,
	down: AtomicBool,
	users_after: AtomicI64,
	/// on every scope pull, which starts a cycle: its `after`, and the last lifecycle pull's
	pulled: watch::Sender<(i64, i64)>,
}

impl Default for Streams {
	fn default() -> Self {
		Self { scope: Mutex::default(), users: Mutex::default(), down: AtomicBool::default(), users_after: AtomicI64::new(-1), pulled: watch::Sender::new((-1, -1)) }
	}
}

impl Concierge {
	fn join(&self, sub: &str) {
		let member = Member {
			email: MEMBER.into(),
			email_verified: true,
			username: sub.into(),
			preferred_name: "M".into(),
			status: "active".into(),
			kyc_level: 0,
			token_version: 1,
			lifecycle_sequence: 1,
		};
		self.push_scope(sub, ScopeEventBody::MemberJoined { user_id: sub.into(), auth_subject: subject(sub), member });
	}

	fn grant(&self, sub: &str, permissions: &[&str]) {
		self.push_scope(sub, ScopeEventBody::GrantsChanged { user_id: sub.into(), auth_subject: subject(sub), permissions: permissions.iter().map(|p| (*p).to_owned()).collect() });
	}

	fn suspend(&self, sub: &str) {
		let mut users = self.0.users.lock().unwrap();
		let sequence = 2 + users.iter().filter(|e| e.user_id == sub).count() as u64; // past MEMBER_JOINED's lifecycle_sequence
		let event_id = format!("user-{}", users.len());
		users.push(UserEvent {
			event_id,
			kind: UserEventKind::Suspended,
			user_id: sub.into(),
			auth_subject: subject(sub),
			sequence,
			occurred_at: 0,
			kyc_level: 0,
			email: MEMBER.into(),
			email_verified: true,
			token_version: 1,
			role: "investor".into(),
		});
	}

	fn push_scope(&self, sub: &str, body: ScopeEventBody) {
		let mut scope = self.0.scope.lock().unwrap();
		let sequence = 1 + scope.iter().filter(|e| e.key_user() == Some(sub)).count() as u64;
		let position = scope.len() as i64 + 1;
		scope.push(ScopeEvent { event_id: format!("scope-{position}"), position, scope: "sa".into(), occurred_at: 0, sequence, body });
	}

	/// Until a cycle that pulled everything written so far has finished: the pulls of the cycle
	/// after it start past all of it.
	async fn synced(&self) {
		let (scope, users) = (self.0.scope.lock().unwrap().len() as i64, self.0.users.lock().unwrap().len() as i64);
		self.0.pulled.subscribe().wait_for(|&(s, u)| s >= scope && u >= users).await.unwrap();
	}
}

trait KeyUser {
	fn key_user(&self) -> Option<&str>;
}

impl KeyUser for ScopeEvent {
	fn key_user(&self) -> Option<&str> {
		match &self.body {
			ScopeEventBody::MemberJoined { user_id, .. } | ScopeEventBody::GrantsChanged { user_id, .. } => Some(user_id),
			ScopeEventBody::ScopeChanged(_) => None,
		}
	}
}

fn subject(sub: &str) -> String {
	format!("google:{sub}")
}

#[async_trait::async_trait]
impl EventSource for Concierge {
	async fn pull_user_events(&self, after: i64, limit: u32) -> Result<Page<Pulled<UserEvent>>, SourceError> {
		if self.0.down.load(Ordering::SeqCst) {
			return Err(SourceError::Unavailable("down".into()));
		}
		let users = self.0.users.lock().unwrap();
		let events: Vec<_> = users.iter().skip(after as usize).take(limit as usize).cloned().map(Pulled::Known).collect();
		self.0.users_after.store(after, Ordering::SeqCst);
		Ok(Page { next_position: after + events.len() as i64, events })
	}

	async fn pull_scope_events(&self, after: i64, limit: u32) -> Result<Page<Pulled<ScopeEvent>>, SourceError> {
		if self.0.down.load(Ordering::SeqCst) {
			return Err(SourceError::Unavailable("down".into()));
		}
		let scope = self.0.scope.lock().unwrap();
		let events: Vec<_> = scope.iter().skip(after as usize).take(limit as usize).cloned().map(Pulled::Known).collect();
		self.0.pulled.send_replace((after, self.0.users_after.load(Ordering::SeqCst)));
		Ok(Page { next_position: after + events.len() as i64, events })
	}
}

/// the page with what changes per run or per stylesheet held still
fn redact(page: &str) -> String {
	let page = regex::Regex::new(r#"name="nonce" value="[^"]+""#).unwrap().replace_all(page, r#"name="nonce" value="[nonce]""#);
	let page = regex::Regex::new(r"client_id(=|%3D)[A-Za-z0-9_-]{43}").unwrap().replace_all(&page, "client_id$1[client]");
	regex::Regex::new(r"connect\.[0-9a-f]+\.css").unwrap().replace_all(&page, "connect.[hash].css").into_owned()
}

/// a DCR client with a portless loopback redirect, and the PKCE verifier for `authorize_url`
async fn register(url: &str, name: &str) -> (String, String) {
	let reg: Value = reqwest::Client::new()
		.post(format!("{url}{BASE}/register"))
		.json(&json!({ "redirect_uris": ["http://localhost/callback"], "client_name": name }))
		.send()
		.await
		.unwrap()
		.json()
		.await
		.unwrap();
	(reg["client_id"].as_str().unwrap().to_owned(), "a-verifier-long-enough-to-be-one-0123456789".to_owned())
}

fn authorize_url(url: &str, client_id: &str) -> reqwest::Url {
	reqwest::Url::parse_with_params(&format!("{url}{BASE}/authorize"), [
		("response_type", "code"),
		("client_id", client_id),
		("redirect_uri", "http://localhost:64461/callback"),
		("code_challenge", &URL_SAFE_NO_PAD.encode(Sha256::digest("a-verifier-long-enough-to-be-one-0123456789"))),
		("code_challenge_method", "S256"),
		("state", "s1"),
	])
	.unwrap()
}

/// what the panel puts on a request to `<BASE>/authorize`, signed with `key`
fn assertion(key: &str, sub: &str, method: &str, may_use: bool) -> String {
	let signer: sa_auth::Signer = key.parse().unwrap();
	signer.sign(&sa_auth::Assertion {
		aud: sa_auth::Service::Playbook,
		sub: sub.into(),
		email: MEMBER.into(),
		email_verified: true,
		name: "M".into(),
		permissions: may_use.then(|| sa_auth::Mcp::Use.as_str()).into_iter().collect(),
		method: method.into(),
		path: format!("{BASE}/authorize"),
		exp: i64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()).unwrap() + sa_auth::TTL,
	})
}

/// Runs `body` against a fresh server, served under `BASE` and projecting `concierge`, that knows one
/// member, holding the access token `TOKEN`. `body` gets the origin.
async fn serve(name: &str, daily_bytes: i64, concierge: Concierge, body: impl AsyncFnOnce(String) -> ()) {
	let dir = std::env::temp_dir().join(format!("playbook_web-test-{}", std::process::id()));
	std::fs::create_dir_all(&dir).unwrap();
	let db = dir.join(format!("{name}.db"));
	if db.exists() {
		std::fs::remove_file(&db).unwrap();
	}
	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
	let config = playbook_web::Config {
		public_url: PUBLIC_URL.into(),
		panel_keys: PANEL.parse::<sa_auth::Signer>().unwrap().public().parse().unwrap(),
		db: db.clone(),
		daily_bytes,
		sync_every: std::time::Duration::from_millis(5),
	};
	let (app, sync) = playbook_web::app(config, concierge).await;
	rusqlite::Connection::open(&db)
		.unwrap()
		.execute(
			"INSERT INTO tokens (hash, kind, sub, email, client_id, expires) VALUES (?1, 'access', 'u0', ?2, 'test', ?3)",
			(URL_SAFE_NO_PAD.encode(Sha256::digest(TOKEN)), MEMBER, i64::MAX),
		)
		.unwrap();
	tokio::select! {
		r = async { axum::serve(listener, app).await } => panic!("the server stopped: {r:?}"),
		() = sync => panic!("the projection stopped"),
		() = body(url) => {}
	}
}

async fn connect(url: &str, token: &str) -> RunningService<RoleClient, ()> {
	let transport = StreamableHttpClientTransport::from_config(StreamableHttpClientTransportConfig::with_uri(format!("{url}{BASE}")).auth_header(token));
	().serve(transport).await.unwrap()
}

async fn call(client: &RunningService<RoleClient, ()>, tool: &str, args: Value) -> (String, bool) {
	let res = client
		.call_tool(CallToolRequestParams::new(tool.to_owned()).with_arguments(args.as_object().unwrap().clone()))
		.await
		.unwrap();
	let text = res.content.iter().map(|c| c.as_text().unwrap().text.as_str()).collect::<Vec<_>>().join("\n");
	(text, res.is_error == Some(true))
}


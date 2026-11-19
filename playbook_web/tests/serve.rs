//! Calls the server as the conductor's forward does: one signed assertion per request, naming the member.

use std::path::PathBuf;

use axum::{
	Router,
	body::{Body, to_bytes},
	http::{Request, StatusCode},
};
use sa_auth::{Assertion, Principal, Service, Signer, User};
use serde_json::{Value, json};
use tower::ServiceExt as _;

const CONDUCTOR: &str = "k1:BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=";

#[tokio::test]
async fn read_returns_one_chapter() {
	let app = app("read", i64::MAX);
	for (path, header) in [
		("knowledge/references/loom/2026-09-18-group-call/src.md", "[03:34]"),
		("knowledge/structured/contractors/splits.md", "## the split"),
	] {
		let (status, text) = get(&app, member("u0"), "/read", json!({ "path": path, "header": header })).await;
		assert_eq!(status, StatusCode::OK, "{text}");
		let headers = text.lines().filter(|l| l.split_once(" | ").is_some_and(|(_, l)| l.starts_with('#'))).count();
		assert_eq!(headers, 1, "{text}");
	}
}

/// The procedure is the `service-arb` skill's: the document names it, and each operation says only what it returns.
/// What changes with the corpus rides on the operations: the pages, the people, what every hit is worth today, and
/// where a broad pattern could narrow to.
#[tokio::test]
async fn the_server_carries_data_and_the_skill_the_procedure() {
	let app = app("contract", i64::MAX);
	let doc = document(&app).await;
	let instructions = doc["info"]["description"].as_str().unwrap();
	assert!(instructions.contains("`service-arb` skill"), "{instructions}");
	assert!(instructions.len() <= 512, "{instructions}");
	let description = |path: &str| doc["paths"][path]["get"]["description"].as_str().unwrap().to_owned();
	let guide = description("/guide");
	assert!(guide.contains("- contractors/splits: splits"), "{guide}");

	let (status, people) = get(&app, member("u0"), "/guide", json!({ "area": "people" })).await;
	assert_eq!(status, StatusCode::OK, "{people}");
	assert!(people.contains("| Eric |"), "{people}");

	// r[verify trust.arithmetic+2]
	let (status, page) = get(&app, member("u0"), "/guide", json!({ "area": "contractors/splits" })).await;
	assert_eq!(status, StatusCode::OK, "{page}");
	assert!(page.contains(" · init 0."), "{page}");
	assert!(page.contains(" · norm 0."), "{page}");
	assert!(!page.contains("<!--"), "a citation is served as what it is worth, never as what that is computed from:\n{page}");

	let (status, text) = get(&app, member("u0"), "/search", json!({ "pattern": "the" })).await;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{text}");
	let error = serde_json::from_str::<Value>(&text).unwrap()["error"].as_str().unwrap().to_owned();
	assert!(error.lines().filter(|l| l.starts_with("- knowledge/")).count() >= 2, "{error}");
}

/// Every operation is the member's own tool on the conductor's `/mcp`, opened by `sa:playbook:mcp:use`, and states
/// what it refuses with.
#[tokio::test]
async fn the_document_lists_three_tools_behind_the_playbook_permission() {
	let doc = document(&app("document", i64::MAX)).await;
	let paths = doc["paths"].as_object().unwrap();
	assert_eq!(paths.keys().collect::<Vec<_>>(), ["/guide", "/read", "/search"]);
	for (path, item) in paths {
		let op = &item["get"];
		assert_eq!(op["x-permission"], "sa:playbook:mcp:use", "{path}");
		assert_eq!(op["x-tool"], true, "{path}");
		for code in ["unanswerable", "over_budget", "unauthenticated", "forbidden"] {
			assert!(op["x-errors"].get(code).is_some(), "{path}: {code} in {}", op["x-errors"]);
		}
	}
}

/// The call that crosses a member's budget is answered; theirs after it is refused until midnight UTC, and
/// another member's is not.
#[tokio::test]
async fn the_budget_is_each_members_own() {
	let app = app("budget", 1);
	let guide = json!({ "area": "reviews" });
	assert_eq!(get(&app, member("u0"), "/guide", guide.clone()).await.0, StatusCode::OK);
	let (status, text) = get(&app, member("u0"), "/guide", guide.clone()).await;
	assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{text}");
	assert_eq!(serde_json::from_str::<Value>(&text).unwrap()["code"], "over_budget");
	assert_eq!(get(&app, member("u1"), "/guide", guide).await.0, StatusCode::OK);
}

/// The log names who asked as the assertion does, refused calls of the corpus included.
#[tokio::test]
async fn a_call_is_logged_under_the_assertions_sub() {
	let (app, db) = app_at("log", i64::MAX);
	get(&app, member("u7"), "/guide", json!({ "area": "reviews" })).await;
	get(&app, member("u7"), "/guide", json!({ "area": "nowhere" })).await;
	let logged: Vec<(String, String, String, bool)> = rusqlite::Connection::open(db)
		.unwrap()
		.prepare("SELECT sub, email, tool, error FROM calls ORDER BY rowid")
		.unwrap()
		.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
		.unwrap()
		.collect::<Result<_, _>>()
		.unwrap();
	let row = |error| ("u7".to_owned(), "u7@example.com".to_owned(), "guide".to_owned(), error);
	assert_eq!(logged, [row(false), row(true)]);
}

/// The conductor vendors the committed document; `PLAYBOOK_GEN=1` writes it, otherwise a stale one fails.
#[tokio::test]
async fn the_committed_document_is_the_served_one() {
	let served = serde_json::to_string_pretty(&document(&app("committed", i64::MAX)).await).unwrap() + "\n";
	let path = concat!(env!("CARGO_MANIFEST_DIR"), "/openapi.json");
	match std::env::var_os("PLAYBOOK_GEN") {
		Some(_) => std::fs::write(path, served).unwrap(),
		None => assert!(
			std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}; `PLAYBOOK_GEN=1 cargo t -p playbook_web` writes it")) == served,
			"{path} is not the served document: `PLAYBOOK_GEN=1 cargo t -p playbook_web`"
		),
	}
}

fn app(name: &str, daily_bytes: i64) -> Router {
	app_at(name, daily_bytes).0
}

/// A fresh server on its own database.
fn app_at(name: &str, daily_bytes: i64) -> (Router, PathBuf) {
	let dir = std::env::temp_dir().join(format!("playbook_web-test-{}", std::process::id()));
	std::fs::create_dir_all(&dir).unwrap();
	let db = dir.join(format!("{name}.db"));
	if db.exists() {
		std::fs::remove_file(&db).unwrap();
	}
	let config = playbook_web::Config {
		assertion_keys: CONDUCTOR.parse::<Signer>().unwrap().public().parse().unwrap(),
		db: db.clone(),
		daily_bytes,
	};
	(playbook_web::app(config), db)
}

fn member(sub: &str) -> Principal {
	Principal::User(User {
		sub: sub.into(),
		email: format!("{sub}@example.com"),
		email_verified: true,
		name: "M".into(),
		permissions: ["sa:playbook:mcp:use"].into_iter().collect(),
		actor: None,
	})
}

async fn document(app: &Router) -> Value {
	let (status, text) = get(app, Principal::Conductor, "/openapi.json", json!({})).await;
	assert_eq!(status, StatusCode::OK, "{text}");
	serde_json::from_str(&text).unwrap()
}

/// `GET path?query`, asserted for `principal` as the conductor signs it: the path, never the query.
async fn get(app: &Router, principal: Principal, path: &str, query: Value) -> (StatusCode, String) {
	let assertion = Assertion {
		aud: Service::Playbook,
		principal,
		method: "GET".into(),
		path: path.into(),
		exp: i64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()).unwrap() + sa_auth::TTL,
	};
	let req = Request::get(format!("{path}?{}", serde_urlencoded::to_string(&query).unwrap()))
		.header(sa_auth::HEADER, CONDUCTOR.parse::<Signer>().unwrap().sign(&assertion))
		.body(Body::empty())
		.unwrap();
	let res = app.clone().oneshot(req).await.unwrap();
	let status = res.status();
	(status, String::from_utf8(to_bytes(res.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap())
}

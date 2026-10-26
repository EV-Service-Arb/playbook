const DAILY_BYTES: i64 = 2_000_000; // ~500k tokens a member a day

#[tokio::main]
async fn main() {
	let env = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("{k} is unset"));
	let environment = env("APP_ENV");
	let _sentry = ev_lib::error_monitoring::init(&ev_lib::error_monitoring::Config {
		dsn: std::env::var("SENTRY_DSN").ok(),
		traces_sample_rate: ev_lib::error_monitoring::Config::traces_sample_rate_for(&environment),
		environment,
		release: ev_lib::error_monitoring::release_name!().map(|r| r.into_owned()),
		service: Some("playbook-web".into()),
	});
	{
		use tracing_subscriber::{EnvFilter, fmt, prelude::*};
		tracing_subscriber::registry()
			.with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
			.with(fmt::layer())
			.with(ev_lib::error_monitoring::tracing_layer()) // errors become Sentry events; below that, breadcrumbs
			.init();
	}
	let config = playbook_web::Config {
		public_url: env("PUBLIC_URL").trim_end_matches('/').to_owned(),
		panel_keys: env("PANEL_ASSERTION_KEYS").parse().unwrap_or_else(|e| panic!("PANEL_ASSERTION_KEYS: {e}")),
		db: "mcp.db".into(),
		daily_bytes: DAILY_BYTES,
		sync_every: std::time::Duration::from_secs(10),
	};
	let concierge = evconcierge_projection::GrpcSource::connect_lazy(&env("CONCIERGE_GRPC_ADDR"), playbook_web::SA, &env("CONCIERGE_CLIENT_SECRET"), std::time::Duration::from_secs(10))
		.unwrap_or_else(|e| panic!("CONCIERGE_GRPC_ADDR: {e}"));
	let (app, sync) = playbook_web::app(config, concierge).await;
	let addr = format!("0.0.0.0:{}", env("PORT"));
	let listener = tokio::net::TcpListener::bind(&addr).await.unwrap_or_else(|e| panic!("{addr}: {e}"));
	tracing::info!("serving on {addr}");
	tokio::select! {
		r = axum::serve(listener, app).with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() }) => r.unwrap(),
		() = sync => unreachable!("the projection loops for ever"),
	}
}

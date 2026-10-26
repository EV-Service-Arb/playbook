//! concierge's word on the members of scope `sa`, kept by pulling its streams (`evconcierge_projection`)
//! with the panel's relying-party credential. The refresh grant reads it (`auth.rs`).

use std::{
	convert::Infallible,
	sync::{Arc, atomic::Ordering},
};

use evconcierge_projection::{Cycle, EventSource, Handler, Projection, ProjectionConfig, ScopeEvent, ScopeEventBody, SqliteProjectionStore, UserEvent, UserEventKind};
use sqlx::SqlitePool;

use crate::{State, now};

/// the panel's relying party at concierge, whose credential pulls, and the scope it reads
pub const SA: &str = "sa";

/// Never returns: a cycle that fails is retried on the next tick, and the refresh grant tells a stale projection by `State::synced`.
pub(crate) async fn run(state: Arc<State>, pool: SqlitePool, concierge: impl EventSource) {
	let every = state.config.sync_every;
	let projection = Projection::new(SqliteProjectionStore::new(pool), Members(state.clone()), concierge, ProjectionConfig::default());
	if let Err(e) = projection.replay_parked().await {
		eprintln!("concierge projection: replaying parked events: {e}");
	}
	loop {
		match projection.cycle(&mut Cycle::default()).await {
			Ok(()) => state.synced.store(now(), Ordering::SeqCst),
			Err(e) => eprintln!("concierge projection: {e}"),
		}
		tokio::time::sleep(every).await;
	}
}

struct Members(Arc<State>);

#[async_trait::async_trait]
impl Handler for Members {
	type Error = Infallible;

	async fn on_user_event(&self, event: &UserEvent) -> Result<(), Infallible> {
		let status = match event.kind {
			UserEventKind::Suspended => "disabled",
			UserEventKind::Reinstated => "active",
			UserEventKind::Created | UserEventKind::KycChanged | UserEventKind::SessionsRevoked | UserEventKind::RoleChanged | UserEventKind::PermissionsChanged => return Ok(()),
		};
		let updated = self.0.db.lock().unwrap().execute("UPDATE members SET status = ?1 WHERE auth_subject = ?2", (status, &event.auth_subject)).unwrap();
		assert_eq!(updated, 1, "{}: a lifecycle event the engine should have parked until MEMBER_JOINED", event.auth_subject);
		Ok(())
	}

	async fn on_scope_event(&self, event: &ScopeEvent) -> Result<(), Infallible> {
		if event.scope != SA {
			return Ok(());
		}
		let db = self.0.db.lock().unwrap();
		match &event.body {
			ScopeEventBody::MemberJoined { user_id, auth_subject, member } => {
				db.execute(
					// concierge states a joining member's set as empty; GRANTS_CHANGED follows when it is not
					"INSERT INTO members (auth_subject, user_id, status, permissions) VALUES (?1, ?2, ?3, '[]')
					 ON CONFLICT (auth_subject) DO UPDATE SET user_id = excluded.user_id, status = excluded.status, permissions = excluded.permissions",
					(auth_subject, user_id, &member.status),
				)
				.unwrap();
			}
			ScopeEventBody::GrantsChanged { auth_subject, permissions, .. } => {
				let updated = db.execute("UPDATE members SET permissions = ?1 WHERE auth_subject = ?2", (serde_json::to_string(permissions).unwrap(), auth_subject)).unwrap();
				assert_eq!(updated, 1, "{auth_subject}: GRANTS_CHANGED before its MEMBER_JOINED, which is sequence 1 of the same key");
			}
			ScopeEventBody::ScopeChanged(_) => {}
		}
		Ok(())
	}
}

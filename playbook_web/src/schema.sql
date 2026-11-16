CREATE TABLE IF NOT EXISTS calls (
	at INTEGER NOT NULL,
	sub TEXT NOT NULL, -- the assertion's: concierge's user id
	email TEXT NOT NULL,
	tool TEXT NOT NULL,
	args TEXT NOT NULL, -- json
	ids TEXT NOT NULL, -- space-separated `path:line`, or the guide section
	bytes INTEGER NOT NULL,
	error INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS calls_by_sub ON calls (sub, at);
PRAGMA user_version = 2;

# playbook_web

The playbook's data for members: a remote MCP server that answers from `guides/`, `structured/{approved,suggested}/`
and `ref/**/*.md`, in slices, to the Service-Arb members holding `sa:playbook:mcp:use`. `build.rs` bakes that text
into the binary, so the server reads nothing from disk per request and nothing else of the repo
reaches the image. `ref/cases/` states no source and no date, so it stays out.

Its tools are the instruments of the `service-arb` skill, which `plugin/` ships with this server's address
(`docs/ARCHITECTURE.md#answering`).

## Tools

| tool | returns |
|---|---|
| `search(pattern, scope?, screen?)` | case-insensitive regex hits, each with path:line, header, label and source URL, two per file. Each kind (notes, course, calls, videos, texts) has its own slots and slots it leaves empty go to the others. Within a kind, chunks rank by which alternatives of the pattern they hold, rare ones weighing more (BM25's idf, each term present or not). The files past the slots are listed by path:line. `screen` searches what recordings showed instead of what was said. A pattern hitting 40% of the files in scope is refused, naming the directories below the scope it hits and how much of each |
| `read(path, header, line?)` | one `###` chapter of a capture or one `##` section of a note, never a whole file |
| `guide(area)` | one `guides/<area>.md` |

A label is who said it, their base, the date and the effective reliability, all from the same
`Doc::effective` that ranks. A call's base is its speaker's, so its label gives the date and the age
term instead. `search`'s description carries the base tables, which `build.rs` reads out of
`docs/ARCHITECTURE.md#reliability`; `guide`'s lists each guide by its opening line,
`# <area>: <what it covers>`.

The `instructions` are one sentence naming the skill: Codex and ChatGPT read only their first 512
characters, and claude.ai none. The server has no prompts.

## Sign-in

The panel at `sa.evinvest.ltd` forwards `<base>/*` and the two `.well-known/oauth-*<base>` paths
unchanged. On `<base>/authorize` (GET and POST) it adds `x-sa-assertion`, its signed word for who
is asking and what of the playbook they may do (`sa_auth`), bound to that method and path.

```
client ──/register (DCR) or a CIMD url──► GET /authorize
   client or redirect_uri unknown ──► 400 page, nothing redirected
   decision=deny (Cancel) ──► redirect_uri?error=access_denied
   assertion: missing or refused ──► 401 page · without sa:playbook:mcp:use ──► panel /access/?need=…&continue=<this url>
   ok ──► consent page: client, account (switch), redirect, form with a nonce (10 min, single use, bound to sub + this request)
browser ──POST /authorize, nonce──► assertion again, nonce spent for the same sub ──► code ──► redirect_uri
client ──/token (PKCE S256)──► access token (1h) + refresh token (rotated on use, 30d from /authorize)
client ──/token refresh──► concierge's word on the member, as the projection holds it:
   not heard from concierge in 5 min, or an event of theirs parked ──► 503, the refresh token kept
   not a member · suspended · without sa:playbook:mcp:use ──► 400 invalid_grant, every token of theirs deleted
client ──<base>, Bearer──► guard: token live · under today's byte budget (else 429)
```

A code is only ever issued by the POST. Who the member is (concierge's `sub`) and the permission
are decided at `/authorize` by the panel's assertion, and again at every refresh by concierge's own
streams: the server pulls scope `sa`'s `ScopeEvents` and `UserEvents` as the panel's relying party
(`evconcierge_projection`, every 10s) into `members`. Revoking `sa:playbook:mcp:use` or suspending the
account bites within the access token's hour; rotating still keeps the deadline its `/authorize`
set. Tokens and nonces are opaque; only their sha256 is stored.

The browser's pages are `src/pages.rs`, on `ev_lib_classes`' tables; `build.rs` runs `tailwindcss`
over them into `<base>/assets/connect.<hash>.css`. Asking for access is the panel's `/access`.

## Configuration

| env | |
|---|---|
| `PORT` | set by the image |
| `APP_ENV` | set by the image, `production`: Sentry's environment |
| `PUBLIC_URL` | where members reach it, `https://sa.evinvest.ltd/playbook_mcp`: the MCP endpoint, the issuer and the resource. Its path is `<base>`: `/authorize`, `/token` and `/register` sit under it, the metadata at `/.well-known/oauth-{protected-resource,authorization-server}<base>` (RFC 9728, 8414), `/health` at the root |
| `PANEL_ASSERTION_KEYS` | the panel's assertion keys, `<kid>:<base64 Ed25519 public key>`, comma-separated while one rotates |
| `CONCIERGE_GRPC_ADDR` | concierge's gRPC, `http://concierge.ev-banking.svc.cluster.local:55670` |
| `CONCIERGE_CLIENT_SECRET` | the client secret of concierge's relying party `sa`: the value concierge hashes as `RP_CLIENT_SECRET_SA`, which the panel holds too |

Missing any fails the start. `SENTRY_DSN` is optional: with it, `error`s go to Sentry. Logs are
`info` and up on stdout, `RUST_LOG` overrides. The database is `mcp.db` in the working directory, `/data` in
the image. `members` and the engine's `concierge_projection_*` are rebuilt from concierge by emptying
them together, never one side alone (the projection crate's README, Rebuilding).

## Reading the log

```sh
kubectl -n personal exec deploy/playbook-web -- sqlite3 /data/mcp.db \
  "SELECT datetime(at, 'unixepoch'), email, tool, args, ids, bytes FROM calls ORDER BY at DESC LIMIT 50"
```

`calls`: `at` (unix seconds), `sub` (concierge's user id), `email`, `tool`, `args` (json), `ids` (the `path:line` of each hit
returned, or the guide), `bytes` (what the member got back), `error`. The full schema is
`src/schema.sql`; `calls_va_sso` is the log from before concierge, by email.

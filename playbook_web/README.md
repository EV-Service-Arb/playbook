# playbook_web

The playbook's data for members: three operations behind the conductor that answer from `guides/`,
`knowledge/structured/{approved,suggested}/` and `knowledge/references/**/*.md`, in slices, to the Service-Arb members holding
`sa:playbook:mcp:use`. The conductor's `/mcp` lists them as tools (`playbook_search`, `playbook_read`, `playbook_guide`). `build.rs` bakes that text
into the binary, so the server reads nothing from disk per request and nothing else of the repo
reaches the image. `knowledge/references/cases/` states no source and no date, so it stays out.

They are the instruments of the `service-arb` skill, which `plugin/` ships with the conductor's `/mcp` address
(`docs/ARCHITECTURE.md#answering`).

## Operations

| operation | returns |
|---|---|
| `GET /search?pattern&scope&screen` | case-insensitive regex hits, each with path:line, header, label and source URL, two per file. Each kind (notes, course, calls, videos, texts) has its own slots and slots it leaves empty go to the others. Within a kind, chunks rank by which alternatives of the pattern they hold, rare ones weighing more (BM25's idf, each term present or not). The files past the slots are listed by path:line. `screen` searches what recordings showed instead of what was said. A pattern hitting 40% of the files in scope is refused, naming the directories below the scope it hits and how much of each |
| `GET /read?path&header&line` | one `###` chapter of a capture or one `##` section of a note, never a whole file |
| `GET /guide?area` | one `guides/<area>.md` |

A label is who said it, their base, the date and the effective reliability, all from the same
`Doc::effective` that ranks. A call's base is its speaker's, so its label gives the date and the age
term instead. `search`'s description carries the base tables, which `build.rs` reads out of
`docs/ARCHITECTURE.md#reliability`; `guide`'s lists each guide by its opening line,
`# <area>: <what it covers>`.

Each answers text, or refuses with `unanswerable` (the corpus holds nothing for those arguments; the error
says what would) or `over_budget`. The document's description is one sentence naming the skill: the conductor
composes its `/mcp` instructions from it, and Codex and ChatGPT read only their first 512 characters.

## Who is asking

Every request carries the conductor's assertion (`sa_auth`, `x-sa-assertion`), and nothing else is trusted: a
row opens to a person holding `sa:playbook:mcp:use`, `/openapi.json` to them and to the conductor, whose catalog
pulls it. Signing clients in is the conductor's `/mcp`. A member is the assertion's `sub` (concierge's user id): the
budget is today's `bytes` of theirs in `calls`, and the call that crosses it is the last one answered until 00:00 UTC.

## Configuration

| env | |
|---|---|
| `PORT` | set by the image |
| `APP_ENV` | set by the image, `production`: Sentry's environment |
| `PANEL_ASSERTION_KEYS` | the conductor's assertion keys, `<kid>:<base64 Ed25519 public key>`, comma-separated while one rotates |

Missing any fails the start. `SENTRY_DSN` is optional: with it, `error`s go to Sentry. Logs are
`info` and up on stdout, `RUST_LOG` overrides. The database is `mcp.db` in the working directory, `/data` in
the image.

## Reading the log

```sh
kubectl -n personal exec deploy/playbook-web -- sqlite3 /data/mcp.db \
  "SELECT datetime(at, 'unixepoch'), email, tool, args, ids, bytes FROM calls ORDER BY at DESC LIMIT 50"
```

`calls`: `at` (unix seconds), `sub` (concierge's user id), `email`, `tool`, `args` (json), `ids` (the `path:line` of each hit
returned, or the guide), `bytes` (what the member got back), `error`. The full schema is
`src/schema.sql`; `calls_va_sso` is the log from before concierge, by email.

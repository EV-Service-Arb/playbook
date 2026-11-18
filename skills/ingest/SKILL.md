---
name: ingest
description: Bring something new into the playbook from a checkout — a source URL or channel to register and pull, a recording with no `content.md` yet, a digest to integrate into the manual, the owner's notes or chats, a new person, a todo or a fact about how our own team runs, or a contradiction in the manual to resolve. Use when handed any of these, when `playbook check` lists a recording nothing reads yet, on "digest this", "distill this", or as `/ingest <step> <path>`.
argument-hint: "[capture|digest|propose|integrate|notes|people|internal|resolve] [<path or url>...]"
---

# ingest

Only in a checkout. The formats are [knowledge/README.md](../../knowledge/README.md); read it first.

Every step reads `knowledge/` through a fresh `playbook view <dir>`, a directory outside the checkout, where each
citation carries what it is worth ([trust](../../knowledge/README.md#trust)). Where the task's information goes
stale faster or slower than its modules suggest, view it with `--half-life`. Every edit goes to `knowledge/`
itself, at the same `path:line`, citations copied from there.

A source goes down these in order, and each step reads only what the one before wrote:

| step | from | to | read |
|---|---|---|---|
| capture | a URL, a channel | `<capture>.md`, `<capture>/shown.md` | [capture.md](capture.md) |
| digest | the capture and `shown.md` | `<capture>/content.md` | [digest.md](digest.md) |
| propose | `content.md` | `knowledge/log/digest/<date>-<capture>.md` | [propose.md](propose.md) |
| integrate | the proposal | `knowledge/structured/` | [integrate.md](integrate.md) |

Commit after each step. One capture per run of a step: batched, every one of them gets worse.

What does not come as a source:

| what arrived | read |
|---|---|
| the owner's notes, the owner's chats | [notes.md](notes.md) |
| a person | [people.md](people.md) |
| about us: a todo, how the team runs | [internal.md](internal.md) |
| a contradiction in the manual | [resolve.md](resolve.md) |

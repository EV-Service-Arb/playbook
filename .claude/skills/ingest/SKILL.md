---
name: ingest
description: Bring something new into the playbook from a checkout — a source URL or channel to register and pull, a call capture whose `read by:` says `nothing yet`, the owner's notes or chats, a new person, a todo or a fact about how our own team runs. Use when handed any of these, when call-pull.rs says a recording was left unread, or as `/ingest call-digest <path>`.
argument-hint: "[pull|call-digest|notes|people|internal] [<path or url>...]"
---

# ingest

Only in a checkout. What arrived picks the procedure, or the argument names it:

| what arrived | read |
|---|---|
| a source URL, a channel | [pull.md](pull.md) |
| a capture its platform never read | [call-digest.md](call-digest.md) |
| the owner's notes, the owner's chats | [notes.md](notes.md) |
| a person | [people.md](people.md) |
| about us: a todo, how the team runs | [internal.md](internal.md) |

Working from a recording rather than its transcript: [recordings.md](recordings.md).

Several at once run in this order: pull, commit, digest, notes, then [distill](../distill/SKILL.md)
over what was pulled. Each step reads what the one before wrote.

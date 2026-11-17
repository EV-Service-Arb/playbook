<!--Reference: https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html-->
# Architecture

## Overview

A manual on service arbitrage — running Google Business Profiles into subcontracted trade work — that
an agent answers from, and the machinery that keeps it true: every line of it traces to the second
someone said it, who they were, and when it was so.

It is a framework for text. Text has no compiler, so the repository gives it one: wherever a rule
about `knowledge/` can be checked, `corpus::compile` checks it and nothing is released that fails;
what cannot be checked is an invariant in [docs/spec/](spec/).

```
   loom · fathom · youtube · chatgpt · skool · discord
                     │  pull                                    code
                     ▼
 references/<platform>/<capture>.md          raw. never edited
                     │  watch                                   code
                     ▼
              <capture>/shown.md             what the picture shows
                     │  digest                                  skill
                     ▼
              <capture>/content.md           the whole, as chapters of attributed points
                     │  propose                                 skill
                     ▼
 log/digest/<date>-<capture>.md              what it should change in the manual
                     │  integrate                               skill
                     ▼
 structured/**                               the manual: conclusions, evidence beneath
                     │  compile                                 code, at build
                     ▼
              who · trust · date, per claim
                     │  search · read · guide                   server, at the moment asked
                     ▼
              effective = trust × γ(age)
```

Each step reads only what the one above it wrote. Two directions follow from that:

- **the manual is filed by where it is used**, `references/` by where it came from. A result someone
  got sits under the method that got it; how to verify a profile and how to get reviews are separate
  pages however often one call mixed them
- **nothing derived is written down.** A citation states who, how they know and when; what that is
  worth is computed when it is read. A number typed into a note is stale the day its table moves

## Layers

**`knowledge/references/`** — what a source said. One file per source, stating its own source id,
where it came from and when it was recorded. A capture is identified by that id and not by its path,
so a file is free to be named and placed after what it *says* — and a source that reorders or renames
upstream is moved, not pulled twice. Re-running a puller over an existing capture is a no-op; a stale
one is re-pulled by deleting it.

```
knowledge/references/
├── README.md                     # the source list — every source, and which of them a puller reaches
├── <platform>/<capture>.md       # raw, as pulled. loom · fathom · drive · vimeo · vocaroo · youtube/<who>
├── <platform>/<capture>/         # beside it: what was made of it, each file naming what made it
│   ├── recording.<ext>           # as the host serves it — gitignored, bar what LFS keeps
│   ├── shown.md                  # what is on screen that the speech does not say, by `playbook watch`
│   ├── frames/<secs>.jpg         # the frame each line of `shown.md` was read off — LFS
│   └── content.md                # the whole recording, as chapters of attributed points, by the digest step
├── docs/<title>.md               # google docs, exported as markdown
├── research/<id>.md              # chatgpt shares, one `## user` / `## assistant` section per turn
├── youtube/README.md             # every video each channel lists, dated, ticked once captured — `playbook pull yt sync`
└── skool_<group>/                # a classroom, via `social_networks` — `gmbpp`, and the `cheap` group
    ├── README.md                 # what upstream holds, and the day it was last compared to this
    └── course/<NN-module>/       # a directory per module, a file per lesson, both in upstream's order
```

Two things a capture cannot state about itself are written beside it instead, since a capture is never
edited: **how far behind upstream it is**, in `skool_<group>/README.md`, rewritten every run with the
day of the run; and **what a source says that this tree has no capture of**, printed by the puller for
`knowledge/references/README.md` to be told about by hand.

### Recordings

```md
# <title>

- source: <https://www.loom.com/share/<id>>        # or fathom, drive, vimeo, vocaroo, youtube
- recorded: · duration: · transcribed by: · pulled by:

## summary

<the platform's, verbatim, where it wrote one>

## transcript

### [02:39](https://www.loom.com/share/<id>?t=159) Location Spoofing Strategy

plain prose, in paragraphs, no stamps in it
```

- a capture keeps the finest stamp its platform gives: the platform's titled chapters, or an untitled
  header per paragraph where it wrote none. Those headers are the only seconds anything may cite
- the transcript is the platform's; where it has none, the puller transcribes the audio itself, and
  `transcribed by:` says so
- a recording nobody speaks in is captured all the same: `transcribed by:` ends `— nobody speaks`
- a recording is read through `playbook recording <capture.md>`, which hands back the one beside the
  capture, fetching it from its host first when there is none
- every line of `shown.md` links the second it describes and embeds the frame at that second, so the
  claim can be checked by eye; its header names the model, the day and what it cost
- chapters of our own, and everything else we read into a recording, live in `content.md`

**`knowledge/structured/`** — the manual. One tree, filed by the thing being operated, in the order
the business meets it. A bullet an agent wrote opens `(?)` until the owner vets it.
**`knowledge/internal/`** — what we know about our own team and its operations. Read in a checkout,
never baked into the member image.
**`knowledge/log/`** — what follows from the notes rather than from a source: what a digest was to
change in the manual, how a contradiction was resolved.

The formats of all of them, the trust tables and the style are [knowledge/README.md](../knowledge/README.md).

## Pullers

Every tool here is a subcommand of the `playbook` crate at the root of the cargo workspace, so each
dependency is pinned once for all of them. `corpus/` beside it is the compiler of `knowledge/`: `playbook check`
prints what it refuses, and `playbook_web` bakes what it resolves. `playbook <cmd>` is `cargo r -p playbook -- <cmd>`, run
from anywhere inside the checkout. Each `pull <platform>` takes source URLs as arguments, or with
none reads `knowledge/references/README.md` for links of its platform and fetches the ones missing from disk. So
adding a source is pasting its link into `knowledge/references/README.md` and re-running. `pull yt` alone goes
through an index: `sync` lists every video of the channels `knowledge/references/README.md` links into
`knowledge/references/youtube/README.md`, and `transcribe` captures what that list has unticked, or only the listed
videos it is given.

`knowledge/references/README.md` is the only registry. A capture quotes its own source URL and a
transcript quotes every link that was said out loud, so a puller that scanned the tree
would feed on its own output. A puller that *finds* a link — one lesson pointing at
another platform's recording — prints it rather than following it, and the registry
grows by hand. The one exception is the group's calls feed: what it posts is the list's own
material, so `pull skool` writes each recording into the registry itself.

```
playbook/src/
├── call.rs            # pull call: loom / fathom share page, drive folder → the platform's summary, chapters and transcript, or whisper's where there is none; `recording`: the one way to a recording, kept or fetched
├── watch.rs           # watch, not a puller: a recording, through `ask_llm`'s `Client::watch`, frames only where the speech says something is shown (`--legacy`: wherever the picture changes) → `shown.md`
├── doc.rs             # pull doc: google doc → its markdown export
├── chatgpt.rs         # pull chatgpt: headless chromium → the DOM's turns
├── yt.rs              # pull yt sync: channels → the dated list of their videos; pull yt transcribe: description and captions in citable blocks, then the picture through `watch`
├── skool.rs           # pull skool: skool's classroom → the course tree, module by module and lesson by lesson; the calls feed → the registry's group calls
├── photo_metadata.rs  # photo-metadata: exif of our own photos, and synthetic sidecars
└── eval.rs            # eval
```

`playbook re-transcribe <audio|picture> <path>...`
lists every call capture under the paths with the estimated cost of redoing it; `picture` is
estimated at the `--legacy` rate either way, as the default's cost is known only after the pick. It runs only with
`--execute`, and asks first when the total is past $1. Audio is redone only for a whisper transcript
no `content.md` reads yet, since a re-pull moves the stamps one cites. `playbook eval` answers `playbook/eval.json`
through a member's client, on this tree and on the last release, each side behind a conductor of its
own (`PANEL_BIN`, in dev sign-in) whose `/mcp` the client connects to, and judges each answer against
the case's expectations.

> each expectation is stated as succinctly as possible, to capture the intention rather than the wording

A platform that needs a session, or one
[`social_networks`](https://github.com/valeratrades/social_networks) already knows, is read
by linking `social_networks_adapters`. It knows how to read a platform; the puller here
decides how a capture is filed.

Skool *conversations* are already `recon posts skool:<slug>`, which writes them into the
rolodex's `venues/` tree; nothing here duplicates that.

## Answering

```
.claude-plugin/marketplace.json   the one marketplace: Claude Code, claude.ai and Codex all read it
plugin/                           the procedure: what a member installs, the owner as well
├─ skills/service-arb/            how to answer, in the terms of the three tools; who to refer someone to
└─ .claude-plugin/plugin.json · .mcp.json   where the data is: the `playbook` server, the conductor's `/mcp`; Codex reads these too
playbook_web/                     the data: search · read · guide, operations the conductor's `/mcp` lists as tools (`sa:playbook:mcp:use`), every call logged
  └─ compiles knowledge/, bakes in its README.md, structured/ and references/**/*.md
skills/                           ingest · ops: upkeep, run from a checkout; .claude/skills links here
```

The skill holds the procedure; the server holds everything that changes with the corpus: the pages,
their list, the people, what each claim is worth today. A skill copy is a snapshot until its client
updates it, so nothing that moves with the corpus is written into it. The server does every
deterministic step (ranking, trust, course linkage); the skill keeps the judgement: the words to
search, anecdote against rule, the write-up. Tool descriptions say what a tool returns, never how to
answer.

The checkout has no `service-arb` skill of its own: the owner installs the plugin as a member does,
and tries unreleased skill text with `claude --plugin-dir plugin` or `playbook eval`.

Members use the knowledge on their own tokens and never get the repo. What the server does, and how,
is in `playbook_web/README.md`. The marketplace needs this repo public; were it to go private,
`plugin/` moves to a public repo of its own.

## LFS

LFS is billed for every object still in history, every month, so a file goes in only if nothing
else keeps it.

| goes in LFS | stays out |
|---|---|
| frames `playbook watch` read a line off — small, and ours alone | recordings on a host that keeps them |
| photos and documents we made or were sent, with no other home | anything a puller can fetch again |
| recordings on a host likely to lose them | |

A host's rating is how sure we are it still serves a recording a year on, 1 least:

| host | rating | what loses a recording |
|---|---|---|
| vocaroo | 1 | expires a recording left unplayed |
| drive | 2 | the owner unshares or deletes, with no notice |
| fathom | 2 | a share link is revoked per call, by whoever recorded it |
| loom | 3 | the owner deletes, or the plan's video cap pushes old ones out |
| vimeo | 4 | the owner deletes; links are otherwise stable |
| youtube | 5 | taken down or made private, rarely |

A recording goes in when its host rates ≤ 2, or ≤ 3 for course material (a recording a lesson in
`knowledge/references/skool_*/course/` links), committed with `git add -f` past the gitignore. Every clone downloads
every object, and every checkout, worktrees included, copies each out of `.git/lfs`, so the free
tier is a budget per clone, not a quota to fill.

Example, as of 2026-10-09 on the org's Free plan: 10 GiB-month of storage and 10 GiB of downloads
included, then $0.07/GiB-month and $0.0875/GiB; one object at most 2 GiB. ~23 GiB stored, nearly
all of it the recordings committed before this rule, is ~$0.90/month for as long as they stay in
history, and an hour of call is ~300–400 MB.

## Invariants

[docs/spec/](spec/): [trust](spec/trust.md), [knowledge](spec/knowledge.md).

<!--Reference: https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html-->
# Architecture

## Overview

A knowledge base about service arbitrage — running Google Business Profiles into
subcontracted trade work. Not code. The repository's whole job is that every claim
in it can be traced to the moment someone said it, and dated.

Sources are captured raw, then distilled by hand or by agent into topic notes.
Capture and distillation never happen in the same file.

```
   loom · fathom · youtube · chatgpt · skool · discord
                     │
                     │  playbook pull, social_networks
                     ▼
                   ref/                        raw · verbatim · dated
                     │
         ┌───────────┴────────────┐
         │ an agent reads it      │ the human's own notes, re-sourced
         ▼                        ▼
 structured/suggested/ ──────► structured/approved/
                  the human promotes │
                                     ▼
                                secondary/     plans, comparisons, decisions

        every bullet on the right links back to a second on the left
```

> very important to note that knowledge-base must be organized based on where and how it's likely to be used. [structured/approved/] drives the final agent's knowledge, so while [ref/] is organized based on where and when data is from, the actual knowledge we poses must follow the access patterns. Eg "expectations" from doing something are clearly subservient to the method/action itself, and data on how verify profiles, and on how to get reviews, are completely separate buckets.
> As such, the 

## Layers

**`ref/`** — raw, verbatim, never hand-edited. One file per source, stating its own
source id, where it came from and when it was recorded. A capture is identified by
that id and not by its path, so a file is free to be named and placed after what it
*says* — and a source that reorders or renames upstream is moved, not pulled twice.
Re-running a puller over an existing capture is a no-op; a stale one is re-pulled by
deleting it.

```
ref/
├── README.md                     # the source list — every source, and which of them a puller reaches
├── loom/<date>-<title>.md        # call recordings: summary, then the transcript under its chapters
├── fathom/<date>-<title>.md      # the same, for calls recorded on fathom
├── drive/<date>-<title>-<id>.md  # the same, for recordings in a shared drive folder
├── vimeo/ · vocaroo/             # the same, for hosts the registry line names and dates
├── <platform>/<capture>/         # beside a call capture: what its recording shows
│   ├── recording.<ext>           # as the host serves it, 720p where there is a choice — gitignored, bar what LFS keeps
│   ├── shown.md                  # what is on screen that the speech does not say, by `playbook watch`
│   └── frames/<secs>.jpg         # the frame each line of `shown.md` was read off — LFS
├── docs/<title>.md               # google docs, exported as markdown
├── research/<id>.md              # chatgpt shares, one `## user` / `## assistant` section per turn
├── youtube/                      # channel videos
│   ├── README.md                 # every video each channel lists, dated, ticked once captured — `playbook pull yt sync`
│   └── <who>/<id>.md             # a video: summary, chapters, what is shown, captions — and `<id>/<secs>.jpg` the frames its `## shown` lines were read off
└── skool_<group>/                # a classroom, via `social_networks` — `gmbpp`, and the `cheap` group
    ├── README.md                 # what upstream holds, and the day it was last compared to this
    └── course/<NN-module>/       # a directory per module, a file per lesson, both in upstream's order
```

Two things a capture cannot state about itself are written beside it instead, since a
capture is never edited: **how far behind upstream it is**, in `skool_<group>/README.md`,
rewritten every run with the day of the run; and **what a source says that this tree
has no capture of**, printed by the puller for `ref/README.md` to be told about by hand.

### Call captures

```md
# <title>

- source: <https://www.loom.com/share/<id>>        # or fathom.video/share/<id>, drive.google.com/file/d/<id>
- recorded: · duration: · transcribed by: · read by: · pulled by:

## summary

<the platform's, verbatim>

## transcript

### [02:39](https://www.loom.com/share/<id>?t=159) Location Spoofing Strategy

plain prose, in paragraphs, no stamps in it
```

- the transcript is the platform's; where it has none, the puller transcribes the audio itself, and
  `transcribed by:` says so
- the platform's summary and chapters are dropped when its chapters stop more than ten minutes short
  of the last words said
- chapters are `###` headers inside `## transcript`, never a list of their own. The header is the
  only timestamp in the capture. A citation links the second a claim was said
- nothing sits above the first header
- paragraphs and headers are a blank line apart
- a digest's prose is the prose of the committed pull it ran on, so a pull is committed before it is
  digested
- a digested summary has a `### <topic> <stamp>` per chapter, same names and times; a platform's own
  summary is verbatim and held to nothing
- speaker turns, where the platform names speakers, are paragraphs opening `**<speaker>**:`
- a recording nobody chaptered gets an untitled header per paragraph instead — the times
  `/ingest call-digest` needs to place chapters. It titles the ones that open a chapter and deletes the rest
- a recording nobody speaks in is captured all the same: `transcribed by:` ends `— nobody speaks`,
  and only then is `## transcript` empty
- a recording is read through `playbook recording <capture.md>`, which hands back the one
  beside the capture, fetching it from its host first when there is none

`playbook pull call` checks every capture on disk against this on each run, and `playbook check calls` runs only
that; a hook runs it after any agent edit under a call capture's platform dir.

Machine transcriptions of a recording belong in `ref/` too, each naming what produced it: whisper's
of its audio, in `transcribed by:`, and `playbook watch`'s of its picture, in `shown.md`,
whose header names the model, the day and what it cost. A youtube capture reads its picture the same
way, into its own `## shown`, and states the models and the cost in its header. Every line of `shown.md` links the second it
describes and embeds the frame at that second, so the claim can be checked by eye.

The one reading an agent may put in `ref/` is what the source's own platform would have:
a recording its platform never summarised gets a summary from `/ingest call-digest`, which says so
in the capture's `read by:` line. Nothing else.

**`structured/suggested/`** — an agent's own reading of a source. Unreviewed.
**`structured/approved/`** — what the human has already vetted.
**`structured/internal/`** — what we know about our own team and its operations. Read in a checkout,
never baked into the member image.

Promotion is the human moving a bullet from one to the other. An agent may place into
`approved/` only material the human already wrote — their own call notes, transcribed
and re-sourced — never its own inference.

**`secondary/`** — conclusions that follow from the notes rather than from a source:
plans, comparisons, decisions, how a contradiction was resolved.

## Blocks

`structured/` is a tree of topical blocks, the same shape on both sides. As with rust modules, `x.md`
is the block and `x/` holds its children, and either one can exist alone:

```
structured/{approved,suggested}/
├── market.md       market/niches/<niche>/…     niche and city choice; per-trade economics
├── profile/        creation · verification/{video,live} · suspension/registration
├── ranking/        naming · categories · site · reviews/{stand,rate_limits,deletion,sourcing,local_guide}
├── conversion.md   site, intake, VA call handling
├── fulfilment.md   contractors, splits, pricing
└── infra.md        gmails, devices, proxies, phones, payments
```

A block splits into `## info` (what is so), `## facts` (what someone ran, and its result) and
`## plays` (an inefficiency and how to act on it). The taxonomy follows the data. A block that keeps
collecting unrelated bullets splits into children.

## Note format

A bullet is one line. Elaboration goes underneath it. Every claim carries its source directly
beneath it, and the link text is the base reliability and the date:

```md
- one review a day per profile, at most
  [r2 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=3026)
```

The rest of the format and the style are in `structured/README.md`. `playbook check cites` enforces
them, in `nix run .#check`, and a hook runs it after any agent edit under `structured/`.

## Reliability

Every citation carries a reliability: 0 to 10, lower is better. Who said it and how they came to know
it sets the base. A person on the [people](#people) list takes their own base instead, whatever their role:

| base | source                                                        |
|------|---------------------------------------------------------------|
| 0    | tested myself                                                 |
| 1    | skool course material or in [structured/approved/]            |
| 3    | a gmbpp member tested it                                      |
| 4    | a gmbpp member claimed it or in [structured/suggested]        |
| 5    | a gmbpp member heard or thought of it; a cheap group member tested it |
| 6    | a cheap group member claimed it                               |
| 7    | a cheap group member heard or thought of it                   |

<a id="people"></a>**people**:

| base | who                                      |
|------|------------------------------------------|
| 2    | Eric                                     |
| 2    | Zach Peterson (youtube `@zachpeters0n`)  |
| 3    | Gabe Jacobs (skool `gabe-jacobs-9502`)   |
| 3    | Marcus Milan (youtube `@marcusmilxn`)    |

Then everything decays with age, up to 10 — a cheap group member having heard of it long ago, which
is as good as nothing. How the decay is counted lives in `playbook/src/cite_check.rs`, not here.

A claim backed by several sources cites its best one first; the rest sit under it as secondary.
Between two sources of equal base, this is newest-wins. Sources may contradict each other; `structured/`
may not. A contradiction is resolved into one claim, weighed by more than the number, and the record of
it goes to `secondary/resolved/`.

## Pullers

Every tool here is a subcommand of the `playbook` crate at the root of the cargo workspace, so each
dependency is pinned once for all of them. `playbook <cmd>` is `cargo r -p playbook -- <cmd>`, run
from anywhere inside the checkout. Each `pull <platform>` takes source URLs as arguments, or with
none reads `ref/README.md` for links of its platform and fetches the ones missing from disk. So
adding a source is pasting its link into `ref/README.md` and re-running. `pull yt` alone goes
through an index: `sync` lists every video of the channels `ref/README.md` links into
`ref/youtube/README.md`, and `transcribe` captures what that list has unticked, or only the listed
videos it is given.

`ref/README.md` is the only registry. A capture quotes its own source URL and a
transcript quotes every link that was said out loud, so a puller that scanned the tree
would feed on its own output. A puller that *finds* a link — one lesson pointing at
another platform's recording — prints it rather than following it, and the registry
grows by hand. The one exception is the group's calls feed: what it posts is the list's own
material, so `pull skool` writes each recording into the registry itself.

```
playbook/src/
├── call.rs            # pull call: loom / fathom share page, drive folder → summary, chapters, and the platform's transcript, or whisper's where there is none; `recording`: the one way to a recording, kept or fetched; `check calls`
├── watch.rs           # watch, not a puller: a recording, through `ask_llm`'s `Client::watch`, frames only where the speech says something is shown (`--legacy`: wherever the picture changes) → `shown.md`
├── doc.rs             # pull doc: google doc → its markdown export
├── chatgpt.rs         # pull chatgpt: headless chromium → the DOM's turns
├── yt.rs              # pull yt sync: channels → the dated list of their videos; pull yt transcribe: captions in citable blocks, summary and chapters through `ask_llm`, the video through `Client::watch`, frames only where the captions say something is shown (`--legacy`: wherever the picture changes) → `## shown`, description
├── skool.rs           # pull skool: skool's classroom → the course tree, module by module and lesson by lesson; the calls feed → the registry's group calls
├── cite_check.rs      # check cites, not a puller: holds every citation in structured/ to its capture in ref/
├── photo_metadata.rs  # photo-metadata: exif of our own photos, and synthetic sidecars
└── eval.rs            # eval
```

`playbook re-transcribe <audio|picture> <path>...`
lists every call capture under the paths with the estimated cost of redoing it; `picture` is
estimated at the `--legacy` rate either way, as the default's cost is known only after the pick. It runs only with
`--execute`, and asks first when the total is past $1. Audio is redone only for a whisper transcript
that has no digest yet, since a re-pull drops the digest. `playbook eval` answers `playbook/eval.json`
through a member's client, on this tree and on the last release, and judges each answer against the
case's expectations.

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
├─ skills/service-arb/SKILL.md    how to answer, in the terms of the three tools
└─ .mcp.json · mcp.json           where the data is: the `playbook` server, for Claude and for Codex
guides/<area>.md                  how to decide, per area: data, served by `guide`
playbook_web/                     the data: search · read · guide over MCP, behind the Service-Arb panel (`sa:playbook:mcp:use`), every call logged
  └─ bakes in guides/, structured/{approved,suggested}/ and ref/**/*.md
skills/                           ingest · distill · ops: upkeep, run from a checkout; .claude/skills links here
```

The skill holds the procedure; the server holds everything that changes with the corpus: the guides,
their list, the reliability tables, the scopes. A skill copy is a snapshot until its client updates
it, so nothing that moves with the corpus is written into it. The server does every deterministic
step (ranking, reliability arithmetic, course linkage); the skill keeps the judgement: the words to
search, who speaks in a call, anecdote against rule, the write-up. Tool descriptions say what a tool
returns, never how to answer.

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
`ref/skool_*/course/` links), committed with `git add -f` past the gitignore. Every clone downloads
every object, and every checkout, worktrees included, copies each out of `.git/lfs`, so the free
tier is a budget per clone, not a quota to fill.

Example, as of 2026-10-09 on the org's Free plan: 10 GiB-month of storage and 10 GiB of downloads
included, then $0.07/GiB-month and $0.0875/GiB; one object at most 2 GiB. ~23 GiB stored, nearly
all of it the recordings committed before this rule, is ~$0.90/month for as long as they stay in
history, and an hour of call is ~300–400 MB.

## Invariants

- Nothing enters `structured/` that did not enter `ref/` first — bar `r0`, which enters as the
  comment that closed its test's issue, and the owner's own words, which enter `// unsourced: Valera`
  and are re-sourced once a capture holds them.
- Every claim carries a link to its source and the date it was said.
- `ref/` is written by pullers, never by hand — bar what `/ingest call-digest` fills in, which says so.
- An agent's own reading goes to `suggested/`, never straight to `approved/`.
- A puller writes a new capture, moves one, or does nothing; it never edits what one says.
- The member image carries `guides/`, `structured/approved/`, `structured/suggested/` and `ref/**/*.md` only: no scripts, no docs, no
  history, no media.

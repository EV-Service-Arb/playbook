---
name: call-digest
description: Write the digest of a recording in ref/ — the summary and chapters its platform never wrote, and the `## claims` every recording gets. Use when call-pull.rs says a recording was left unread, when a capture's `read by:` says `nothing yet`, when it has no `claims by:`, or when asked to summarise, chapter or pull the claims out of a recording in this repo.
---

# call-digest

`scripts/call-pull.rs` writes what the platform's own AI made of a recording, and always a full
transcript. The digest is ours: a summary and chapters where the platform wrote none, and on every
recording, its `## claims`. The capture's shape is "Call captures" in `docs/ARCHITECTURE.md`.

## Pick the target

With a path argument, that file. Without one, the first capture with no claims yet:

```bash
grep -L '^- claims by: ' ref/{loom,fathom,drive,vimeo,vocaroo}/*.md ref/youtube/*/*.md | xargs grep -L -- '— nobody speaks'
```

Do one file per run. These are long; a run that batches them writes worse digests.

A video (`ref/youtube/`) is a puller's capture whose claims predate the puller writing them; it gets
`## claims` only, never a summary or chapters.

## `## summary` and chapters

Only when `read by:` says `nothing yet`:

```md
- read by: nothing yet — loom wrote no summary and no chapters, and `/call-digest` writes them here
```

Write what is missing, in loom's shape, so the only thing that tells a reader which recordings you
read is that one line.

### `## summary`

Goes between the header and `## transcript`. A paragraph, then a `### <name> <stamp>` section per
topic with bullets under it — `ref/loom/2026-05-30-gmb-passive-profits-group-call.md` is the shape. The stamp
drops the leading zero: `0:00`, `3:34`, `1:02:27`.

A one-recorder video gets a first-person paragraph (`In this video, I walk you through…`); a call
gets it in the third person, named (`Eric explains…`, `Aidan reports…`). Say what was decided and
what is meant to happen next. No preamble, no "this video covers".

### Chapters

An unchaptered capture has an untitled header over every paragraph:

```md
### [03:34](https://www.loom.com/share/<id>?t=214)
```

Choose the ones where a topic starts and give them a title:

```md
### [03:34](https://www.loom.com/share/<id>?t=214) Spoofing location on an android
``` Delete every other untitled header. The
paragraphs underneath stay exactly as they are. A chapter is a topic, so a
two-hour call gets tens of them, not five. The first header stays, so nothing sits above it.

The summary's `###` topics and the chapters are the same list, with the same names and times.

Then rewrite the header line:

```md
- read by: `/call-digest`, <today's date>
```

If you did not write all of it, say what you wrote: ``- read by: `/call-digest`, 2026-09-18 — chapters only``.

## `## claims`

Every recording. Directly above `## transcript`; in a video, directly below `## shown`.

```md
## claims

### Service call fees: what to charge 2:38
- Art Khan, tested: charges $39 per inspection
  [r3 2026-03-31](https://fathom.video/share/<id>?t=170)
- Eric, claims: a plumber complaining about a $60–100 fee has too much work already
  [r2 2026-03-31](https://fathom.video/share/<id>?t=212)
```

- `### <topic> <stamp>`: a chapter's name and time, exactly, chapters in order. A chapter nobody
  claims anything in is left out
- `- <who>, <mode>: <claim>`: one claim per bullet, terse and lowercase past the name, in the style of
  `structured/README.md`. Who is the name in full, as the recording gives it
  - `tested`: they ran it themselves and report what happened
  - `claims`: they state it as so, without having run it in front of us
  - `heard`: someone told them, or they are guessing out loud
- under it, its one citation, in `structured/`'s format: `[r<base> <date>](<this capture's source>?t=<second>)`
  - the date is the capture's own, `recorded:` or `uploaded:`
  - the second is where the claim is said, read off the chapter header above it and the words. Not
    the chapter's second
  - the base is `docs/ARCHITECTURE.md`'s `## Reliability`, from the speaker's group and the mode. A
    person on its people list takes their own base. A recording a skool lesson links is course
    material: `r1`. Otherwise the group is the registry's: a capture `ref/README.md` lists under
    the gmbpp group's calls or Eric's is a gmbpp member's unless the speaker says otherwise, and
    `skill/service-arb/sections/experts.md` names who is who
  - a speaker in no rated group gets no claims
- what the recording says, at its second. Not what follows from it, not what you know besides. A
  question asked is not a claim; its answer is, by whoever answers
- a claim repeated later in the call is written once, at its clearest second

Then add the header line under `read by:`:

```md
- claims by: `/call-digest`, <today's date>
```

When the recording claims nothing, the line ends `— none` and there is no `## claims`.

## Then check

```bash
scripts/call-pull.rs --check && scripts/cite-check.rs
```

A hook runs them after every edit here as well.

## What not to do

- do not change a word of the transcript. Moving headers is all you do there. `ref/` is verbatim
- do not touch `# title`, `source:`, `recorded:`, `uploaded:`, `duration:`, `transcribed by:` or `pulled by:`
- do not edit a platform's summary. It is verbatim
- do not write a claim the recording does not make. A reading of a source that the source does not
  support poisons everything downstream of it
- do not promote anything into `structured/`. That is a separate step, and a human's
- a re-pull (deleting the file and running `call-pull.rs`) throws your work away. That is the right
  trade: the platform having caught up is worth more than what you wrote

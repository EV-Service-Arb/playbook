# call-digest

Write the summary and chapters a capture's platform never wrote, in loom's shape, so that its
`read by:` line is the only thing telling a reader who read it. The shape is
[Call captures](../../docs/ARCHITECTURE.md#call-captures); before reading the recording itself, [recordings.md](recordings.md).

## Target

The path given, else one of `grep -l 'read by: nothing yet' ref/*/*.md`. One capture per run: batched,
the summaries get worse.

## Summary

A paragraph, then a `### <topic> <stamp>` per chapter with bullets under it, as in
`ref/loom/2026-05-30-gmb-passive-profits-group-call.md`. Stamps drop the leading zero: `3:34`, `1:02:27`.

One recorder is first person (`In this video, I walk you through…`); a call is third person, named
(`Eric explains…`, `Aidan reports…`). Say what was decided and what happens next.

## Chapters

Title the untitled headers where a topic starts and delete the rest. A chapter is a topic, so a
two-hour call gets tens of them, not five.

Then the header line: ``- read by: `/ingest call-digest`, <today>``, or with what you wrote, if not all
of it: ``- read by: `/ingest call-digest`, 2026-09-18 — chapters only``.

## Mistakes

- writing a claim the recording does not make
- touching any header line but `read by:`
- putting anything into `structured/`: that is [distill](../distill/SKILL.md)
- keeping a capture from a re-pull to save its digest. Once the platform has caught up, its reading wins

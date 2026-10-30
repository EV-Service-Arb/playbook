---
name: distill
description: Turn what a capture in ref/ says into claims in structured/suggested/. Use on "distill this", on a capture nobody has distilled yet, and after answering a question in a checkout where the answer leaned on ref/ because structured/ had nothing.
---

# distill

Claims go to `structured/suggested/`, in [the block format](../../structured/README.md).

- a claim goes to the block the next asker would open, per the note in
  [Overview](../../docs/ARCHITECTURE.md#overview), not where the source happened to say it
- a claim already present joins its citation run. It never becomes a second bullet
- a contradiction is [resolved](../ingest/resolve.md). Nothing is deleted
- a new block goes into [Blocks](../../docs/ARCHITECTURE.md#blocks)' tree
- loom names no speakers. Who said it comes from what is said: names, roles, who answers whom
- a digested summary is our reading, not the source. Cite the transcript
- a second inside a chapter is the untitled header above the claim in the capture's last commit before
  its digest; a chapter the platform wrote has no finer second than its own
- `## shown` and `shown.md` are a model's reading: [recordings.md](../ingest/recordings.md)
- a source no row of either reliability table covers is asked of the owner, never guessed.
  `ref/research/` sits at r5 with no row

## From an answer

An answer `structured/` could not give and `ref/` did leaves its clearest claim behind, so the next
question finds it there. Only what the answer leaned on, best number first: no anecdote, no guess,
nothing a better source contradicts.

> members can't write back yet; their corrections reach the knowledge base only through a checkout. general-case write-back, as PRs from members' agents: [#33](https://github.com/EV-Service-Arb/playbook/issues/33)
<!--DEPRECATE: remove once #33 lands-->

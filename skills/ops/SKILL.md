---
name: ops
description: Act on our own profiles and books from a checkout — build a verification paper pack for an address, fill Google's live-verification form, generate photo metadata for a city, or check what was spent and is owed. Use when asked to do any of these, rather than to answer how.
---

# ops

The tools sit beside the checkout. Use one only if it exists.

- `../verif_tools`: `prepare-verif` builds the paper pack for an address, `submit-live` fills Google's
  live-verification form in your Chrome
- `../accounting`: its [accountant](../../../accounting/.claude/skills/accountant/SKILL.md) skill, for
  what was spent and is owed
- `scripts/photo-metadata.rs <dir> --city <city>`: metadata for photos

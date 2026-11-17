# propose

Read one `content.md` against the manual and write what it should change:
`knowledge/log/digest/<capture's date>-<capture's name>.md`. Nothing in `knowledge/structured/` is
touched in this step; this is the plan [integrate](integrate.md) carries out, and the record of why.

For each point of the digest worth keeping, one entry:

```md
## <page it belongs on, as a path under knowledge/structured/>

- <the point, with its citation as the manual will carry it>
  does: <confirms | qualifies | contradicts | adds a step | is new evidence for> "<the bullet it meets, quoted>"
```

- a point goes to the page the next asker would open, not to where the call happened to raise it. The
  result someone got from a method sits under that method
- a point no page fits: say which new page, and what else already in the manual would move there
- a point that contradicts the manual is marked, never softened: [resolve.md](resolve.md) settles it
- what is left out and why, in one closing list: `## not kept`. A story no conclusion stands on stays
  in the digest

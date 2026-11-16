Ask about service arbitrage as usual; with `service-arb` installed, its skill answers from the playbook, citing the dated source of each claim. To hold a conversation to the playbook, say so ("answer from service-arb"); in Claude Code, `/service-arb:service-arb` does the same.

The skill's instruments, the `playbook` server's tools:

| tool | returns |
|---|---|
| `search(pattern, scope?, screen?)` | matching passages: the manual first, then the course, then calls, videos and texts, each kind in its own slots, each labelled with who backs it and what that is worth today |
| `read(path, header, line?)` | one chapter of a capture or one section of a manual page |
| `guide(area)` | one page of the manual (`reviews`, `profile/verification`, …), or `people` |

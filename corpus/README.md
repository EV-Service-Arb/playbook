The compiler of `knowledge/`: holds it to [knowledge/README.md](../knowledge/README.md) and resolves every
citation to who said it, how sure, when the information is from, and the half-life of its module.

- `compile` is the only way in. `playbook check` prints what it refuses; `playbook_web/build.rs` bakes
  what it resolves, so nothing is released that fails it
- it resolves no further than `init`. `Backing::norm` and `Backing::label` are called by whoever knows what
  day it is; `label` is the one form every reader gets a citation in
- the trust tables and the default half-life are read out of `knowledge/README.md`; nothing here restates them

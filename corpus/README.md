The compiler of `knowledge/`: holds it to [knowledge/README.md](../knowledge/README.md) and resolves every
citation to who said it, what that is worth and when the information is from.

- `compile` is the only way in. `playbook check` prints what it refuses; `playbook_web/build.rs` bakes
  what it resolves, so nothing is released that fails it
- it resolves no further than `source(who) × grade(how)`. `gamma` is applied by whoever knows what
  day it is
- the trust tables are read out of `knowledge/README.md`; nothing here restates them

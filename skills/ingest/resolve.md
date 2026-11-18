# resolve

Sources may disagree; `knowledge/structured/` may not. Run when it contradicts itself, a source contradicts it, or one method may beat another.

A contradiction is two claims that cannot both be acted on by the same operator in the same situation.
Before calling one, read who each was said to and what they were running: advice to a beginner and
advice to a member already in a trade, a rule for new profiles and a result on ten-year-old ones, one
person's pass and another's rejection, are each scoped by their case, and the scope goes into the bullet.
Two numbers for one quantity are not one either: both stay as evidence, per `knowledge/README.md`.
Methods with different trade-offs can coexist. We only evict those that are strictly inferior to other known methods.
// evict as in we still list those methods in `### other`, and point to what they are beat by and why

- gather every mention first, across `knowledge/structured/` and `knowledge/references/`
- then weigh, through the view: norm first, with init and age beside it where the topic goes stale faster or
  slower than its module; what each speaker meant, what they were doing when it worked or failed. The rest of what we know on the topic counts, and so does common industry
  knowledge where the practice overlaps
- the conclusion is our opinion. Where the evidence is not conclusive, say what decides between the
  sides rather than pick one
- the record, `knowledge/log/resolved/<date>-<slug>.md`: every source weighed, linked, and a short why for
  the conclusion
- the conflicting bullets become one, with a `resolved:` line linking the record
- the conflict and its conclusion, or where the weighing got stuck, go to the owner either way. They
  may know what no source says
- a revision, one speaker against their own earlier word: the later statement wins unless they say otherwise.
  Find every citation of that `who` on the topic, `grep -rn '\[<who> ' <view>/structured`, and rewrite each bullet
  standing on the earlier view, one record for the lot
- when a bullet without `(?)` is involved, the record goes to `tmp/resolved/`, and it is applied only once the
  owner confirms

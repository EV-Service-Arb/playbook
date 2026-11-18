# Trust

How much a citation is worth when it is read. The tables it draws on are in
[knowledge/README.md](../../knowledge/README.md#trust).

r[trust.init]
What a citation was worth when said, `init`, MUST be `source(who) × P(certainty)`: the speaker's base value, times the
two-sided normal coverage of the statement's certainty in σ.

r[trust.norm]
What it is worth today, `norm`, MUST be `init × γ(age; half-life)`, in `[0, 1]`, higher is better. Advice is ranked by norm.

r[trust.certainty]
Certainty MUST be read off the statement — what the speaker bases it on, how they qualify it — never off who said it.

r[trust.decay]
`γ` MUST be a power law of age, `(1 + age/τ)^-s`: it never reaches zero, and it loses less with each year.

r[trust.age]
Age MUST count from when the information was observed: the capture's date, or earlier where the speaker dates it. Never later.

r[trust.derived+2]
Nothing computed from the trust tables or the clock may be written into `knowledge/`, a skill or a tool description. Only what it is computed from is written: who, how they know, how sure, when. A view rendered from it is written outside the checkout.

r[trust.arithmetic+2]
No reader, human or model, may be asked to compute trust. Every reader, members and checkout agents alike, gets init, age and norm on every citation, or is told it has none.

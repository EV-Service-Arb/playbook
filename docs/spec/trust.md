# Trust

How much a claim is worth when it is read. The tables it draws on are in
[knowledge/README.md](../../knowledge/README.md#trust).

r[trust.effective]
Effective trust MUST be `source(who) × grade(how) × γ(age)`, in `[0, 1]`, higher is better.

r[trust.decay]
`γ` MUST be a power law of age, `(1 + age/τ)^-s`: it never reaches zero, and it loses less with each year.

r[trust.age]
Age MUST count from when the information was observed: the capture's date, or earlier where the speaker dates it. Never later.

r[trust.derived]
Nothing computed from the trust tables or the clock may be written into `knowledge/`, a skill or a tool description. Only what it is computed from is written: who, how they know, when.

r[trust.arithmetic]
No reader, human or model, may be asked to compute trust. A hit carries its effective number, or says it has none.

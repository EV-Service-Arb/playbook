#set page(width: 210mm, height: auto, margin: 14mm)
#set text(size: 9.5pt)
#show raw.where(block: true): it => block(fill: luma(248), inset: 8pt, radius: 3pt, width: 100%, text(size: 7.6pt, it))
#import "@preview/lilaq:0.4.0" as lq

= Trust arithmetic

What a citation is worth, computed off what it states. The tables, and the defaults plotted here, are read
off #link("../knowledge/README.md")[knowledge/README.md]; the rules are #link("../docs/spec/trust.md")[docs/spec/trust.md].

```
<!--2026-09-25 Dante ran 2σ <url>-->     written: who · how · σ · when
          │                │   │
 source(Dante)  ×  P(|z| < 2σ)              init: what it was worth when said
          │
          ×  γ(age; half-life, step)        norm: what it is worth today; advice is ranked by it

 γ = 1 / (1 + log_b(1 + (b−1)·age/h))       h: the module's half-life · b: step > 1
   1 at 0 · ½ at h · ⅓ at (b+1)h · ¼ at (b²+b+1)h · … never 0
   b → 1 is 1/(1 + age/h): every half-life costs the same
```

#let readme = read(".knowledge/README.md")
#let default(row) = readme.match(regex("(?m)^\\| " + row + " \\| (\\S+) \\|$")).captures.first()
#let days(tf) = {
  let n = float(tf.slice(0, -1))
  n * (d: 1, w: 7, M: 30, y: 365).at(tf.last())
}
#let gamma(age, h, b) = if b == 1 { 1 / (1 + age / h) } else { 1 / (1 + calc.log(1 + (b - 1) * age / h, base: b)) }

#let h0 = default("half-life")
#let b0 = float(default("step"))
#let sets = (
  (h0, b0, "default"),
  (h0, 1, ""),
  (h0, 16, ""),
  ("5y", b0, ""),
  ("3M", b0, ""),
)
#let curves(xs, unit: 1) = sets.map(((h, b, note)) => lq.plot(
  xs.map(a => a / unit),
  xs.map(a => gamma(a, days(h), b)),
  mark: none,
  label: [h #h, b #b #if note != "" [(#note)]],
))

== γ over age

#let years = lq.linspace(0, 3650, num: 400)
#lq.diagram(
  width: 16cm,
  height: 7cm,
  ylim: (0, 1),
  xlim: (0, 10),
  xlabel: [age, years],
  ylabel: [γ],
  ..curves(years, unit: 365),
)

== The same, age on a log scale

`h` places the first notch; past it, each next one sits about `b` times further out.

#let logs = range(0, 401).map(i => calc.pow(10, i / 100))
#lq.diagram(
  width: 16cm,
  height: 7cm,
  ylim: (0, 1),
  xscale: "log",
  xlabel: [age, days],
  ylabel: [γ],
  ..curves(logs),
)

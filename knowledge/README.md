# knowledge

```
references/<platform>/<capture>.md     raw: what a source said, as pulled. never edited
references/<platform>/<capture>/       readings of it: shown.md (the picture), content.md (the whole)
log/digest/<date>-<capture>.md         what a digest should change in the manual, before it does
log/resolved/<date>-<slug>.md          how a contradiction was settled
structured/                            the manual: what we do, and the evidence under it
internal/                              about our own team. never shipped
```

A source moves down this list one step at a time, each step reading only what the one before wrote:
`pull` → `watch` → digest → propose → integrate. The steps are in `skills/ingest/`; what holds them is
[docs/spec/](../docs/spec/). `playbook check` holds every format below, and a hook runs it on any edit here.

## Citation

One line, an html comment, under the text it backs:

```md
<!--2026-09-25 Dante ran https://www.loom.com/share/8e7db736690348a3a708702e99ee92ab?t=353-->
```

- the date the information is from: the capture's, or earlier where the speaker dates it
- who: a row of [people](#people). Left out where the source fixes it: a course lesson, a channel's video,
  a chatgpt share. `agent`: one of ours inferred it, from nothing it could cite
- how they know: `ran` it and report the result, `says` so, `heard` it from someone
- the url of the capture, at the second it was said. No url: it was never recorded
- `us ran` cites the comment that closed the test's issue

In a `content.md` the capture is given, so a point's citation is `<!--Dante ran ?t=353-->`, with the
date only where it is earlier than the capture's.

## Trust

`source(who) × grade(how) × γ(age)`, per [docs/spec/trust.md](../docs/spec/trust.md). Only the server computes it.

| grade | how |
|---|---|
| 1 | ran |
| 0.8 | says |
| 0.6 | heard |

<a id="people"></a>A person's trust is a number, or the row they stand in for:

| who | trust | reach | proven at | available |
|---|---|---|---|---|
| us | 1 | | | |
| gmbpp member | 0.6 | | | |
| cheap member | 0.4 | | | |
| chatgpt | 0.45 | | | |
| agent | 0.25 | | | |
| Valera | 0.8 | | | |
| Eric | 0.8 | youtube `eric` | | |
| Zach Peterson | 0.8 | youtube `zach` | | |
| Gabe Jacobs | 0.7 | skool `gabe-jacobs-9502` | android spoofing (root, Magisk), injecting real footage into video verification, AI vs human receptionist | 6 // inferred, offered to help others with verification |
| Marcus Milan | 0.7 | youtube `marcus` | | |
| Matt Gazda | 0.8 | youtube `matt` | | |
| Ilya | gmbpp member | discord `ilyac` | house cleaning, run for years remotely: contractor splits, insurance, reselling off-niche leads. reviews from the philippines onto aged profiles, review gating, DBA names | 5 // inferred, shared at length on the 2026-09-18 call |
| Ryan D | gmbpp member | Eric's group calls, handle unknown | review tables: 200+ reviews in a week, campus entrances, QR close | 5 // inferred, shared the whole method on a call |
| Henry A | gmbpp member | discord `growthghost`, skool `henry-a-1639` | review screenshot capture for reinstatement appeals (Playwright) | 6 // inferred, offered his code, pointed us to Devin |
| Grisha G | gmbpp member | skool `grisha-g-8980` | review tables at flea markets, ~$15/day | 3 |
| Vladimir Bjelica | gmbpp member | skool `vladimir-bjelica-9798` | plumbing on subcontractors: hourly plumbers, a 50–55% cut, large bids | 3 |
| Artheme Chaykin | gmbpp member | discord `artaxiaszeno`, skool `artheme-chaykin-5928` | french pay-per-call plumbing, a $4/h philippine VA dispatching | 3 |
| Juan Palacios | gmbpp member | skool `juan-palacios-3841` | live video verification for auto detailing: exterior, signage, vehicle | 3 |
| Sam Hart | gmbpp member | skool `sam-hart-1875` | live video verification: four passed in two days from a home office, a framed logo taped to the door, no spoofing | 4 // inferred, offered to post a video of his method in the community |
| Dante | gmbpp member | | | |
| John Osayi | gmbpp member | | | |
| James McBride | gmbpp member | | | |
| Kit Volcano | gmbpp member | | | |
| Glenn | gmbpp member | | | |
| Jordan Rodriguez | gmbpp member | | | |
| Lamarious Thomas | gmbpp member | | | |
| Devin Grandt | gmbpp member | | | |
| Raphael D'Angelo | gmbpp member | | | |
| David | gmbpp member | | | |
| Luke Silver | gmbpp member | | | |
| Derick Parsons | gmbpp member | | | |
| Alex Zubarev | gmbpp member | | | |
| Muhammad | gmbpp member | | | |
| William | gmbpp member | | | |
| Elliot | gmbpp member | | | |
| Donovan | gmbpp member | | | |
| Bradley | gmbpp member | | | |
| Abdullah | gmbpp member | | | |
| PP | gmbpp member | | | |
| Erwin | gmbpp member | | | |
| jd | gmbpp member | | | |
| Kareem | gmbpp member | | | |
| Connor | gmbpp member | | | |
| Aiden | gmbpp member | | | |
| Tony | gmbpp member | | | |
| Anmar | gmbpp member | | | |
| Luca | gmbpp member | | | |
| Karim | gmbpp member | | | |
| Cole | 0.7 | | landscaping on subcontractors, design-and-build: marks the sub's quote up 15–25%, takes half upfront. "hundreds of thousands a month", as a member relays it | |
| Greg | gmbpp member | | | |
| Walker | gmbpp member | | | |
| Chris | gmbpp member | | | |
| TruSpark Electric | gmbpp member | | | |
| Art Khan | gmbpp member | | | |
| Randall | gmbpp member | | | |
| Anthony | gmbpp member | | | |
| Cheyenne | gmbpp member | | | |
| Joseph | gmbpp member | | | |
| Raymond | gmbpp member | | | |
| Paul | gmbpp member | | | |
| Finn | gmbpp member | | | |
| Larry | gmbpp member | | | |
| JS | gmbpp member | | | |
| Prabh | gmbpp member | | | |
| Melo | gmbpp member | | | |
| Mendel | gmbpp member | | | |
| Phil | gmbpp member | | | |
| Brandon | gmbpp member | | | |
| Hayden | gmbpp member | | | |
| Sumit | gmbpp member | | | |
| Joey | gmbpp member | | | |
| Keith | gmbpp member | | | |
| Noah | gmbpp member | | | |
| Julian | gmbpp member | | | |
| Kai | gmbpp member | | | |
| Zach | gmbpp member | | | |
| Nathaniel | gmbpp member | | | |
| Raf | gmbpp member | | | |
| Austin | gmbpp member | | | |
| Graham | gmbpp member | | | |
| Bobby | gmbpp member | | | |
| Alan | gmbpp member | | | |
| Shaheem | gmbpp member | | | |

`available` is 1–10, how often they have helped or shared; 3 is never asked. youtube `<x>` is the
channel captured under `references/youtube/<x>/`. A speaker nobody could
name is cited as their group's row.

## Digest: `content.md`

```md
# <the capture's title>

- of: [<capture>.md](../<capture>.md)
- digested by: <what>, <date>

## gist

who was on and what scale each operates at, then what was settled. a paragraph

## chapters

### [12:34](<url>?t=754) <topic>

- <a point, in the speaker's terms. numbers, names, handles as said>
  <!--Dante ran ?t=771-->
- Q: <asked> / A: <answered>
  <!--Eric says ?t=802-->

## open

- <a question nobody answered; something someone said they would do>
```

- every stamp is one the capture has

> a digest can name the wrong speaker: loom names nobody. Whoever notices one decides which it is: a
> one-off is corrected in place; a digest wrong throughout is weak as a whole and is digested again by a
> stronger model

- a chapter is a topic: a two-hour call has tens of them
- what a point would change in how we operate decides whether it is written. So does who says it: the
  larger the operation they run, the more of what they say is kept

## Manual: `structured/`

Pages are filed under the thing being operated, in the order the business meets it. `x.md` is the
page and `x/` holds its children. A claim scoped to one trade lives under `market/niches/<trade>/`, at
the path it would have outside it.

```md
# <topic>

<what we do here, as of now. the page's answer, in a few lines>

## <a step or a decision, in the order it is met>

- <a conclusion: reads true without knowing who said it>
  > what qualifies it
  - <evidence: who, what they ran or said, the number>
    <!--2026-09-25 Dante ran https://…?t=353-->

## open

- Q: <not answered yet>
- TODO: <something to go test>
```

- every bullet has a citation somewhere beneath it, bar those under `## open`
- `(?)` opens a bullet nobody has vetted. Vetting is the owner deleting it
- the lead and `## open` carry no `(?)`: the lead is rewritten whenever the page under it changes, and an
  open question claims nothing
- one claim, one bullet. A new source for it joins the evidence
- several methods that each work: each its own conclusion, saying the situation it wins in and what it
  costs. The page lays them out; the reader picks
- a number is an estimate. Almost nothing here has an exact value, even in theory: it moves with the
  city, the trade, the competition, the month. So a conclusion names what we are after and what moves
  it, and carries no number of its own; each source's number stays under it as evidence, in that source's
  words. Sources that differ are not in conflict, and are never averaged or widened into one range: how
  far apart they are is how sure the estimate is

## Style

- terse lowercase bullets. no filler, no hedging
- `>` what qualifies the line above
- `//` thinking out loud, the second-order observation
- `reason:` why, under the claim it explains
- `resolved:` links the record in `log/resolved/`
- `(!)` the part that changes how we would act
- evidence names its speaker in full: `Sam Hart: four passed in two days`
- backticks for handles, services and tools
- `*niche*:` opens evidence from one trade under a claim that holds for all. A claim that holds for one
  trade only lives under `market/niches/<trade>/`

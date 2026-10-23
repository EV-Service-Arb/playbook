## info
- (!) 2–6* profiles per managing gmail
  > up to 5–6 when we are added as manager, opsec is clean, and the managing email is high-trust and on an org (workspace)
  > 2–3 at most when we sit on the profiles as owner, or the account is already tainted
  reason: one banned gmail takes every profile on it at once
  resolved: [2026-10-23](../../secondary/resolved/2026-10-23-profiles-per-gmail.md)
  [r1 2026-08-20](https://www.loom.com/share/35e5f66432cc47e284303e545af0c0aa?t=194)
  [r2 2026-10-09](https://www.loom.com/share/6c13eda9c6dd41ff901b5a1e8d2a8b1d?t=341)
  [r2 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=2643)
  [r2 2026-07-24](https://www.loom.com/share/fc2ebac87db2407ab54bf7aef15b7ea2?t=2547)
  [r3 2026-07-03](https://www.loom.com/share/8aa800d5f10848339d6ecc5839b53ada?t=2244)
  [r4 2026-10-09](https://www.loom.com/share/6c13eda9c6dd41ff901b5a1e8d2a8b1d?t=796)
  [r4 2026-07-17](https://www.loom.com/share/92f78b7b1c0e483792e88f68b5c04277?t=3302)
- buy gmails off real people: facebook marketplace and groups, craigslist — pay them, facebook gift cards work — Eric
  [r2 2026-10-23](https://www.loom.com/share/ebbbd994c4a04b8089375226cb930b24?t=3409)
- `Quo` ~$15/mo for the number the VA works out of — calls route `CallRail` → `Quo`, and they log in from their own computer to answer and text
  // heard as "Quo"; Eric says it and `OpenPhone` are the same thing
  reason: a filipino VA can't text a US customer off a filipino area code
  [r2 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=574)
- `Hushed` also works for this — `David`
  [r2 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=2192)
- (!) new numbers draw spam and sales calls every few minutes — filter in `CallRail` from day one: settings → block numbers → filter callers by category; block robocallers, telemarketers and toll-free, *challenge* the rest — Eric
  > challenge plays a message and the caller presses a key to connect. it may cost a lead or two, worth it while the VA keeps up
  [r2 2026-10-23](https://www.loom.com/share/ebbbd994c4a04b8089375226cb930b24?t=3064)
  [r2 2026-10-23](https://www.loom.com/share/ebbbd994c4a04b8089375226cb930b24?t=3120)
  [r2 2026-10-23](https://www.loom.com/share/ebbbd994c4a04b8089375226cb930b24?t=3158)
  [r2 2026-10-23](https://www.loom.com/share/ebbbd994c4a04b8089375226cb930b24?t=3197)
- `TextVerified` for SMS verifications generally
  > google voice numbers are rejected as VOIP — facebook marketplace now asks for phone verification on some posts
  [r4 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=6173)
- location spoofers work, if you use a solid one — Eric links one in the course's video modules
  [r2 2026-10-23](https://www.loom.com/share/ebbbd994c4a04b8089375226cb930b24?t=2251)
  > a member recommends `3uTools`, free // heard as "Three Tools"
  [r4 2026-10-23](https://www.loom.com/share/ebbbd994c4a04b8089375226cb930b24?t=2251)
  [r4 2026-10-23](https://www.loom.com/share/ebbbd994c4a04b8089375226cb930b24?t=2281)
- (!) mock-location apps are visibly spoofed — they all go through the android mock API, so the phone has to be rooted first — `Valera`
  [r4 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=254)
- `Octobrowser` — free, ~100 profiles, makes a phone look like a different device; runs on iphone or android
  > one member is a week into testing it for reviews, earlier manual attempts held ~60%
  [r4 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=5767)
- (!) a bad proxy is worse than no proxy at all — `Valera`
  reason: there are many innocent reasons to be on a VPN and exactly one to be on a burnt shared proxy
  > buy residential, from a small seller, and check it hasn't been circulating
  [r5 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=5887)
- don't run local open-weight models for this — inference sold by the labs is subsidised ~10x below what your own hardware costs — `Valera`
  A: for guardrails, seed the conversation on a weaker model that is already helping, then switch models mid-thread; or frame it as reverse-engineering your own lost repo. ~80% hit rate
  [r5 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=3932)
- every past coaching call is already transcribed with timestamps — skool homepage → coaching calls → open the loom → transcript pane
  [r3 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=4026)

## facts
- root, then `Magisk`, then any GPS spoofer from its store — "GPS Locator", but there are a hundred and they all work
  // heard as "Majestic"
  > rooting is the hard part; youtube it and have chatgpt walk you through
  > if you skip rooting you can still spoof by tethering the phone to a computer — slower, works
  [r3 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=273)
- `Octobot` on desktop: load the cookies that ship with a bought aged gmail, then a private residential *mobile* proxy, and the session reads as that person on their phone
  [r3 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=5945)
- (!) `Privacy.com` issues prepaid cards where you set the name, address and zip to anything and it still clears
  > used to buy the cheapest google workspace plan per gmail without every account sharing one payment identity
  [r3 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=4591)
- `accsmarket` sells facebook accounts with ad manager already enabled // heard as "Accounts Market"
  > `Jordan` says buying business managers is routine and has a dedicated supplier
  > post from a bought account, never your personal one
  [r3 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=6203)

## plays
- TODO: `Virtual Master` on android — many virtual phones on one handset, one residential IP each, reportedly enough for video verification
  // heard as "Virtual Master" and "PrivacyMaster", unverified, both second-hand from a forum post
  [r5 2026-09-18](https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=4180)

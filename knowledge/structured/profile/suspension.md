# suspension
<!--half-life 1y-->

google suspends a profile it does not trust: a fresh gmail, a NAP shared with another profile, content that does not fit the trade, a young profile changing what identifies it. a suspended
profile is appealed at once, on its own, with documents that match it, which win back a video-verified profile far more often than an instant one; google support can give a taken-down one the video option back. registering the business: [registration](suspension/registration.md).

## the gmail

- aged gmails can directly get suspended themselves, so prefer to have actual emails own them
  <!--2026-09-18 gmbpp member heard 0.5σ https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=3173-->
  - Matt Gazda: a brand new gmail, or one bought "off a plug", reads as suspicious: google weighs the email's activity and history
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=64-->
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=98-->
- (?) (!) a banned gmail takes every profile verified on it with it, and winning the gmail appeal does not bring them back — each profile must then be appealed individually
  <!--2026-09-18 gmbpp member ran 2σ https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=2105-->

## the domain

- several profiles share one `.com` domain, each on a subdomain of its own carrying its own NAP
  > one domain to pay for. if google links profiles by their domain, they share its fate: nobody has measured that
  - switch all to sub-domains // [site_conductor#117](https://github.com/EV-Service-Arb/site_conductor/issues/117)
    <!--2026-11-18 Valera says 1σ-->
  - Matt Gazda: reusing a domain, use subdomains — "this is actually meta right now" — with a different NAP on each
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=167-->
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=200-->
    > only to save money, or where the profiles are branded // transcript garbled there
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=234-->
- (?) a domain per profile, the site duplicated onto it and the name swapped: costs a domain each, and nothing on the site ties one profile to another
  - (?) Eric: the shared domain is what links profiles; a company running a real brand shares one site. a safety margin, not a measured result
    <!--2026-09-18 Eric says 1σ https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=2543-->
  - Eric says that brand names matching is fine, but website matching could technically become problematic
    <!--2026-09-18 Eric says 1σ https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=2504-->
    Q: what's a reasonable number though?

    oh, and apparently you can keep the name be the same, and just spread out domains later
    <!--2026-09-18 Eric says 1σ https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=2558-->
  - (?) Everett, 2026-11: a different URL per profile, mostly the same site reworded with different keywords
    <!--2026-11-20 gmbpp member ran 2σ https://www.loom.com/share/41b6d61558844002ab0ad40951b56546?t=14847-->
- must use .com domains
  <!--2026-11-18 Valera says 1σ-->
  - Matt Gazda: `.com`, never `site.co.ai`, `.it`, `.net`: zero trust signals with google. people he knows lost every profile over the domain, and he has seen profiles wiped in a day for it
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=200-->
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=234-->

## NAP

- one NAP per profile, the same on the profile, its site and every citation, and never shared with another profile in its niche
  - Matt Gazda: the same NAP on two or three profiles gets them suspended, and they would not rank anyway
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=133-->
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=167-->
  - (?) Connor, 2026-11: his first 20 *detailing* GMBs, on three emails and all on one phone number, were all taken down within a month
    <!--2026-11-20 Connor ran 2σ https://www.loom.com/share/41b6d61558844002ab0ad40951b56546?t=10215-->

## what is on the profile

- everything on the profile fits its trade: no AI photos that look fake, no other trade's content, no site unrelated to it
  - Matt Gazda: locksmithing on a plumbing profile, "a random website for your website"
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=98-->
- (?) one brand's photos and descriptions can be shared across its profiles, but a profile in another city gets its own description; the risk is someone who dislikes you seeing every profile is yours, so vary the branding
  - (?) Eric, 2026-11: a member had 10 under one brand until someone figured out what he was doing
    <!--2026-11-06 Eric says 1σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=1472-->
    <!--2026-11-06 Eric says 1σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=2864-->
    <!--2026-11-06 gmbpp member ran 2σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=1472-->
- (?) whoever works on the profile logs in from the US or Canada, VPN or not, and from any state; an offshore login, a Philippine VA's or a `Fiverr` seller's, does more harm than good
  resolved: [2026-11-21](../../log/resolved/2026-11-21-us-ip-only.md)
  - only US IPs work on the profile: no offshore SEO, no `Fiverr`
    - Matt Gazda: do it yourself
      <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=98-->
  - (?) Eric, 2026-11: a VPN to manage a U.S. profile from Canada makes no difference, and logging in from another state is no concern; a Filipino VA logging in from a Philippine IP does more harm than good
    <!--2026-11-06 Eric says 1σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=2797-->
    <!--2026-11-06 Eric says 1σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=2895-->
    <!--2026-11-06 Eric says 1σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=418-->

## changes

- (?) on a young profile the name, the address and the domain stay alone; its phone number changes without trouble on a video-verified one, untried on an instant one. photos, posts, products, chat, citations and opening hours change freely
  resolved: [2026-11-21](../../log/resolved/2026-11-21-young-profile-phone.md)
  - (!) leave a young profile's NAP and website domain alone; photos, posts, products, chat and citations change freely
    > a name change that cannot wait touches a small part of the name: `landscaping of Arizona` → `flourish landscaping of Arizona`
    - Matt Gazda: NAP and domain are the highest trust signals google reads. the first 6 months, the first year to be careful; his slide says 4 months
      <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=268-->
      <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=301-->
    - (?) Eric, 2026-11: opening hours changed with reviews coming in make no difference
      <!--2026-11-13 Eric says 1σ https://www.loom.com/share/28f60c76396841d39b9afc506a4bf805?t=948-->
    - (?) Eric, 2026-11: never rename a profile, it typically triggers re-verification; Chris's two, two to three months old, renamed to the city and "Plumber", vanished and brought no lead
      <!--2026-11-06 Eric says 1σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=1829-->
      <!--2026-11-06 Chris ran 2σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=1743-->
      <!--2026-11-06 Chris ran 2σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=1802-->
    - (?) Eric, 2026-11, asked by Alan, who put the wrong number on his video-verified *plumbing* profile: the phone number is not a big deal
      <!--2026-11-20 Eric says 1σ https://www.loom.com/share/41b6d61558844002ab0ad40951b56546?t=572-->

## citations

- citations whose NAP matches the profile's protect it from suspension; mismatched ones cause it
  - Matt Gazda: `Yelp`, `Facebook`, `X`, `Nextdoor`. "a huge sleeper", the best way never to be suspended again
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=336-->
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=369-->
  - (?) Eric, 2026-11: for a suspended instant-verified profile, citations aren't worth the effort
    <!--2026-11-06 Eric says 1σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=2507-->

## the address and the rating

- (?) a second profile on one address in the same niche can trip suspension; one in a different niche, or on its own suite, ranks for other categories and searches
  resolved: [2026-11-21](../../log/resolved/2026-11-21-one-address.md)
  - a second profile on the same address can trip google's automatic suspension
    - Matt Gazda: seen it, unsure how it works
      <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=369-->
  - (?) Eric, 2026-11: different niches on one address do have a point, ranking for different categories and searches
    <!--2026-11-20 Eric says 1σ https://www.loom.com/share/41b6d61558844002ab0ad40951b56546?t=2697-->
  > three videos at one address, as suites or as insurance: [video](verification/video.md#attempts)
- a flood of negative reviews suspends a profile
  - Matt Gazda: his energy-drink table's QR code showed in a viral instagram clip; ~1.4 stars, then suspended
    <!--2026-11-17 ran 2σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=403-->

## suspended profiles

- (?) suspended profiles still appear in maps and still take calls — what you lose is the ability to edit or optimise them
  > so an unsuspended-looking but suspended profile is not urgent; several people are sitting on them deliberately
  <!--2026-09-18 gmbpp member ran 2σ https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=2130-->

## appeals

- appeal within the first 24 hours
  - Matt Gazda: a high chance of reinstatement, even submitting with no evidence. he does not know whether an AI or a person reads it
    <!--2026-11-17 says 1σ https://www.youtube.com/watch?v=0IRfNgq_4zY&t=33-->
- (?) appealing with pasted chatgpt text got auto-rejected and escalated into the full gmail ban
  <!--2026-09-18 gmbpp member ran 2σ https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=2081-->
- (?) what actually worked: transfer the profiles to a fresh account, then appeal and re-verify from there — Eric, after a gmail suspension took 10 profiles
  <!--2026-09-18 Eric ran 2σ https://www.loom.com/share/c197556c940b4d01b53376dab70d32e5?t=3191-->
- (?) official business registration, business licence, tax certificates, utility bills (electricity/phone/water/internet) are the reinstatement evidence google asks for
  <!--2026-09-18 says 1σ https://chatgpt.com/share/6aacb045-ac9c-83eb-a97d-8624c616a763-->
  - (?) Derick Parsons, 2026-11: a Fiverr seller Henry A put him on to made the documents he lacked; he skipped the request for additional information, and the profile stayed up and brought two leads the day it came back
    <!--2026-11-06 Derick Parsons ran 2σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=3520-->
    <!--2026-11-06 Derick Parsons ran 2σ https://www.loom.com/share/83a2bc8ce49142a8aec1ce5b3976b98e?t=3561-->
  > they work on a video-verified profile far better than on an instant one: [verification](verification.md#routes)
- (?) name and address on those documents must match the profile being appealed
  <!--2026-09-18 says 1σ https://chatgpt.com/share/6aacb045-ac9c-83eb-a97d-8624c616a763-->
- (?) a taken-down profile can win back the video option through google support, by persisting: messages back and forth, with documents
  - (?) Connor, 2026-11: got it once, for a *detailing* profile: about 10 messages plus documents, and the option a week later
    <!--2026-11-20 Connor ran 2σ https://www.loom.com/share/41b6d61558844002ab0ad40951b56546?t=10043-->
  - (?) Everett, 2026-11: gave up after a 16-email thread of the same questions
    <!--2026-11-20 gmbpp member ran 2σ https://www.loom.com/share/41b6d61558844002ab0ad40951b56546?t=10078-->
- (?) appeal each profile on its own, with documents whose name and address match it. a won gmail appeal does not bring its profiles back // inferred
  <!--2026-10-30 agent says 1σ-->

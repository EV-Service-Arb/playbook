The playbook is the `service-arb` plugin: a skill that says how to answer, and the `playbook`
server it answers from, a remote MCP server at `https://sa.evinvest.ltd/playbook_mcp` behind the
Service-Arb panel. Any member granted `sa:playbook:mcp:use` can install it in Claude Code, Codex,
claude.ai or ChatGPT. There is no key to copy between devices: each one signs in through the panel
in the browser.

```
member: install service-arb in a client ──► browser: sa.evinvest.ltd sign-in
   no access yet ──► "Request access" ──► owner: Telegram ping → "Grant access" ──► the tab continues
   access ──► "Allow access" ──► back in the client, connected
```

## For the owner: giving someone access

1. Send them this Installation section, as is. Nothing else needs sharing: no token, no file, no invite.
2. When they press **Request access**, the panel's Telegram bot pings `sa:admin` holders. Its
   **Grant access** button opens the banking cabinet's grant form with their email and
   `sa:playbook:mcp:use` filled in; submit it. Their waiting tab carries on by itself.

Revoking the permission stops new sign-ins at once, but a client already connected keeps working
for up to 30 days: refreshing its token does not re-check the permission, so it bites when their
client next has to sign in again.

## For members: connecting

1. Install the plugin in your client: open its section below. Each one opens the sign-in in the browser.
   A `service-arb` server you added by hand before the plugin goes: two copies of the same tools get
   in each other's way.
2. The browser opens `sa.evinvest.ltd`. Sign in with Google if asked (any account; the first sign-in
   creates it). Without access yet, press **Request access** and keep the tab open: it moves on by
   itself once the owner grants it. Then press **Allow access** on the page naming your client.
3. The browser sends you back to the client, now connected.

## When something goes wrong

| what you see | what it means |
|---|---|
| "No access to the playbook yet" | you are signed in, but not granted `sa:playbook:mcp:use`. **Request access**, or **Use a different account** |
| "This sign-in link has expired" | the page sat open over 10 minutes, or was submitted twice. Sign in again |
| the client reports `access_denied` | **Cancel** was pressed. Sign in again |
| the client asks you to sign in again | normal: sign-in lasts 30 days. Sign in again |
| "…has used today's … bytes" (429) | the daily reading budget is spent; it resets at 00:00 UTC |
| the playbook's tools listed twice, under `service-arb` and under `playbook` | a server added by hand before the plugin is still there: `claude mcp remove service-arb`, `codex mcp remove service-arb`, or delete the `Service Arb` connector |
| an old `sa.valeratrades.com` or `service-arb.valeratrades.com` entry | that address is gone: remove it as in the row above, then step 1 |
| answers that ignore the playbook's sources | the skill is not installed or not on: in ChatGPT without Skills, see its section |

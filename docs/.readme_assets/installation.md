The playbook is the `service-arb` plugin: a skill that says how to answer, and the `playbook`
server it answers from: the Service-Arb panel's MCP server at `https://sa.evinvest.ltd/mcp`, whose
`playbook_*` tools read the playbook beside the panel's own `search` and `call`. Any member granted
`sa:playbook:mcp:use` can install it in Claude Code, Codex, claude.ai or ChatGPT. There is no key to
copy between devices: each one signs in through the panel in the browser.

```
member: install service-arb in a client ──► browser: sa.evinvest.ltd sign-in
   no access yet ──► "Request access" ──► owner: Telegram ping → "Grant access" ──► the tab continues
   access ──► "Allow" ──► back in the client, connected
```

## For the owner: giving someone access

1. Send them this Installation section, as is. Nothing else needs sharing: no token, no file, no invite.
2. When they press **Request access**, the panel's Telegram bot pings `sa:admin` holders. Its
   **Grant access** button opens the banking cabinet's grant form with their email and
   `sa:playbook:mcp:use` filled in; submit it. Their waiting tab carries on by itself.

Revoking the permission bites within a minute, connected clients included: the panel checks it on
every call.

## For members: connecting

1. Install the plugin in your client: open its section below. Each one opens the sign-in in the browser.
   A `service-arb` server you added by hand before the plugin goes: two copies of the same tools get
   in each other's way.
2. The browser opens `sa.evinvest.ltd`. Sign in with Google if asked (any account; the first sign-in
   creates it). Without access yet, press **Request access** and keep the tab open: it moves on by
   itself once the owner grants it. Then press **Allow** on the page naming your client.
3. The browser sends you back to the client, now connected.

## When something goes wrong

| what you see | what it means |
|---|---|
| "No access yet" | you are signed in, but not a member. **Request access**, or **Use a different account** |
| the client reports `access_denied` | **Cancel** was pressed. Sign in again |
| the client asks you to sign in again | normal: your panel session ended. Sign in again |
| `over_budget`: "…has used today's … bytes" | the daily reading budget is spent; it resets at 00:00 UTC |
| the playbook's tools listed twice, under `service-arb` and under `playbook` | a server added by hand before the plugin is still there: `claude mcp remove service-arb`, `codex mcp remove service-arb`, or delete the `Service Arb` connector |
| Claude Code lists both `service-arb@ev-service-arb` and `service-arb@synced` | installed in Claude Code and on claude.ai both: `claude plugin uninstall service-arb@ev-service-arb` |
| an old `sa.valeratrades.com` or `service-arb.valeratrades.com` entry | that address is gone: remove it as in the row above, then step 1 |
| answers that ignore the playbook's sources | the skill is not installed or not on: in ChatGPT without Skills, see its section |

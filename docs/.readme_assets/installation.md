The playbook is a remote MCP server at `https://sa.evinvest.ltd/playbook_mcp`, behind the
Service-Arb panel. Any member granted `sa:playbook:mcp:use` can connect it to Claude Code, Codex,
claude.ai or ChatGPT. Nothing is installed locally, and there is no key to copy between devices: each device
signs in through the panel in the browser.

```
member: add service-arb in a client ──► browser: sa.evinvest.ltd sign-in
   no access yet ──► "Request access" ──► owner: Telegram ping → "Grant access" ──► the tab continues
   access ──► "Allow access" ──► back in the client, connected
```

## For the owner: giving someone access

1. Send them the section below, as is. Nothing else needs sharing: no token, no file, no invite.
2. When they press **Request access**, the panel's Telegram bot pings `sa:admin` holders. Its
   **Grant access** button opens the banking cabinet's grant form with their email and
   `sa:playbook:mcp:use` filled in; submit it. Their waiting tab carries on by itself.

Revoking the permission stops new sign-ins at once, but a client already connected keeps working
for up to 30 days: refreshing its token does not re-check the permission, so it bites when their
client next has to sign in again.

## For members: connecting

1. Add the server in your client, from the table below. Each one opens the sign-in in the browser.
2. The browser opens `sa.evinvest.ltd`. Sign in with Google if asked (any account; the first sign-in
   creates it). Without access yet, press **Request access** and keep the tab open: it moves on by
   itself once the owner grants it. Then press **Allow access** on the page naming your client.
3. The browser sends you back to the client, now connected.

| client | add it | sign in again |
|---|---|---|
| Claude Code | `claude mcp add --scope user --transport http service-arb https://sa.evinvest.ltd/playbook_mcp`, then in Claude Code: `/mcp` → `service-arb` → **Authenticate** | `/mcp` → **Authenticate** |
| Codex | `codex mcp add service-arb --url https://sa.evinvest.ltd/playbook_mcp` | `codex mcp login service-arb` |
| claude.ai, and its desktop and mobile apps | Settings → Connectors → **Add custom connector**. Name `service-arb`, URL `https://sa.evinvest.ltd/playbook_mcp`, then **Connect**. Turn it on in a chat from the tools menu | **Connect** on the connector |
| ChatGPT, paid plans | Settings → Apps & Connectors → Advanced: turn on **Developer mode**. Then **Create**: name `service-arb`, URL `https://sa.evinvest.ltd/playbook_mcp`, authentication OAuth. Select it in a chat under **+** → Developer mode | **Connect** on the connector |

Claude Code and Codex connect one device: do step 1 on each device. A claude.ai or ChatGPT connector
is on the account, so every device of that account has it.

## When something goes wrong

| what you see | what it means |
|---|---|
| "No access to the playbook yet" | you are signed in, but not granted `sa:playbook:mcp:use`. **Request access**, or **Use a different account** |
| "This sign-in link has expired" | the page sat open over 10 minutes, or was submitted twice. Sign in again |
| the client reports `access_denied` | **Cancel** was pressed. Sign in again |
| the client asks you to sign in again | normal: sign-in lasts 30 days. Sign in again |
| "…has used today's … bytes" (429) | the daily reading budget is spent; it resets at 00:00 UTC |
| an old `sa.valeratrades.com` or `service-arb.valeratrades.com` entry | that address is gone: remove it (`claude mcp remove service-arb`, `codex mcp remove service-arb`, or delete the connector), then step 1 |

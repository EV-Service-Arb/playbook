# playbook
<br>
[<img alt="ci errors" src="https://img.shields.io/github/actions/workflow/status/EV-Service-Arb/playbook/errors.yml?branch=main&style=for-the-badge&style=flat-square&label=errors&labelColor=420d09" height="20">](https://github.com/EV-Service-Arb/playbook/actions?query=branch%3Amain) <!--NB: Won't find it if repo is private-->
[<img alt="ci warnings" src="https://img.shields.io/github/actions/workflow/status/EV-Service-Arb/playbook/warnings.yml?branch=main&style=for-the-badge&style=flat-square&label=warnings&labelColor=d16002" height="20">](https://github.com/EV-Service-Arb/playbook/actions?query=branch%3Amain) <!--NB: Won't find it if repo is private-->

A knowledge base on service arbitrage — running Google Business Profiles into subcontracted trade work — where every claim traces back to a dated source. Members read it through an MCP server, from Claude Code, Codex, claude.ai or ChatGPT; see Installation.

![prompt: "what I need for live call"](docs/.assets/playbook_live_call_advice.png)
![actioning it: following a lead from the answer](docs/.assets/playbook_eric_drive_link.png)
<!-- markdownlint-disable -->
<details>
<summary>
<h2>Installation</h2>
</summary>

The playbook is a remote MCP server at `https://sa.evinvest.ltd/playbook_mcp`, behind the
Service-Arb panel. Any member granted `sa:playbook:mcp:use` can connect it to Claude Code, Codex,
claude.ai or ChatGPT. Nothing is installed locally, and there is no key to copy between devices: each device
signs in through the panel in the browser.

```
member: add service-arb in a client ──► browser: sa.evinvest.ltd sign-in
   no access yet ──► "Request access" ──► owner: Telegram ping → "Grant access" ──► the tab continues
   access ──► "Allow access" ──► back in the client, connected
```

### For the owner: giving someone access

1. Send them this Installation section, as is. Nothing else needs sharing: no token, no file, no invite.
2. When they press **Request access**, the panel's Telegram bot pings `sa:admin` holders. Its
   **Grant access** button opens the banking cabinet's grant form with their email and
   `sa:playbook:mcp:use` filled in; submit it. Their waiting tab carries on by itself.

Revoking the permission stops new sign-ins at once, but a client already connected keeps working
for up to 30 days: refreshing its token does not re-check the permission, so it bites when their
client next has to sign in again.

### For members: connecting

1. Add the server in your client: open its section below. Each one opens the sign-in in the browser.
2. The browser opens `sa.evinvest.ltd`. Sign in with Google if asked (any account; the first sign-in
   creates it). Without access yet, press **Request access** and keep the tab open: it moves on by
   itself once the owner grants it. Then press **Allow access** on the page naming your client.
3. The browser sends you back to the client, now connected.

### When something goes wrong

| what you see | what it means |
|---|---|
| "No access to the playbook yet" | you are signed in, but not granted `sa:playbook:mcp:use`. **Request access**, or **Use a different account** |
| "This sign-in link has expired" | the page sat open over 10 minutes, or was submitted twice. Sign in again |
| the client reports `access_denied` | **Cancel** was pressed. Sign in again |
| the client asks you to sign in again | normal: sign-in lasts 30 days. Sign in again |
| "…has used today's … bytes" (429) | the daily reading budget is spent; it resets at 00:00 UTC |
| an old `sa.valeratrades.com` or `service-arb.valeratrades.com` entry | that address is gone: remove it (`claude mcp remove service-arb`, `codex mcp remove service-arb`, or delete the connector), then step 1 |
<!-- markdownlint-disable -->
<details>
<summary>
<h3>Chatgpt</h3>
</summary>

Paid plans. The plugin is on the account: every device signed in to it
gets it.

1.  Sidebar: **Plugins** → **Add** → **Add custom MCP server**

    ![](./docs/.readme_assets/assets/chatgpt_plugins_add.png)

2.  Name: `Service Arb`, or your own. Description: what it covers, so
    ChatGPT knows when to use it. URL:
    `https://sa.evinvest.ltd/playbook_mcp`. Authentication: **OAuth**.
    Tick **I understand and want to continue**, then **Create as a
    plugin**, and sign in.

    <img src="./docs/.readme_assets/assets/chatgpt_custom_mcp.png" width="50.0%" />

3.  In a chat: **+** → **Service Arb**. To keep it off the web, say so:
    “answer only from service-arb”.

    <img src="./docs/.readme_assets/assets/chatgpt_chat_plus.png" width="60.0%" />

Sign in again: **Connect** on the plugin.


</details>
<!-- markdownlint-restore -->
<!-- markdownlint-disable -->
<details>
<summary>
<h3>Claude App</h3>
</summary>

The connector is on the account: every device signed in to it, browser,
desktop and mobile, gets it.

1.  Bottom left, your name → **Settings**

    <img src="./docs/.readme_assets/assets/claude_settings.png" width="35.0%" />

2.  **Connectors** → **Add** → **Add custom connector**

    <img src="./docs/.readme_assets/assets/claude_connectors.png" width="70.0%" />

3.  Name: `Service Arb`, or your own. URL:
    `https://sa.evinvest.ltd/playbook_mcp`. Then **Continue**, and sign
    in.

    <img src="./docs/.readme_assets/assets/claude_custom_connector.png" width="50.0%" />

4.  In a chat, turn it on from the tools menu.

Sign in again: **Connect** on the connector.


</details>
<!-- markdownlint-restore -->
<!-- markdownlint-disable -->
<details>
<summary>
<h3>Claude Code</h3>
</summary>

Connects one device: run it on each.

``` sh
claude mcp add --scope user --transport http service-arb https://sa.evinvest.ltd/playbook_mcp
```

Then in Claude Code: `/mcp` → `service-arb` → **Authenticate**. Sign in
again the same way.


</details>
<!-- markdownlint-restore -->
<!-- markdownlint-disable -->
<details>
<summary>
<h3>Codex</h3>
</summary>

Connects one device: run it on each.

``` sh
codex mcp add service-arb --url https://sa.evinvest.ltd/playbook_mcp
```

Sign in again: `codex mcp login service-arb`.


</details>
<!-- markdownlint-restore -->


</details>
<!-- markdownlint-restore -->

## Usage
Ask about service arbitrage as usual; with `service-arb` connected it answers from the playbook on its own, citing the dated source of each claim:

| tool | returns |
|---|---|
| `search(pattern, scope?, screen?)` | matching passages: approved notes first, then suggested, the course, then calls, videos and texts, each kind in its own slots |
| `read(path, header, line?)` | one chapter of a capture or one section of a note |
| `guide(section)` | how to decide in one area (`reviews`, `verification`, …) |

To hold a conversation to the playbook, say so ("answer from service-arb"); in Claude Code, `/mcp__service-arb__ask` does the same.



<br>

<sup>
	This repository follows <a href="https://github.com/valeratrades/.github/tree/master/best_practices">my best practices</a> and <a href="https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md">Tiger Style</a> (except "proper capitalization for acronyms": (VsrState, not VSRState) and formatting). For project's architecture, see <a href="./docs/ARCHITECTURE.md">ARCHITECTURE.md</a>.
</sup>

#### License

<sup>
	Licensed under <a href="LICENSE">Blue Oak 1.0.0</a>
</sup>

<br>

<sub>
	Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be licensed as above, without any additional terms or conditions.
</sub>


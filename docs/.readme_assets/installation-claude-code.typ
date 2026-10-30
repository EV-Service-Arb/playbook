Connects one device: run it on each.

```sh
claude plugin marketplace add EV-Service-Arb/playbook --sparse .claude-plugin plugin
claude plugin install service-arb@ev-service-arb
claude mcp remove service-arb  # only if you added the server by hand before
```

`--sparse` fetches the plugin alone, not the whole knowledge base. Then in Claude Code: `/mcp` → `plugin:service-arb:playbook` → *Authenticate*. Sign in again the same way.

Installed on claude.ai already: Claude Code has it as `service-arb@synced`, so skip the install, or it runs twice.

Updates: `/plugin` → *Marketplaces* → `ev-service-arb` → *Enable auto-update*. Without it, `claude plugin marketplace update ev-service-arb`; the skill otherwise stays as installed.

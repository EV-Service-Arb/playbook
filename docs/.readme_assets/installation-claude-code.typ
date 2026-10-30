Connects one device: run it on each.

```sh
claude plugin marketplace add EV-Service-Arb/playbook
claude plugin install service-arb@ev-service-arb
claude mcp remove service-arb  # only if you added the server by hand before
```

Then in Claude Code: `/mcp` → `plugin:service-arb:playbook` → *Authenticate*. Sign in again the same way.

Updates: `/plugin` → *Marketplaces* → `ev-service-arb` → *Enable auto-update*. Without it the skill stays as installed.

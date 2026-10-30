Connects one device: run it on each. Needs a Codex with `codex plugin`: `codex update`.

```sh
codex plugin marketplace add EV-Service-Arb/playbook --sparse .claude-plugin --sparse plugin
codex plugin add service-arb@ev-service-arb
codex mcp login playbook
codex mcp remove service-arb  # only if you added the server by hand before
```

`--sparse` fetches the plugin alone: without it Codex downloads the whole knowledge base, gigabytes. Start a new session afterwards. Sign in again: `codex mcp login playbook`. Updates: `codex plugin marketplace upgrade ev-service-arb`.

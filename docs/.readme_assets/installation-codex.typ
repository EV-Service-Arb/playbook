Connects one device: run it on each. Needs a Codex with `codex plugin`: `codex update`.

```sh
codex plugin marketplace add EV-Service-Arb/playbook
codex plugin add service-arb@ev-service-arb
codex mcp remove service-arb  # only if you added the server by hand before
```

Then start a new session, and sign in when it asks. Updates: `codex plugin marketplace upgrade`.

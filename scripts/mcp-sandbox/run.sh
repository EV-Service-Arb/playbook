# Your own Claude Code without the service-arb plugin, any hand-added service-arb server, or their tokens, so the install runs as for a new member; auth opens a Chromium with no sessions.
# Wiped every run; `--keep` resumes the last one.
s="$(git rev-parse --show-toplevel)/scripts/mcp-sandbox/tmp"
if [ "${1:-}" = "--keep" ]; then shift; else rm -rf "$s"; fi

if [ ! -d "$s" ]; then
  mkdir -p "$s/cfg" "$s/chrome" "$s/bin"
  for f in settings.json CLAUDE.md rules skills agents hooks keybindings.json; do # not plugins, which would carry the real install's service-arb; nor the daemon or session state, which the real install owns
    if [ -e ~/.claude/"$f" ]; then ln -s ~/.claude/"$f" "$s/cfg/$f"; fi
  done
  jq '.mcpOAuth |= with_entries(select(.key | test("^(service-arb|plugin:service-arb:)") | not))' ~/.claude/.credentials.json > "$s/cfg/.credentials.json"
  chmod 600 "$s/cfg/.credentials.json"
  jq --arg s "$s" 'del(.mcpServers["service-arb"]) | .projects |= map_values(del(.mcpServers["service-arb"])) | .projects[$s].hasTrustDialogAccepted = true' ~/.claude.json > "$s/cfg/.claude.json"
  cat > "$s/bin/xdg-open" <<SHIM
#!/bin/sh
exec chromium --user-data-dir="$s/chrome" --no-first-run --no-default-browser-check "\$@" >/dev/null 2>&1 &
SHIM
  chmod +x "$s/bin/xdg-open"
  ln -s xdg-open "$s/bin/open"
fi

export CLAUDE_CONFIG_DIR="$s/cfg" PATH="$s/bin:$PATH" BROWSER="$s/bin/xdg-open"
cd "$s"
if ! claude plugin list 2>/dev/null | grep -q 'service-arb@ev-service-arb'; then
  claude plugin marketplace add EV-Service-Arb/playbook
  claude plugin install service-arb@ev-service-arb
fi
exec claude "$@"

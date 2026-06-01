# rustscale — Agent instructions

This project is `rustscale`, a Rust MCP server that exposes the Tailscale API to AI agents. The binary is named `tailscale`.

## Repository layout

```
src/
  tailscale.rs        HTTP client for api.tailscale.com/api/v2
  app.rs              TailscaleService: business logic, destructive gate
  mcp/
    tools.rs          MCP tool dispatch shim (no logic here)
    schemas.rs        JSON Schema for the tailscale tool
    rmcp_server.rs    RMCP ServerHandler
    routes.rs         Axum router
    prompts.rs        MCP prompts
  mcp.rs              AppState, AuthPolicy, build_auth_layer
  config.rs           TailscaleConfig, McpConfig, AuthConfig, env loading
  cli.rs              CLI shim (no logic here)
  main.rs             Mode dispatch, resolve_auth_policy
  lib.rs              Public surface + testing module
tests/
  cli_parse.rs        CLI arg parsing tests
  destructive_gate.rs Destructive gate unit tests
  tool_dispatch.rs    MCP dispatch shim tests
```

## What the server does

Exposes a single MCP tool `tailscale` with action dispatch. Actions:

**Read:** `devices`, `device`, `device_routes`, `keys`, `acl`, `dns`, `users`
**Write:** `authorize_device`
**Destructive:** `delete_device` (requires `TAILSCALE_ALLOW_DESTRUCTIVE=true` AND `confirm=true`)
**Meta:** `help`

## Key constraints for agents

1. **Do not add logic to shims.** `mcp/tools.rs` and `cli.rs` parse args and call `TailscaleService`. All logic belongs in `app.rs`. All HTTP belongs in `tailscale.rs`.

2. **Destructive gate is in `app.rs` only.** Do not check `allow_destructive` or `confirm` anywhere else.

3. **Do not touch `src/`.** This project's task is documentation and test compilation. Source code is correct as-is.

4. **Tests do not require a live Tailscale account.** They use a stub API key and verify gate logic, dispatch routing, and CLI parsing in-process.

## Running tests

```bash
cargo test --no-run    # compile only
cargo test             # compile + run
```

## Environment setup for testing

No env vars needed for the test suite. The tests construct `AppState` directly from `rustscale::mcp::testing` helpers.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->

## Plugin setup hooks

Plugin setup is owned by the binary. `plugins/tailscale/hooks/hooks.json` calls `${CLAUDE_PLUGIN_ROOT}/bin/rtailscale setup plugin-hook` directly (no shell wrapper). The binary's `apply_plugin_options()` (`src/setup.rs`), run at the top of the plugin-hook path, maps `CLAUDE_PLUGIN_OPTION_*` values to the binary's `TAILSCALE_*` env vars (and `CLAUDE_PLUGIN_DATA` → `TAILSCALE_MCP_HOME`); `install_self()` self-installs the binary into `~/.local/bin`.

`tailscale setup check` is read-only, `tailscale setup repair` is idempotent, and `tailscale setup plugin-hook --no-repair` is audit mode. Do not add Docker Compose, systemd, or service bootstrap logic into the hook path.

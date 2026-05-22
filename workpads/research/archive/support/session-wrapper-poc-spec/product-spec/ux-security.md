# Session Wrapper Product Spec: UX, Security, And Privacy

## UX Principles

- No ambient auth: auth is always selected explicitly.
- No surprise browser lifecycle changes.
- No broad session exports by default.
- Non-interactive by default so agents can call commands reliably.
- Any command that may require user action must fail with a clear machine-readable reason unless `--interactive` or a dedicated interactive subcommand is used.
- No cookie values in normal logs.
- Sensitive temporary files are created in OS temp storage and deleted immediately.
- Authenticated outputs are sensitive too.
- Every run should explain which sessions were used and which domains were allowed.
- Every imported session should be inspectable without revealing values.
- Timeouts should be explicit and configurable; no command should hang indefinitely.

## Security And Privacy Requirements

- Treat cookies, localStorage, sessionStorage, and exported browser state as credential-equivalent.
- Store persisted session material outside project repos by default.
- Use restrictive filesystem permissions for session files.
- Avoid writing raw broad state files except as short-lived temp files.
- Do not include session values in command output unless `--show-secrets` is explicitly passed.
- Do not commit session files or authenticated outputs.
- Keep hosted services out of authenticated fetch paths.
- Make provider-session inclusion explicit.
- Always post-filter imported cookies/storage against allowlists, even if the backend claims to filter.

# Release Binary GitHub Private Repo Smoke

Date: 2026-05-22

Code snapshot: `b5d1d88786c9d5082ea120d24de47bd1069736c3` (`b5d1d88 Split login-finish session tests`)

## Objective

Exercise the `target/release/aget` binary against a real public GitHub page and private GitHub repositories, using only local session state for authenticated access.

The concrete questions were:

- Can the release binary fetch public GitHub content?
- Does an unauthenticated private GitHub repository fetch fail in an expected way?
- Can `aget` reuse an existing browser-authenticated GitHub session by import?
- If import is not enough, can a user-driven login create reusable local state?
- Can the resulting local session fetch more than one private repository under the same GitHub account?

## Environment

- Binary: `target/release/aget`
- Working tree: `/Users/nicolas/devel/aget`
- Network and home-directory access required escalation in the agent sandbox.
- Private output artifacts were written under `/private/tmp`.
- Session name chosen for the reusable GitHub session: `github`.
- Auth/session scope: `github.com` plus GitHub storage origin saved by the login flow.

## Commands And Results

### Public repository fetch

Command:

```bash
target/release/aget get https://github.com/nicolaslara/aget \
  --output /private/tmp/aget-public-readme.md \
  --max-chars 12000
```

Result:

- Succeeded.
- Output contained the public `aget` README content.
- Artifact: `/private/tmp/aget-public-readme.md`.

### Private repository without session

Command:

```bash
target/release/aget get https://github.com/nicolaslara/zodl-desktop \
  --output /private/tmp/zodl-desktop-nosession.md \
  --max-chars 4000
```

Result:

- Command exited successfully as an extraction, but GitHub returned its generic unauthenticated/private-repo page.
- This is the expected product behavior for a generic fetcher: `aget` extracted what the site returned; the caller inferred that auth was needed.

### Existing browser session import attempts

Commands tested:

```bash
target/release/aget session authorize github \
  --url https://github.com/nicolaslara/zodl-desktop \
  --browser chrome \
  --browser-profile Default \
  --allow-domain github.com \
  --must-contain zodl-desktop \
  --output /private/tmp/zodl-desktop-github-session.md \
  --timeout 30
```

```bash
target/release/aget session authorize github \
  --url https://github.com/nicolaslara/zodl-desktop \
  --browser chrome \
  --browser-profile '/Users/nicolas/Library/Application Support/Arc/User Data/Profile 1' \
  --allow-domain github.com \
  --must-contain zodl-desktop \
  --output /private/tmp/zodl-desktop-github-session.md \
  --timeout 30
```

```bash
target/release/aget session authorize github \
  --url https://github.com/nicolaslara/zodl-desktop \
  --browser chrome \
  --browser-profile Default \
  --allow-domain .github.com \
  --must-contain zodl-desktop \
  --output /private/tmp/zodl-desktop-github-session.md \
  --timeout 30
```

Result:

- Chrome `Default` with `github.com` exported no auth state.
- Explicit Arc profile path through the Chrome importer exported no auth state.
- Chrome `Default` with `.github.com` exported no auth state.
- `session authorize --browser arc` was advertised by help but rejected by implementation.

### User-driven login flow

Commands:

```bash
target/release/aget session login start github \
  --url https://github.com/nicolaslara/zodl-desktop \
  --timeout 30
```

After the user completed GitHub login in the opened browser:

```bash
target/release/aget session login finish github --timeout 30
```

Result:

- Succeeded.
- Saved session `github` with 13 cookies and 1 storage origin.
- Redacted inspection showed GitHub cookie domains and `https://github.com` storage origin.
- No cookie values or private README contents were recorded in this experiment log.

### Private `zodl-desktop` fetch with session

Commands:

```bash
target/release/aget get https://github.com/nicolaslara/zodl-desktop \
  --session github \
  --output /private/tmp/zodl-desktop-readme.md \
  --max-chars 12000 \
  --timeout 30
```

```bash
target/release/aget get https://github.com/nicolaslara/zodl-desktop/blob/main/README.md \
  --session github \
  --output /private/tmp/zodl-desktop-readme-only.md \
  --max-chars 12000 \
  --timeout 30
```

Result:

- Both succeeded.
- The repository page extraction showed the private repo file list and README section.
- The direct README blob extraction produced a cleaner README-only artifact.
- Artifacts:
  - `/private/tmp/zodl-desktop-readme.md`
  - `/private/tmp/zodl-desktop-readme-only.md`

### Second private repository fetch with same session

Command:

```bash
target/release/aget get https://github.com/nicolaslara/ai \
  --session github \
  --output /private/tmp/nicolaslara-ai-repo.md \
  --max-chars 12000 \
  --timeout 30
```

Result:

- Succeeded with the same `github` session.
- Output identified `nicolaslara/ai` as a private repository.
- It showed repository metadata including 2 branches, 48 commits, repository file list, and GitHub's "Add a README" prompt.
- Artifact: `/private/tmp/nicolaslara-ai-repo.md`.

## Project Status From This Experiment

- The release binary can fetch public GitHub markdown through the owned default extraction path.
- The release binary can persist a reusable local GitHub session through the user-driven login flow.
- Session replay works for multiple private GitHub repositories under the same saved session and allowed scope.
- The generic safety boundary held: unauthenticated private access produced the site response, and the caller decided to request/login with a session.
- Private page content was kept local in output files and redacted session inspection avoided exposing cookie values.

## Issues Found

1. `session authorize --browser arc` is visible in help/enum output but rejected by implementation.
   - Impact: the browser-choice surface over-promises for `authorize`.
   - Follow-up: either support Arc import through this command or hide/mark unsupported choices for `authorize`.

2. Existing-browser import did not recover GitHub auth from Chrome `Default` or the tested Arc profile path.
   - Impact: the preferred OAuth-safe import path may not be reliable enough yet for GitHub.
   - Follow-up: improve diagnostics around which profile was inspected, whether cookies were absent, encrypted/unreadable, or filtered by scope.

3. Private GitHub repository landing pages may include dynamic "Uh oh! There was an error while loading" text even when authenticated extraction succeeds.
   - Impact: agents should prefer direct blob URLs for README/file content when the exact file is known.
   - Follow-up: document this as caller guidance or add higher-level repository/file recipes outside the generic binary.

4. The login flow worked, but GitHub is OAuth-sensitive and may reject automation-controlled browsers in other states.
   - Impact: real-browser import and current-tab extraction remain important companion paths.
   - Follow-up: keep this as positive smoke evidence, not a guarantee that all OAuth providers will allow the controlled login profile.

## Confidence

High for the observed release-binary behavior on this machine: public fetch, private unauthenticated failure mode, user-driven GitHub login, and multi-private-repo session replay all ran successfully.

Medium for generalizing the browser import findings, because profile selection and local browser state are machine-specific.

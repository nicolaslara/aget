### D142: I19i splits mock-site browser/rendering tests

The next integration-test decomposition slice moved browser fallback and Chrome-rendered extraction coverage from `tests/mock_site_cli.rs` to `tests/mock_site_browser.rs`. The new test target owns the owned browser fallback replay test, ignored local-Chrome browser fallback smokes, localStorage-backed rendered extraction, waited JavaScript rendering, script auto-rendering, shadow DOM flattening, render delay, rendered overlay cleanup, image waiting, and network-idle smokes. `tests/mock_site_cli.rs` now keeps the command/default/static extraction and session-oriented mock-site coverage.

Validation:

- `cargo test --test mock_site_cli`
- `cargo test --test mock_site_browser`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the mock-site browser/rendering split. The split is mechanical, both affected integration targets passed, and the full standard suite is green.

### D143: I19i splits mock-site session/auth tests

The next integration-test decomposition slice moved mock-site session and auth scenario coverage from `tests/mock_site_cli.rs` to `tests/mock_site_sessions.rs`. The new test target owns cookie and storage replay, unauthenticated/expired/logout states, imported Chrome session replay through the compatibility mock, and login bootstrap replay. `tests/mock_site_cli.rs` is now focused on command/default/static extraction parity and is below 900 lines.

Validation:

- `cargo test --test mock_site_cli`
- `cargo test --test mock_site_sessions`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the mock-site session/auth split. The split is mechanical, both affected integration targets passed, and the full standard suite is green.

### D144: I19i splits shared session CLI helpers

The next integration-test decomposition slice moved shared `tests/session_cli.rs` fixture and fake-tool helpers to `tests/support/session_cli.rs`. The support module now owns agent-browser mock command wrapping, Chrome import state fixtures, NYTimes/provider-only state fixtures, JSON envelope helpers, reusable session model fixtures, local cmux loopback HTTP fixtures, and small wrappers for the shared mock backend/cmux commands. `tests/session_cli.rs` now imports these helpers through the support module, which prepares the file for behavior-focused splits without duplicating command or session fixture setup.

Validation:

- `cargo test --test session_cli`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the session CLI helper split. The helper move is mechanical, the affected integration target passed, and the full standard suite is green.

### D145: I19i splits session import CLI tests

The next session CLI decomposition slice moved the cmux import, Chrome import, owned Chrome import error-classification, and ignored real-cmux replay tests from `tests/session_cli.rs` into `tests/session_cli/imports.rs`. The root `tests/session_cli.rs` now declares that module with an explicit path, keeping the behavior-focused import tests out of the root file without making the submodule an independent Cargo integration target.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test --test session_cli`
- `git diff --check`
- `cargo test`

Confidence: High for the import-test split. The move is mechanical, the affected integration target passed, and the full standard suite is green.

### D146: I19i splits session login CLI tests

The next session CLI decomposition slice moved login start, finish, cancel, helper API, and ignored real-login smoke coverage from `tests/session_cli.rs` into `tests/session_cli/login.rs`. The root `tests/session_cli.rs` now declares separate import and login behavior modules and is reduced to list, compose, and inspect coverage.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test --test session_cli`
- `git diff --check`
- `cargo test`

Confidence: High for the login-test split. The move is mechanical, the affected integration target passed, and the full standard suite is green.

### D147: I19i splits shared get CLI helpers

The next get CLI decomposition slice moved shared fake backend, fake agent-browser, success-envelope, metadata discovery, loopback cookie echo server, and saved-session fixture helpers from `tests/get_cli.rs` into `tests/support/get_cli.rs`. `tests/get_cli.rs` now imports those helpers through the shared integration-test support module, preparing later behavior-focused get-test splits without duplicating backend command or session fixture setup.

Validation:

- `cargo fmt`
- `cargo test --test get_cli`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the get CLI helper split. The helper move is mechanical, the affected integration target passed, and the full standard suite is green.

### D148: I19i splits session-backed get CLI tests

The next get CLI decomposition slice moved session-backed replay, replay-scope rejection, repeated-session composition, provider/app cookie flow, session sensitivity, primary-backend failure redaction, agent-browser fallback, unauthenticated fallback suppression, fallback close-failure preservation, and ignored real Crawl4AI session replay coverage from `tests/get_cli.rs` into `tests/get_cli/session.rs`. The root `tests/get_cli.rs` now declares that behavior module with an explicit path and keeps non-session output, option, timeout, and backend-failure tests in the root file.

Validation:

- `cargo fmt`
- `cargo test --test get_cli`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the session-backed get-test split. The move is mechanical, the affected integration target passed, and the full standard suite is green.

### D149: I19i splits owned mock-site extractor parity tests

The next mock-site CLI decomposition slice moved the long owned static extractor parity test from `tests/mock_site_cli.rs` into `tests/mock_site_cli/owned.rs`. The root mock-site CLI test target now declares the owned behavior module with an explicit path and keeps command-backed redirect/output-shaping/default-backend/content-format parity smokes in the root file.

Validation:

- `cargo fmt`
- `cargo test --test mock_site_cli`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High for the owned mock-site split. The move is mechanical, the affected integration target passed, and the full standard suite is green.

### D150: I19i completed oversized module split

I19i is complete. The original production bottlenecks are now decomposed into behavior-owned extraction and CDP modules, and the largest integration-test bottlenecks are split into shared support helpers plus behavior modules. The current largest files in the touched surface are around 600-750 lines, instead of the original 2k-3k line extraction/CDP/test files. The remaining medium-sized files are coherent enough for follow-up feature work and do not need more physical splitting before moving to the next workpad task.

Final size evidence:

- `src/browser_cdp/client.rs`: 753 lines
- `src/extraction/mod.rs`: 733 lines
- `src/extraction/owned.rs`: 669 lines
- `tests/mock_site_cli/owned.rs`: 725 lines
- `tests/get_cli.rs`: 709 lines
- `tests/get_cli/session.rs`: 599 lines
- `tests/session_cli/login.rs`: 734 lines
- `tests/session_cli/imports.rs`: 652 lines
- `tests/session_cli.rs`: 417 lines

Validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Confidence: High. Each split was committed separately after focused validation plus the full standard gate, and no intentional behavior changes were introduced.

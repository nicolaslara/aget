## Browser Automation

| Topic | URL | Notes |
| --- | --- | --- |
| Playwright persistent context | https://playwright.dev/docs/api/class-browsertype#browser-type-launch-persistent-context | Evaluate dedicated profile login reuse. |
| Playwright BrowserContext API | https://playwright.dev/docs/api/class-browsercontext | Browser contexts are isolated; non-persistent contexts do not write browsing data to disk. `storageState()` captures cookies, localStorage, and IndexedDB state, making it useful for scoped auth-state export/import rather than full-profile reuse. |
| Playwright authentication docs | https://playwright.dev/docs/auth | Recommends authenticating once and reusing saved storage state when tests do not mutate shared server-side state. Notes Playwright does not provide an API to persist `sessionStorage`, which matters for sites whose auth depends on session storage. |
| Playwright persistent context API | https://playwright.dev/docs/api/class-browsertype | `launchPersistentContext(userDataDir)` uses on-disk browser storage and returns the only context; Playwright warns browsers do not allow multiple instances sharing one user data directory and recommends a separate empty directory instead of the default browser profile. |
| Chrome DevTools Protocol | https://chromedevtools.github.io/devtools-protocol/ | Evaluate current-browser/current-tab extraction. |
| Chrome remote debugging port hardening | https://developer.chrome.com/blog/remote-debugging-port | Chrome 136+ ignores `--remote-debugging-port` and `--remote-debugging-pipe` for the default Chrome data directory; remote debugging must use a non-standard `--user-data-dir`, reinforcing dedicated-profile automation. |
| Chrome debugger extension API | https://developer.chrome.com/docs/extensions/reference/debugger | Chrome's debugger API is an alternate transport for the Chrome DevTools Protocol and requires version-compatible attachment, so direct CDP attach is browser/version sensitive. |
| Chromium DevTools HTTP handler | https://chromium.googlesource.com/chromium/src/+/HEAD/content/browser/devtools/devtools_http_handler.cc | Chromium's remote debugging HTTP handler exposes the DevTools protocol over WebSocket when enabled. Treat the debugging endpoint as a broad live-browser control/data surface, especially for authenticated pages. |
| WebDriver BiDi | https://w3c.github.io/webdriver-bidi/ | Evaluate future browser automation standard. |
| WebDriver Classic spec | https://w3c.github.io/webdriver/ | WebDriver sessions are remote-end sessions with standardized capabilities, but persistent profile/login reuse is not a portable standard behavior; the spec advises new profiles per session for privacy. |
| Selenium Chrome browser options | https://www.selenium.dev/documentation/webdriver/browsers/chrome/ | Chrome can be launched with `--user-data-dir=...` through browser-specific options, but this is profile-directory reuse with the same locking/version/privacy concerns as manual Chrome profile use. |
| Selenium Firefox browser options | https://www.selenium.dev/documentation/webdriver/browsers/firefox/ | Firefox can be launched with `-profile /path/to/profile` through browser-specific options; profile reuse is browser-specific rather than a cross-browser WebDriver persistence model. |
| Chromium user data directories | https://chromium.googlesource.com/chromium/src/+/HEAD/docs/user_data_dir.md | Primary source for Chrome/Chromium user data/profile paths, profile subdirectories, user cache directories, and `--user-data-dir` override behavior. Notes that two running Chrome instances cannot share the same user data directory in Chrome Remote Desktop context. |
| Chrome profile version compatibility | https://chromium.org/administrators/common-problems-and-solutions | Chrome profiles are not backwards-compatible across mismatched Chrome versions; roaming/network-profile use across versions can cause crashes or data loss. |
| Firefox profile service | https://firefox-source-docs.mozilla.org/toolkit/profile/ | Gecko profiles store persistent data; profile root/local directories may be split from caches; profiles are locked with OS file locks and a second instance using the same profile fails with profile-in-use. |
| Firefox profile lock support note | https://support.mozilla.org/en-US/kb/firefox-already-running-not-responding | User-facing evidence that Firefox requires an unlocked profile and can leave lock files after abnormal shutdown; relevant to profile-reuse UX and failure messaging. |
| Selenium CDP caveats | https://www.selenium.dev/documentation/webdriver/bidi/cdp/ | CDP is not designed for testing, has no stable API, and behavior is browser-version-dependent; Selenium frames WebDriver BiDi as the standards-based replacement. |
| WebDriver BiDi spec | https://www.w3.org/TR/webdriver-bidi/ | W3C specification for bidirectional remote control of user agents over WebSocket, extending WebDriver sessions with event streaming and browser/session modules. |
| OWASP Logging Cheat Sheet | https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html | Audit logs should exclude or mask session identifiers, access tokens, passwords, encryption keys, sensitive personal data, and other high-sensitivity data; use hashing/sanitization where correlation is needed. |
| NIST Privacy Framework | https://www.nist.gov/privacy-framework | Privacy risk should be managed across the data lifecycle from collection through disposal; the core includes selective collection/disclosure, data minimization, provenance, audit/log records, and deletion/disposition outcomes. |

## Rust Candidates

| Need | Candidate | Notes |
| --- | --- | --- |
| HTTP | `reqwest` | Mature async HTTP client. |
| HTTP | `ureq` | `https://crates.io/crates/ureq`, docs `https://docs.rs/ureq/3.3.0`. I19d uses v3.3.0 for the owned static extractor because it is blocking, HTTPS-capable, supports redirects and global timeouts, and fits the current synchronous extraction boundary. License: MIT OR Apache-2.0. |
| CLI | `clap` | Standard Rust CLI framework. |
| HTML parse | `scraper`, `html5ever`, `kuchiki` | Need quality comparison. |
| HTML parse/select | `scraper` | `https://crates.io/crates/scraper`, docs `https://docs.rs/scraper/0.27.0`. I19d uses v0.27.0 for browser-grade HTML parsing and CSS selector matching in the owned static extractor. License: ISC. |
| HTML parse support | `html5ever` | `https://crates.io/crates/html5ever`, docs `https://docs.rs/html5ever/0.39.0`. Added as a direct dependency only to access the `TreeSink` trait needed to remove excluded elements from `scraper`'s parsed document tree. License: MIT OR Apache-2.0. |
| DOM traversal | `ego-tree` | `https://crates.io/crates/ego-tree`, docs `https://docs.rs/ego-tree/0.11.0`. Added as a direct dependency because `scraper` stores parsed DOM nodes in `ego-tree`, and the first owned markdown renderer needs explicit child-node traversal. License: ISC. |
| HTML to markdown | `html2md` | Rejected for this project after `cargo info html2md` reported GPL-3.0+. The first owned markdown slice uses a small local renderer instead. |
| URL parsing/joining | `url` | `https://crates.io/crates/url`, docs `https://docs.rs/url/2.5.8`. I19d uses v2.5.8 so owned markdown resolves relative links and images against the final URL or HTML `<base href>`. License: MIT OR Apache-2.0. |
| Browser/CDP WebSocket | `tungstenite` | `https://crates.io/crates/tungstenite`, docs `https://docs.rs/tungstenite/0.29.0`. I19e uses the blocking WebSocket client for a minimal local Chrome DevTools Protocol renderer in the AgetBrowser fallback. License: MIT OR Apache-2.0. |
| Browser/CDP | `chromiumoxide`, `fantoccini` | Need maintenance/reliability research. |
| Token estimate | `tiktoken-rs` | Need model compatibility check. |
| Cache | SQLite/`rusqlite`, filesystem | Need schema design. |


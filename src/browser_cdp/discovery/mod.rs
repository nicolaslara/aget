mod active_port;
mod diagnostics;
mod endpoint;
mod profile;

pub(super) use active_port::{read_devtools_active_port, wait_for_devtools_active_port};
pub(super) use diagnostics::classify_chrome_startup_error;
use diagnostics::devtools_ws_url_from_chrome_stderr;
#[cfg(test)]
pub(super) use diagnostics::{devtools_ws_url_from_stderr, relevant_chrome_stderr};
pub(crate) use endpoint::discover_cdp_ws_url;
#[cfg(test)]
pub(super) use endpoint::rewrite_cdp_ws_host;
pub(super) use profile::{connect_existing_profile_browser, wait_for_profile_browser_shutdown};

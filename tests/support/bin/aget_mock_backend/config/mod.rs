mod expectations;
mod options;
mod state;

use std::env;
use std::fs;

use serde_json::Value;

pub(crate) use self::expectations::{assert_expected_args, assert_expected_environment};
pub(crate) use self::state::{assert_expected_state, expand_state_placeholders, read_state};

pub(crate) fn read_config() -> Result<Value, String> {
    let config_path = env::current_exe()
        .map_err(|error| format!("locate mock backend executable: {error}"))?
        .with_extension("json");
    if !config_path.exists() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(
        &fs::read_to_string(&config_path)
            .map_err(|error| format!("read mock backend config: {error}"))?,
    )
    .map_err(|error| format!("parse mock backend config: {error}"))
}

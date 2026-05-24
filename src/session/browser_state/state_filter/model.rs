use serde::Deserialize;

use crate::session::StorageEntry;

#[derive(Debug, Deserialize)]
pub(crate) struct BrowserState {
    #[serde(default)]
    pub(crate) cookies: Vec<BrowserCookie>,
    #[serde(default)]
    pub(crate) origins: Vec<BrowserOrigin>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BrowserCookie {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) domain: String,
    pub(crate) path: String,
    #[serde(default, deserialize_with = "deserialize_browser_state_expires")]
    pub(crate) expires: Option<i64>,
    #[serde(rename = "httpOnly", default)]
    pub(crate) http_only: bool,
    #[serde(default)]
    pub(crate) secure: bool,
    #[serde(rename = "sameSite", default)]
    pub(crate) same_site: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BrowserOrigin {
    pub(crate) origin: String,
    #[serde(rename = "localStorage", default)]
    pub(crate) local_storage: Vec<StorageEntry>,
    #[serde(rename = "sessionStorage", default)]
    pub(crate) session_storage: Vec<StorageEntry>,
}

fn deserialize_browser_state_expires<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let expires = Option::<serde_json::Number>::deserialize(deserializer)?;
    expires
        .map(|number| {
            number
                .as_i64()
                .or_else(|| number.as_f64().map(|value| value.trunc() as i64))
                .ok_or_else(|| Error::custom("browser state expires must be numeric"))
        })
        .transpose()
}

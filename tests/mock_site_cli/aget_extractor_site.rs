#[path = "aget_extractor_site/cleanup.rs"]
mod cleanup;
#[path = "aget_extractor_site/formats_options.rs"]
mod formats_options;
#[path = "aget_extractor_site/main_content.rs"]
mod main_content;
#[path = "aget_extractor_site/markdown.rs"]
mod markdown;
#[path = "aget_extractor_site/selectors.rs"]
mod selectors;

use crate::support::mock_site::MockSite;

pub(crate) fn aget_extractor_parity_site() -> MockSite {
    let builder = crate::support::mock_site::MockSite::builder();
    let builder = formats_options::routes(builder);
    let builder = main_content::routes(builder);
    let builder = selectors::routes(builder);
    let builder = markdown::routes(builder);
    cleanup::routes(builder).start()
}

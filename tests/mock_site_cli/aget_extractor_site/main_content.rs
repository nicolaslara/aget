#[path = "main_content/basic.rs"]
mod basic;
#[path = "main_content/density.rs"]
mod density;
#[path = "main_content/noise_threshold.rs"]
mod noise_threshold;

use crate::support::mock_site::MockSiteBuilder;

pub(crate) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    let builder = basic::routes(builder);
    let builder = density::routes(builder);
    noise_threshold::routes(builder)
}

#[path = "markdown/basic.rs"]
mod basic;
#[path = "markdown/inline.rs"]
mod inline;
#[path = "markdown/links_code_lists.rs"]
mod links_code_lists;
#[path = "markdown/wrapping_preserve.rs"]
mod wrapping_preserve;

use crate::support::mock_site::MockSiteBuilder;

pub(crate) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    let builder = basic::routes(builder);
    let builder = inline::routes(builder);
    let builder = wrapping_preserve::routes(builder);
    links_code_lists::routes(builder)
}

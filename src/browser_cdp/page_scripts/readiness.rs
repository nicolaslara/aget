use std::time::Duration;

use crate::error::AgetError;

use super::super::io_aget_error;

pub(in crate::browser_cdp) fn selector_exists_expression(
    selector: &str,
) -> Result<String, AgetError> {
    let selector = selector
        .trim()
        .strip_prefix("css:")
        .unwrap_or(selector)
        .trim();
    Ok(format!(
        "document.querySelector({}) !== null",
        serde_json::to_string(selector).map_err(io_aget_error)?
    ))
}

pub(in crate::browser_cdp) fn full_page_scan_expression(
    scroll_delay: Duration,
    max_scroll_steps: usize,
) -> String {
    let delay_ms = scroll_delay.as_millis();
    format!(
        r#"(async () => {{
        const delayMs = {delay_ms};
        const maxSteps = {max_scroll_steps};
        const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
        const viewportHeight = () => Math.max(
            window.innerHeight || 0,
            document.documentElement?.clientHeight || 0,
            1
        );
        const pageHeight = () => Math.max(
            document.documentElement?.scrollHeight || 0,
            document.body?.scrollHeight || 0,
            viewportHeight()
        );

        let currentPosition = viewportHeight();
        window.scrollTo(0, currentPosition);
        await sleep(delayMs);

        let totalHeight = pageHeight();
        let steps = 0;
        while (currentPosition < totalHeight && steps < maxSteps) {{
            currentPosition = Math.min(currentPosition + viewportHeight(), totalHeight);
            window.scrollTo(0, currentPosition);
            await sleep(delayMs);
            steps += 1;

            const newHeight = pageHeight();
            if (newHeight > totalHeight) {{
                totalHeight = newHeight;
            }}
        }}

        window.scrollTo(0, 0);
        await sleep(delayMs);
        window.scrollTo(0, totalHeight);
        return {{ steps, totalHeight }};
    }})()"#
    )
}

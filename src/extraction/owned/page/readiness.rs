use scraper::Html;

use super::super::super::html_clean::parse_css_selector;

pub(super) fn should_render_scripted_response(body: &str) -> bool {
    let document = Html::parse_document(body);
    let Ok(selector) = parse_css_selector("script") else {
        return false;
    };
    document
        .select(&selector)
        .any(|script| is_executable_script_type(script.attr("type")))
}

fn is_executable_script_type(script_type: Option<&str>) -> bool {
    let Some(script_type) = script_type else {
        return true;
    };
    let script_type = script_type
        .split(';')
        .next()
        .unwrap_or(script_type)
        .trim()
        .to_ascii_lowercase();
    if script_type.is_empty() {
        return true;
    }
    matches!(
        script_type.as_str(),
        "module"
            | "text/javascript"
            | "application/javascript"
            | "text/ecmascript"
            | "application/ecmascript"
            | "text/jscript"
    )
}

#[cfg(test)]
mod tests {
    use super::should_render_scripted_response;

    #[test]
    fn script_detection_covers_common_executable_javascript_types() {
        for script in [
            r#"<script>window.ready = true</script>"#,
            r#"<script src="/app.js"></script>"#,
            r#"<script type="">window.ready = true</script>"#,
            r#"<script type="module">window.ready = true</script>"#,
            r#"<script type="text/javascript">window.ready = true</script>"#,
            r#"<script type="text/javascript; charset=utf-8">window.ready = true</script>"#,
            r#"<script type="application/javascript">window.ready = true</script>"#,
            r#"<script type="text/ecmascript">window.ready = true</script>"#,
            r#"<script type="application/ecmascript">window.ready = true</script>"#,
        ] {
            assert!(
                should_render_scripted_response(&format!("<html><body>{script}</body></html>")),
                "expected executable script detection for {script}"
            );
        }
    }

    #[test]
    fn script_detection_ignores_non_executable_data_script_types() {
        for script in [
            r#"<script type="application/ld+json">{"name":"Docs"}</script>"#,
            r#"<script type="application/json">{"name":"Docs"}</script>"#,
            r#"<script type="application/json" src="/data.json"></script>"#,
            r#"<script type="importmap">{"imports":{}}</script>"#,
            r#"<script type="speculationrules">{"prerender":[]}</script>"#,
        ] {
            assert!(
                !should_render_scripted_response(&format!("<html><body>{script}</body></html>")),
                "expected static extraction for {script}"
            );
        }
    }
}

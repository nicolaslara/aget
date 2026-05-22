use std::path::Path;

use aget::{Aget, AgetExtractorBackend, OutputFormat};

pub(super) fn assert_local_content_urls(aget_home: &Path) {
    let raw = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(
            "raw:<html><body><main><h1>Raw Local</h1><p>Use #anchor.</p><script>window.ready = true;</script></main></body></html>",
        )
        .content_format(OutputFormat::Markdown)
        .selector("main")
        .run()
        .unwrap();
    assert_eq!(raw.content, "# Raw Local\n\nUse #anchor.");
    assert_eq!(
        raw.final_url,
        "raw:<html><body><main><h1>Raw Local</h1><p>Use #anchor.</p><script>window.ready = true;</script></main></body></html>"
    );

    let temp = tempfile::tempdir().unwrap();
    let html_path = temp.path().join("page.html");
    std::fs::write(
        &html_path,
        r#"<html><head><base href="https://docs.example/base/"></head><body><main><h1>File Local</h1><p>Read <a href="guide.html">Guide</a>.</p></main></body></html>"#,
    )
    .unwrap();
    let file_url = format!("file://{}", html_path.display());

    let file = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(&file_url)
        .content_format(OutputFormat::Markdown)
        .selector("main")
        .run()
        .unwrap();
    assert_eq!(
        file.content,
        "# File Local\n\nRead [Guide](https://docs.example/base/guide.html)."
    );
    assert_eq!(file.final_url, file_url);
}

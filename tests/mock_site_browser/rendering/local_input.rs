use aget::{Aget, AgetExtractorBackend, OutputFormat};

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_processes_raw_content_in_browser_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(
            r##"raw:<html><body><main><h1>Raw Browser</h1><div id="client-result">Loading</div><script>document.querySelector("#client-result").innerHTML = "<p>Rendered Raw</p>"</script></main></body></html>"##,
        )
        .content_format(OutputFormat::Text)
        .backend_option("aget.process_in_browser", "true")
        .wait_for_selector("#client-result p")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Raw Browser\nRendered Raw");
    assert!(extraction.final_url.starts_with("raw:<html>"));
}

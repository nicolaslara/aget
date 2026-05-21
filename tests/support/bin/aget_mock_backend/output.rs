use serde_json::Value;

pub(crate) fn print_backend_result(
    ok: bool,
    final_url: Option<String>,
    content: Option<String>,
    warnings: Vec<Value>,
    error: Option<String>,
) {
    println!(
        "{}",
        serde_json::json!({
            "ok": ok,
            "final_url": final_url,
            "content": content,
            "warnings": warnings,
            "error": error,
        })
    );
}

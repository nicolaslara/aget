use std::collections::BTreeMap;

use super::protocol::{has_cookie, html};
use super::{MockRequest, MockResponse};

pub(super) fn response_for(
    request: &MockRequest,
    routes: &BTreeMap<String, MockResponse>,
) -> String {
    if let Some(response) = routes.get(&request.path) {
        return response.to_http();
    }

    match request.path.as_str() {
        "/public" => html(
            200,
            &[],
            r#"
<html>
  <body>
    <main id="content">
      <nav>In-Article Navigation</nav>
      <h1>Public Main</h1>
      <p>Visible public article.</p>
    </main>
    <footer>Footer chrome</footer>
  </body>
</html>
"#,
        ),
        "/warning" => html(
            200,
            &[],
            r#"<html><body><main><h1>Warning Page</h1><p>Visible warning body.</p></main></body></html>"#,
        ),
        "/protected" if has_cookie(request, "app_session", "valid-app") => html(
            200,
            &[],
            r#"<html><body><main><h1>Protected Account</h1><p>Private account body.</p></main></body></html>"#,
        ),
        "/protected" if has_cookie(request, "app_session", "expired") => html(
            401,
            &[],
            r#"<html><body><main><h1>Session Expired</h1></main></body></html>"#,
        ),
        "/protected" => html(302, &[("Location", "/login?next=/protected")], ""),
        "/requires-two"
            if has_cookie(request, "app_session", "valid-app")
                && has_cookie(request, "provider_session", "valid-provider") =>
        {
            html(
                200,
                &[],
                r#"<html><body><main><h1>Composed Session</h1><p>Both cookies arrived.</p></main></body></html>"#,
            )
        }
        "/requires-two" => html(401, &[], r#"<main><h1>Missing composed auth</h1></main>"#),
        "/storage-protected" => html(
            200,
            &[],
            r#"
<html>
  <body>
    <main><h1>Storage App Shell</h1><div id="storage-result"></div></main>
    <script>
      fetch("/storage-api", { headers: { "X-Local-Token": localStorage.getItem("local_token") }})
    </script>
  </body>
</html>
"#,
        ),
        "/storage-api"
            if request
                .headers
                .get("x-local-token")
                .is_some_and(|token| token == "storage-secret") =>
        {
            html(
                200,
                &[],
                r#"<html><body><main><h1>Storage Protected</h1></main></body></html>"#,
            )
        }
        "/storage-api" => html(401, &[], r#"<main><h1>Missing storage token</h1></main>"#),
        "/delayed" => html(
            200,
            &[],
            r#"
<html>
  <body>
    <main><h1>Delayed Shell</h1><div id="initial">Loading</div></main>
    <script>
      setTimeout(() => {
        const ready = document.createElement("div")
        ready.id = "ready"
        ready.textContent = "Delayed Ready"
        document.body.appendChild(ready)
      }, 25)
    </script>
  </body>
</html>
"#,
        ),
        "/redirect" => html(302, &[("Location", "/public")], ""),
        "/login" => html(
            200,
            &[],
            r#"<html><body><main><h1>Login Form</h1><form action="/login/callback"></form></main></body></html>"#,
        ),
        "/login/callback" => html(
            200,
            &[("Set-Cookie", "app_session=valid-app; Path=/; HttpOnly")],
            r#"<html><body><main><h1>Login Complete</h1></main></body></html>"#,
        ),
        "/logout" => html(
            200,
            &[("Set-Cookie", "app_session=expired; Path=/; Max-Age=0")],
            r#"<html><body><main><h1>Logged Out</h1></main></body></html>"#,
        ),
        _ => MockResponse::html(r#"<main><h1>Not Found</h1></main>"#)
            .status(404)
            .to_http(),
    }
}

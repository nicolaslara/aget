pub(in crate::browser_cdp) fn iframe_process_expression() -> &'static str {
    r#"(async () => {
        const iframes = Array.from(document.querySelectorAll("iframe"));
        let replaced = 0;
        let inaccessible = 0;

        const waitForLoad = (iframe) => new Promise((resolve) => {
            const timeout = setTimeout(resolve, 30000);
            iframe.addEventListener("load", () => {
                clearTimeout(timeout);
                resolve();
            }, { once: true });
        });

        for (let index = 0; index < iframes.length; index += 1) {
            const iframe = iframes[index];
            try {
                iframe.id = iframe.id || `iframe-${index}`;
                if (!iframe.contentDocument || !iframe.contentDocument.body) {
                    inaccessible += 1;
                    continue;
                }
                if (iframe.contentDocument.readyState !== "complete") {
                    await waitForLoad(iframe);
                }
                const doc = iframe.contentDocument;
                if (!doc || !doc.body) {
                    inaccessible += 1;
                    continue;
                }
                const parser = new DOMParser();
                const parsed = parser.parseFromString(doc.body.innerHTML || "", "text/html");
                const replacement = document.createElement("div");
                replacement.className = `extracted-iframe-content-${index}`;
                while (parsed.body.firstChild) {
                    replacement.appendChild(parsed.body.firstChild);
                }
                iframe.replaceWith(replacement);
                replaced += 1;
            } catch (_) {
                inaccessible += 1;
            }
        }

        return JSON.stringify({ total: iframes.length, replaced, inaccessible });
    })()"#
}

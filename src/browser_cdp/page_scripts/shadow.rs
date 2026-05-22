pub(in crate::browser_cdp) fn shadow_dom_attach_override_expression() -> &'static str {
    r#"(() => {
        if (Element.prototype.__agetOpenShadowRoots) {
            return;
        }
        const originalAttachShadow = Element.prototype.attachShadow;
        Object.defineProperty(Element.prototype, "__agetOpenShadowRoots", { value: true });
        Element.prototype.attachShadow = function(init) {
            return originalAttachShadow.call(this, { ...init, mode: "open" });
        };
    })()"#
}

pub(in crate::browser_cdp) fn shadow_dom_flatten_expression() -> &'static str {
    r#"(() => {
        const voidTags = new Set([
            "area", "base", "br", "col", "embed", "hr", "img", "input",
            "link", "meta", "param", "source", "track", "wbr"
        ]);

        const escapeAttr = (value) => String(value)
            .replace(/&/g, "&amp;")
            .replace(/"/g, "&quot;");

        const attrs = (node) => {
            let output = "";
            for (const attr of Array.from(node.attributes || [])) {
                output += ` ${attr.name}="${escapeAttr(attr.value)}"`;
            }
            return output;
        };

        const serialize = (node) => {
            if (node.nodeType === Node.TEXT_NODE) {
                return node.textContent || "";
            }
            if (node.nodeType === Node.COMMENT_NODE || node.nodeType !== Node.ELEMENT_NODE) {
                return "";
            }
            const tag = node.tagName.toLowerCase();
            const content = node.shadowRoot
                ? serializeShadowRoot(node)
                : Array.from(node.childNodes).map(serialize).join("");
            if (voidTags.has(tag)) {
                return `<${tag}${attrs(node)}>`;
            }
            return `<${tag}${attrs(node)}>${content}</${tag}>`;
        };

        const serializeShadowRoot = (host) =>
            Array.from(host.shadowRoot.childNodes)
                .map((child) => serializeShadowChild(child, host))
                .join("");

        const serializeShadowChild = (node, host) => {
            if (node.nodeType === Node.TEXT_NODE) {
                return node.textContent || "";
            }
            if (node.nodeType === Node.COMMENT_NODE || node.nodeType !== Node.ELEMENT_NODE) {
                return "";
            }
            const tag = node.tagName.toLowerCase();
            if (tag === "style") {
                return "";
            }
            if (tag === "slot") {
                const assigned = node.assignedNodes({ flatten: true });
                const source = assigned.length > 0 ? assigned : Array.from(node.childNodes);
                return Array.from(source)
                    .map((child) => assigned.length > 0 ? serialize(child) : serializeShadowChild(child, host))
                    .join("");
            }
            const content = node.shadowRoot
                ? serializeShadowRoot(node)
                : Array.from(node.childNodes).map((child) => serializeShadowChild(child, host)).join("");
            if (voidTags.has(tag)) {
                return `<${tag}${attrs(node)}>`;
            }
            return `<${tag}${attrs(node)}>${content}</${tag}>`;
        };

        return serialize(document.documentElement);
    })()"#
}

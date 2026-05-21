use crate::error::AgetError;

use super::io_aget_error;

pub(super) fn origin_storage_expression() -> &'static str {
    r#"(() => {
        const result = { origin: location.origin, localStorage: [], sessionStorage: [] };
        try {
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                result.localStorage.push({ name: key, value: localStorage.getItem(key) });
            }
        } catch(e) {}
        try {
            for (let i = 0; i < sessionStorage.length; i++) {
                const key = sessionStorage.key(i);
                result.sessionStorage.push({ name: key, value: sessionStorage.getItem(key) });
            }
        } catch(e) {}
        return result;
    })()"#
}

pub(super) fn local_storage_set_expression(name: &str, value: &str) -> Result<String, AgetError> {
    Ok(format!(
        "localStorage.setItem({}, {})",
        serde_json::to_string(name).map_err(io_aget_error)?,
        serde_json::to_string(value).map_err(io_aget_error)?
    ))
}

pub(super) fn session_storage_set_expression(name: &str, value: &str) -> Result<String, AgetError> {
    Ok(format!(
        "sessionStorage.setItem({}, {})",
        serde_json::to_string(name).map_err(io_aget_error)?,
        serde_json::to_string(value).map_err(io_aget_error)?
    ))
}

pub(super) fn selector_exists_expression(selector: &str) -> Result<String, AgetError> {
    Ok(format!(
        "document.querySelector({}) !== null",
        serde_json::to_string(selector).map_err(io_aget_error)?
    ))
}

pub(super) fn rendered_overlay_cleanup_expression() -> &'static str {
    r#"(async () => {
        if (!document.body) {
            return 0;
        }

        const isVisible = (elem) => {
            const style = window.getComputedStyle(elem);
            return style.display !== "none" && style.visibility !== "hidden" && style.opacity !== "0";
        };
        const commonSelectors = [
            'button[class*="close" i]',
            'button[class*="dismiss" i]',
            'button[aria-label*="close" i]',
            'button[title*="close" i]',
            'a[class*="close" i]',
            'span[class*="close" i]',
            '[class*="cookie-banner" i]',
            '[id*="cookie-banner" i]',
            '[class*="cookie-consent" i]',
            '[id*="cookie-consent" i]',
            '[class*="newsletter" i]',
            '[class*="subscribe" i]',
            '[class*="popup" i]',
            '[class*="modal" i]',
            '[class*="overlay" i]',
            '[class*="dialog" i]',
            '[role="dialog"]',
            '[role="alertdialog"]',
        ];

        let removed = 0;
        const removeElement = (elem) => {
            if (elem && elem.parentNode) {
                elem.remove();
                removed += 1;
            }
        };

        for (const selector of commonSelectors.slice(0, 6)) {
            for (const button of Array.from(document.querySelectorAll(selector))) {
                if (!isVisible(button)) {
                    continue;
                }
                try {
                    button.click();
                    await new Promise((resolve) => setTimeout(resolve, 100));
                } catch (_) {}
            }
        }

        for (const elem of Array.from(document.querySelectorAll("*"))) {
            if (elem === document.documentElement || elem === document.body || !isVisible(elem)) {
                continue;
            }
            const style = window.getComputedStyle(elem);
            const zIndex = Number.parseInt(style.zIndex, 10);
            const hasHighZIndex = Number.isFinite(zIndex) && zIndex > 999;
            const positioned = hasHighZIndex || style.position === "fixed" || style.position === "absolute";
            const largeOrOverlayLike =
                elem.offsetWidth > window.innerWidth * 0.5 ||
                elem.offsetHeight > window.innerHeight * 0.5 ||
                style.backgroundColor.includes("rgba") ||
                Number.parseFloat(style.opacity) < 1;
            if (positioned && largeOrOverlayLike) {
                removeElement(elem);
            }
        }

        for (const selector of commonSelectors) {
            for (const elem of Array.from(document.querySelectorAll(selector))) {
                if (isVisible(elem)) {
                    removeElement(elem);
                }
            }
        }

        for (const elem of Array.from(document.querySelectorAll("*"))) {
            if (elem === document.documentElement || elem === document.body || !isVisible(elem)) {
                continue;
            }
            const position = window.getComputedStyle(elem).position;
            if (position === "fixed" || position === "sticky") {
                removeElement(elem);
            }
        }

        document.body.style.marginRight = "0px";
        document.body.style.paddingRight = "0px";
        document.body.style.overflow = "auto";
        document.body.scrollIntoView(false);
        await new Promise((resolve) => setTimeout(resolve, 50));
        return removed;
    })()"#
}

pub(super) fn shadow_dom_attach_override_expression() -> &'static str {
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

pub(super) fn shadow_dom_flatten_expression() -> &'static str {
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

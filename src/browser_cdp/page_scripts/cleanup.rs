pub(in crate::browser_cdp) fn rendered_overlay_cleanup_expression() -> &'static str {
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

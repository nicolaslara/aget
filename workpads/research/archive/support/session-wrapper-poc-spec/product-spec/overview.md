# Session Wrapper Product Spec: Overview

## Summary

`aget` is a local-first session and extraction tool for agents. Given a URL, visible browser surface, or saved session, it produces clean agent-ready markdown while keeping authenticated content and credential-equivalent session material local by default.

The PoC should be a thin wrapper over existing tools:

- `agent-browser` for Chrome profile snapshotting and browser state export.
- Crawl4AI for browser-rendered markdown extraction from Playwright-compatible state.
- cmux as an optional integration for users already browsing in cmux panes.

The product value is not the extraction backend itself. The product value is safe, explicit, composable local session management for agent web access.

## Problem

Hosted URL-to-markdown tools are excellent for public pages but are the wrong default for authenticated/private content. Browser automation tools can interact with logged-in pages but usually expose raw browser state, lack markdown-quality output, or force awkward profile/login workflows.

Users need a local tool that can:

- Start with no ambient auth by default.
- Create independent named sessions.
- Import only approved session material from a browser or browser surface.
- Combine sessions intentionally for a request.
- Fetch authenticated pages locally as clean markdown.
- Avoid sending cookies, local storage, private HTML, screenshots, or extracted content to hosted services.

## Goals

- Provide an empty-session default: `aget get <url>` should use no saved cookies unless explicitly requested.
- Provide `aget <url>` as a shorthand alias for `aget get <url>`.
- Support independent named sessions: `google`, `facebook`, `hellointerview`, `github-work`, `github-personal`.
- Support combining sessions per request: `aget get <url> --session google --session hellointerview`.
- Support creating a new session from a combination: `aget session compose hi-oauth --session google --session hellointerview`.
- Support OAuth-style workflows where provider state is useful during login but can be excluded later.
- Support users with and without cmux.
- Support agent-safe non-interactive operation by default.
- Support bounded runtime with configurable timeouts.
- Allow passing extraction options through to the backend without baking every extractor feature into the top-level command model.
- Treat all session state as credential-equivalent bearer material.
- Keep v1 as a thin wrapper with a clear migration path to owned v2 internals.

## Non-Goals For PoC

- Do not implement a custom browser engine or crawler.
- Do not implement multi-step site action APIs such as search, add-to-cart, posting, checkout, or account mutation.
- Do not bypass paywalls, access controls, anti-bot systems, or site policy.
- Do not automate credential entry.
- Do not send authenticated content or session state to hosted extraction services.
- Do not depend on cmux as a required runtime.
- Do not require users to use their primary browser profile by default.

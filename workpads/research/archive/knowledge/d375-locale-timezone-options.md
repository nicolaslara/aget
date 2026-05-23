# D375: Crawl4AI Locale And Timezone Options

Decision: `AgetExtractor` supports explicit `crawl4ai.locale` and
`crawl4ai.timezone_id` options for owned CDP-rendered extraction.

Source evidence:

- `references/repos/crawl4ai/crawl4ai/async_configs.py` defines
  `CrawlerRunConfig.locale` and `CrawlerRunConfig.timezone_id` constructor
  fields, stores them on the run config, and includes both fields in `dump()`.
- `references/repos/crawl4ai/crawl4ai/browser_manager.py` applies provided
  values to Playwright browser context settings as `locale` and `timezone_id`.
- The same browser manager includes both fields in the browser-context
  signature, so changing them can require a distinct context.

Implementation boundary:

- The owned backend accepts `crawl4ai.locale` and `crawl4ai.timezone_id` as
  trimmed string options.
- CDP rendering applies them with `Emulation.setLocaleOverride` and
  `Emulation.setTimezoneOverride` after page-domain setup and before
  navigation/capture.
- Static HTTP extraction cannot observe locale/timezone, so otherwise-static
  requests route through owned browser rendering when either option is present.
- The compatibility Crawl4AI command helper forwards both options to real
  Crawl4AI when the installed config accepts them.

Non-goals for this slice:

- No geolocation, proxy, arbitrary-header, or random-user-agent support was
  added.
- Locale/timezone are explicit caller choices only; `aget` does not infer or
  randomize browser identity.

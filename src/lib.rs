mod browser_cdp;

pub mod aget;
pub mod aget_browser;
pub mod aget_extractor;
pub mod cli;
pub mod error;
pub mod extraction;
pub mod interact;
pub(crate) mod process;
pub mod session;

pub use aget::{
    Aget, AgetBrowserBackend, AuthorizationPredicateResult, AuthorizationState,
    AuthorizeSessionOptions, AuthorizeSessionResult, CurrentTabOptions, GetRequest,
};
pub use aget_browser::AgetBrowser;
pub use aget_extractor::AgetExtractor;
pub use cli::{
    ArtifactsCommand, ArtifactsSubcommand, AuthorizeSessionCommand, BatchCommand, BrowserChoice,
    CacheCommandOptions, CachePolicy, Cli, Command, ComposeSessionCommand, CrawlCommand,
    CurrentTabCommand, DebugCaptureCommandOptions, DeleteArtifactCommand, DeleteSessionCommand,
    DoctorCheck, DoctorCommand, EnvelopeFormat, ExtractCommand, ExtractOutput, ExtractorOption,
    GetCommand, GlobalOptions, ImportBrowserSessionCommand, ImportChromeSessionCommand,
    ImportCmuxSessionCommand, ImportSessionCommand, ImportSessionSource, InlineContent,
    InspectArtifactCommand, InspectSessionCommand, InteractCommand, LoginCancelCommand,
    LoginFinishCommand, LoginSessionCommand, LoginSessionSubcommand, LoginStartCommand, MapCommand,
    MapOutput, OutputFormat, PruneArtifactsCommand, SearchPageCommand, SearchPageOutput,
    SessionCommand, SessionSubcommand,
};
pub use error::{AgetError, ErrorCode, ErrorResponse, ENVELOPE_SCHEMA_VERSION};
pub use extraction::{
    get_url, AgetExtractorBackend, CacheMetadata, CacheStatus, DebugArtifact, DebugArtifacts,
    DebugCaptureOptions, GetOptions, GetSuccess, TimingMs, UsageMetrics,
};
pub use interact::{
    execute_browser_action_plan, parse_action_plan_json, redacted_action_request,
    validate_action_plan, write_redacted_action_request, ActionDefinition, ActionPlan,
    ActionPlanDefaults, ActionPlanError, ActionPlanValidationOptions, BrowserActionExecution,
    BrowserActionResult, BrowserActionRunOptions, BrowserActionRunSource, BrowserActionStatus,
    CaptureAction, ClickAction, ExtractAction, SelectAction, SelectorMatch, SubmitAction,
    TypeAction, WaitAction, WaitLoadState, ACTIONS_SCHEMA_VERSION,
};
pub use session::{
    cancel_login_session, complete_login_session, compose_session, finish_login_session,
    import_chrome_session, import_cmux_session, merge_login_session, start_login_session,
    ChromeImportOptions, CmuxImportOptions, LoginCancelOptions, LoginCancelResult,
    LoginCompleteOptions, LoginFinishOptions, LoginFinishResult, LoginStartOptions,
    LoginStartResult, Session, SessionCookie, SessionOrigin, SessionSource, SessionStore,
    StorageEntry,
};

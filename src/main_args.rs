use std::ffi::OsString;

pub(crate) fn command_name_from_args(args: &[OsString]) -> &'static str {
    let tokens = args
        .iter()
        .skip(1)
        .filter_map(|arg| arg.to_str())
        .collect::<Vec<_>>();

    if let Some(index) = tokens.iter().position(|token| *token == "session") {
        return match tokens.get(index + 1).copied() {
            Some("list") => "session.list",
            Some("authorize") => "session.authorize",
            Some("inspect") => "session.inspect",
            Some("delete") => "session.delete",
            Some("compose") => "session.compose",
            Some("import") => match tokens.get(index + 2).copied() {
                Some("cmux") => "session.import.cmux",
                Some("browser") => "session.import.browser",
                Some("chrome") => "session.import.chrome",
                _ => "session.import",
            },
            Some("login") => match tokens.get(index + 2).copied() {
                Some("start") => "session.login.start",
                Some("finish") => "session.login.finish",
                Some("cancel") => "session.login.cancel",
                _ => "session.login",
            },
            _ => "session",
        };
    }

    if tokens.contains(&"doctor") {
        return "doctor";
    }

    if let Some(index) = tokens.iter().position(|token| *token == "artifacts") {
        return match tokens.get(index + 1).copied() {
            Some("list") => "artifacts.list",
            Some("inspect") => "artifacts.inspect",
            Some("delete") => "artifacts.delete",
            Some("prune") => "artifacts.prune",
            _ => "artifacts",
        };
    }

    if tokens.contains(&"batch") {
        return "batch";
    }

    if tokens.contains(&"map") {
        return "map";
    }

    if tokens.contains(&"crawl") {
        return "crawl";
    }

    if tokens.contains(&"search-page") {
        return "search-page";
    }

    if tokens.contains(&"extract") {
        return "extract";
    }

    if tokens.contains(&"get")
        || tokens
            .iter()
            .any(|token| token.starts_with("http://") || token.starts_with("https://"))
    {
        return "get";
    }
    if tokens.contains(&"current-tab") {
        return "current-tab";
    }

    "cli"
}

pub(crate) fn args_request_json_envelope(args: &[OsString]) -> bool {
    let mut iter = args.iter().filter_map(|arg| arg.to_str());
    while let Some(arg) = iter.next() {
        if arg == "--envelope" {
            return matches!(iter.next(), Some("json"));
        }
        if arg == "--envelope=json" {
            return true;
        }
    }
    false
}

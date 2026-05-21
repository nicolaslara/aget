use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct Args {
    pub(crate) url: String,
    pub(crate) state: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) metadata: PathBuf,
    pub(crate) format: String,
    pub(crate) selector: Option<String>,
    pub(crate) exclude_selector: Option<String>,
    pub(crate) wait_for: Option<String>,
    pub(crate) extractor_options: Vec<String>,
}

pub(crate) fn parse_args(raw: Vec<String>) -> Result<Args, String> {
    let mut args = Args {
        format: "markdown".to_string(),
        ..Args::default()
    };
    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--url" => args.url = next_value(&mut iter, "--url")?,
            "--state" => args.state = PathBuf::from(next_value(&mut iter, "--state")?),
            "--output" => args.output = PathBuf::from(next_value(&mut iter, "--output")?),
            "--metadata" => args.metadata = PathBuf::from(next_value(&mut iter, "--metadata")?),
            "--format" => args.format = next_value(&mut iter, "--format")?,
            "--selector" => args.selector = Some(next_value(&mut iter, "--selector")?),
            "--exclude-selector" => {
                args.exclude_selector = Some(next_value(&mut iter, "--exclude-selector")?)
            }
            "--wait-for" => args.wait_for = Some(next_value(&mut iter, "--wait-for")?),
            "--extractor-option" => args
                .extractor_options
                .push(next_value(&mut iter, "--extractor-option")?),
            _ => return Err(format!("unsupported backend arg: {arg}")),
        }
    }

    if args.url.is_empty()
        || args.state.as_os_str().is_empty()
        || args.output.as_os_str().is_empty()
        || args.metadata.as_os_str().is_empty()
    {
        return Err("missing required backend args".to_string());
    }
    for option in &args.extractor_options {
        if !option.starts_with("crawl4ai.") || !option.contains('=') {
            return Err(format!("unsupported extractor option: {option}"));
        }
    }
    Ok(args)
}

fn next_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("missing value for {flag}"))
}

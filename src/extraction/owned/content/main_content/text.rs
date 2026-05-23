pub(in crate::extraction::owned::content::main_content) fn word_count<'a>(
    pieces: impl IntoIterator<Item = &'a str>,
) -> usize {
    pieces
        .into_iter()
        .flat_map(str::split_whitespace)
        .filter(|word| !word.is_empty())
        .count()
}

pub(in crate::extraction::owned::content::main_content) fn normalize_text_pieces<'a>(
    pieces: impl IntoIterator<Item = &'a str>,
) -> String {
    pieces
        .into_iter()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

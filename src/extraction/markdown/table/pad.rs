pub(in crate::extraction::markdown) fn pad_markdown_tables(markdown: &str) -> String {
    let mut output = Vec::new();
    let mut table = Vec::new();

    for line in markdown.lines() {
        if is_gfm_table_line(line) {
            table.push(line.to_string());
            continue;
        }
        flush_padded_table(&mut output, &mut table);
        output.push(line.to_string());
    }
    flush_padded_table(&mut output, &mut table);

    output.join("\n")
}

fn flush_padded_table(output: &mut Vec<String>, table: &mut Vec<String>) {
    if table.is_empty() {
        return;
    }
    if table.len() < 2 {
        output.append(table);
        return;
    }

    let rows = table
        .iter()
        .map(|line| table_cells(line))
        .collect::<Vec<_>>();
    let max_columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    if max_columns == 0 {
        output.append(table);
        return;
    }

    let widths = (0..max_columns)
        .map(|column| {
            rows.iter()
                .map(|row| row.get(column).map_or(0, |cell| cell.chars().count()))
                .max()
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();

    for row in rows {
        let is_separator = row.iter().all(|cell| cell.chars().all(|ch| ch == '-'));
        output.push(format_padded_table_row(&row, &widths, is_separator));
    }
    table.clear();
}

fn is_gfm_table_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.matches('|').count() >= 2
}

fn table_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn format_padded_table_row(cells: &[String], widths: &[usize], is_separator: bool) -> String {
    let mut line = String::from("|");
    for (index, width) in widths.iter().enumerate() {
        let cell = cells.get(index).map_or("", String::as_str);
        let fill = if is_separator { '-' } else { ' ' };
        let cell_len = cell.chars().count();
        line.push(' ');
        line.push_str(cell);
        for _ in 0..(width.saturating_sub(cell_len) + 1) {
            line.push(fill);
        }
        line.push('|');
    }
    line
}

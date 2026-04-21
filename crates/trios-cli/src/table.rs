//! Markdown table parser and renderer for #143

use anyhow::Result;

/// Table row for IGLA tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableRow {
    pub task: String,
    pub agent: String,
    pub status: String,
    pub bpb: Option<f64>,
    pub ref_issue: String,
}

/// Parse markdown table from issue body
pub fn parse_table(markdown: &str) -> Result<Vec<TableRow>> {
    let rows = markdown
        .lines()
        .skip_while(|l| !l.contains("Task"))
        .skip(1) // header
        .skip_while(|l| l.contains("---"))
        .take_while(|l| l.starts_with("|"))
        .filter_map(|l| parse_row(l))
        .collect();
    
    Ok(rows)
}

fn parse_row(line: &str) -> Option<TableRow> {
    let cells: Vec<&str> = line
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|s| s.trim())
        .collect();
    
    if cells.len() < 5 {
        return None;
    }

    Some(TableRow {
        task: cells[0].to_string(),
        agent: cells[1].to_string(),
        status: cells[2].to_string(),
        bpb: cells[3].parse().ok(),
        ref_issue: cells.get(4).unwrap_or(&"").to_string(),
    })
}

/// Update table and render back to markdown
pub fn update_table(markdown: &str, agent: &str, status: &str, bpb: Option<f64>) -> Result<String> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut output = String::new();
    
    let mut in_table = false;
    let mut header_found = false;
    
    for line in &lines {
        if line.contains("Task") && !header_found {
            in_table = true;
            header_found = true;
            output.push_str(line);
            output.push('\n');
            continue;
        }
        
        if line.contains("---") {
            output.push_str(line);
            output.push('\n');
            continue;
        }
        
        if in_table && line.starts_with("|") {
            if let Some(row) = parse_row(line) {
                if row.agent == agent {
                    // Update this row
                    output.push_str(&render_row(&row.task, agent, status, bpb, &row.ref_issue));
                    output.push('\n');
                    continue;
                }
            }
        } else if in_table && !line.starts_with("|") {
            in_table = false;
        }
        
        output.push_str(line);
        output.push('\n');
    }
    
    Ok(output)
}

fn render_row(task: &str, agent: &str, status: &str, bpb: Option<f64>, ref_issue: &str) -> String {
    format!(
        "| {:39} | {:7} | {:12} | {:6} | {:10} |",
        task,
        agent,
        status,
        bpb.map(|b| b.to_string()).unwrap_or_else(|| "—".to_string()),
        ref_issue
    )
}

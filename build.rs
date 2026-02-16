use std::env;
use std::fs;
use std::path::Path;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Logcheck-based log filtering tool
#[derive(Parser)]
#[command(name = "logcheck-filter")]
#[command(about = "Filter logs using logcheck rules")]
#[command(version)]
pub struct Cli {
    /// Path to logcheck rules directory
    #[arg(long, required = true, help = "Path to logcheck rules directory")]
    pub rules: PathBuf,

    /// Output format
    #[arg(long, value_enum, default_value = "text", help = "Output format")]
    pub format: OutputFormat,

    /// Show mode
    #[arg(long, value_enum, default_value = "all", help = "What entries to show")]
    pub show: ShowMode,

    /// Show statistics after processing
    #[arg(long, help = "Show processing statistics")]
    pub stats: bool,

    /// Enable colored output
    #[arg(long, help = "Enable colored output")]
    pub color: bool,

    /// Write filtered logs to file (informational logs go to stdout)
    #[arg(long, help = "Write filtered logs to file")]
    pub output_file: Option<PathBuf>,

    /// Input source
    #[command(subcommand)]
    pub input: InputSource,
}

#[derive(Subcommand)]
pub enum InputSource {
    /// Read from a file
    File {
        /// Path to log file
        path: PathBuf,
    },
    /// Read from standard input
    Stdin,
    /// Read from systemd journal
    #[cfg(target_os = "linux")]
    Journald {
        /// Systemd unit to filter
        #[arg(long, help = "Filter by systemd unit")]
        unit: Option<String>,
        /// Follow mode (like tail -f)
        #[arg(long, help = "Follow new journal entries")]
        follow: bool,
        /// Number of lines to show from end
        #[arg(long, help = "Show last N entries")]
        lines: Option<usize>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Human-readable text format
    Text,
    /// JSON format
    Json,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ShowMode {
    /// Show all log entries
    All,
    /// Show only violations (cracking/violations)
    Violations,
    /// Show only unmatched entries
    Unmatched,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip generation when building on docs.rs
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Skipping CLI documentation generation on docs.rs");
        return Ok(());
    }

    println!("cargo:rerun-if-changed=src/cli/args.rs");

    // Create docs directory
    let docs_dir = Path::new("docs");
    if !docs_dir.exists() {
        fs::create_dir_all(docs_dir)?;
        println!("cargo:warning=Created docs/ directory");
    }

    // Generate the CLI command
    let cmd = Cli::command();

    // Generate markdown CLI reference
    let markdown_content = generate_markdown_docs(&cmd, docs_dir)?;

    // Generate HTML version for web viewing
    generate_html_docs(docs_dir, &markdown_content)?;

    // Generate man pages
    generate_man_pages(&cmd, docs_dir)?;

    Ok(())
}

fn generate_markdown_docs(
    _cmd: &clap::Command,
    docs_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let options = clap_markdown::MarkdownOptions::default()
        .title("logcheck-filter CLI Reference".to_string())
        .show_table_of_contents(true)
        .show_footer(true);

    let markdown_content = clap_markdown::help_markdown_custom::<Cli>(&options);

    let markdown_path = docs_dir.join("cli-reference.md");
    fs::write(&markdown_path, &markdown_content)?;

    println!(
        "cargo:warning=Generated markdown CLI reference at {}",
        markdown_path.display()
    );

    Ok(markdown_content)
}

fn generate_man_pages(
    cmd: &clap::Command,
    docs_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let man_dir = docs_dir.join("man");
    if !man_dir.exists() {
        fs::create_dir_all(&man_dir)?;
    }

    let man = clap_mangen::Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Vec::new();
    man.render(&mut buffer)?;

    let man_path = man_dir.join("logcheck-filter.1");
    fs::write(&man_path, buffer)?;

    println!("cargo:warning=Generated man page at {}", man_path.display());

    Ok(())
}

fn generate_html_docs(docs_dir: &Path, markdown: &str) -> Result<(), Box<dyn std::error::Error>> {
    let html_body = markdown_to_html(markdown);

    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>logcheck-filter CLI Reference</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
            line-height: 1.6;
            color: #24292e;
            max-width: 980px;
            margin: 0 auto;
            padding: 20px;
            background-color: #ffffff;
        }}
        h1 {{
            border-bottom: 1px solid #eaecef;
            padding-bottom: 0.3em;
            font-size: 2em;
            margin-top: 24px;
            margin-bottom: 16px;
        }}
        h2 {{
            border-bottom: 1px solid #eaecef;
            padding-bottom: 0.3em;
            font-size: 1.5em;
            margin-top: 24px;
            margin-bottom: 16px;
        }}
        h3 {{
            font-size: 1.25em;
            margin-top: 24px;
            margin-bottom: 16px;
        }}
        h4 {{
            font-size: 1em;
            margin-top: 24px;
            margin-bottom: 16px;
        }}
        code {{
            background-color: #f6f8fa;
            border-radius: 3px;
            padding: 0.2em 0.4em;
            font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
            font-size: 85%;
        }}
        pre {{
            background-color: #f6f8fa;
            border-radius: 6px;
            padding: 16px;
            overflow: auto;
            line-height: 1.45;
        }}
        pre code {{
            background-color: transparent;
            padding: 0;
            font-size: 100%;
        }}
        ul, ol {{
            padding-left: 2em;
            margin-top: 0;
            margin-bottom: 16px;
        }}
        li {{
            margin-top: 0.25em;
        }}
        p {{
            margin-top: 0;
            margin-bottom: 16px;
        }}
        a {{
            color: #0366d6;
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
        blockquote {{
            padding: 0 1em;
            color: #6a737d;
            border-left: 0.25em solid #dfe2e5;
            margin: 0 0 16px 0;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
            margin-bottom: 16px;
        }}
        table th, table td {{
            padding: 6px 13px;
            border: 1px solid #dfe2e5;
        }}
        table th {{
            font-weight: 600;
            background-color: #f6f8fa;
        }}
        table tr {{
            background-color: #ffffff;
            border-top: 1px solid #c6cbd1;
        }}
        table tr:nth-child(2n) {{
            background-color: #f6f8fa;
        }}
        hr {{
            height: 0.25em;
            padding: 0;
            margin: 24px 0;
            background-color: #e1e4e8;
            border: 0;
        }}
        .header {{
            margin-bottom: 32px;
        }}
        .footer {{
            margin-top: 48px;
            padding-top: 24px;
            border-top: 1px solid #eaecef;
            color: #6a737d;
            font-size: 0.9em;
        }}
    </style>
</head>
<body>
    <div class="header">
        <p><a href="index.html">← Back to Documentation Index</a></p>
    </div>
    {}
    <div class="footer">
        <p>Generated from CLI definitions using <code>clap</code> and <code>clap-markdown</code></p>
    </div>
</body>
</html>"#,
        html_body
    );

    let html_path = docs_dir.join("cli-reference.html");
    fs::write(&html_path, html_content)?;

    println!(
        "cargo:warning=Generated HTML CLI reference at {}",
        html_path.display()
    );

    Ok(())
}

/// Convert markdown to HTML using simple string replacements
/// This is a basic converter suitable for clap-markdown output
fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_list = false;
    let mut list_type = "";

    for line in markdown.lines() {
        let trimmed = line.trim();

        // Handle code blocks
        if trimmed.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }

        // Pass through raw HTML tags (clap-markdown includes some HTML)
        if trimmed.starts_with('<') && trimmed.ends_with('>') {
            close_list_if_needed(&mut html, &mut in_list);
            html.push_str(trimmed);
            html.push('\n');
            continue;
        }

        // Handle headings
        if let Some(rest) = trimmed.strip_prefix("#### ") {
            close_list_if_needed(&mut html, &mut in_list);
            html.push_str(&format!("<h4>{}</h4>\n", process_inline_markdown(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("### ") {
            close_list_if_needed(&mut html, &mut in_list);
            html.push_str(&format!("<h3>{}</h3>\n", process_inline_markdown(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            close_list_if_needed(&mut html, &mut in_list);
            html.push_str(&format!("<h2>{}</h2>\n", process_inline_markdown(rest)));
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            close_list_if_needed(&mut html, &mut in_list);
            html.push_str(&format!("<h1>{}</h1>\n", process_inline_markdown(rest)));
        }
        // Handle unordered lists (with indentation support)
        else if let Some(rest) = trimmed.strip_prefix("* ") {
            if !in_list || list_type != "ul" {
                close_list_if_needed(&mut html, &mut in_list);
                html.push_str("<ul>\n");
                in_list = true;
                list_type = "ul";
            }
            html.push_str(&format!("<li>{}</li>\n", process_inline_markdown(rest)));
        }
        // Handle dash lists (alternative list syntax)
        else if let Some(rest) = trimmed.strip_prefix("- ") {
            if !in_list || list_type != "ul" {
                close_list_if_needed(&mut html, &mut in_list);
                html.push_str("<ul>\n");
                in_list = true;
                list_type = "ul";
            }
            html.push_str(&format!("<li>{}</li>\n", process_inline_markdown(rest)));
        }
        // Handle horizontal rules
        else if trimmed == "---" || trimmed == "***" {
            close_list_if_needed(&mut html, &mut in_list);
            html.push_str("<hr>\n");
        }
        // Handle empty lines
        else if trimmed.is_empty() {
            close_list_if_needed(&mut html, &mut in_list);
            if !html.ends_with('\n') {
                html.push('\n');
            }
        }
        // Handle regular paragraphs
        else {
            close_list_if_needed(&mut html, &mut in_list);
            html.push_str(&format!("<p>{}</p>\n", process_inline_markdown(trimmed)));
        }
    }

    close_list_if_needed(&mut html, &mut in_list);
    html
}

fn close_list_if_needed(html: &mut String, in_list: &mut bool) {
    if *in_list {
        // Determine which list type to close based on the last opened tag
        if html.contains("<ul>") && !html.ends_with("</ul>\n") {
            html.push_str("</ul>\n");
        } else if html.contains("<ol>") && !html.ends_with("</ol>\n") {
            html.push_str("</ol>\n");
        }
        *in_list = false;
    }
}

/// Process inline markdown (bold, italic, code, links)
fn process_inline_markdown(text: &str) -> String {
    // If the text contains HTML tags, pass it through with minimal processing
    if text.contains('<') && text.contains('>') {
        return text.to_string();
    }

    let mut result = escape_html(text);

    // Handle inline code (must be done before other replacements)
    let mut code_processed = String::new();
    let mut in_code = false;
    let mut current = String::new();

    for ch in result.chars() {
        if ch == '`' {
            if in_code {
                code_processed.push_str(&format!("<code>{}</code>", current));
                current.clear();
            }
            in_code = !in_code;
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        code_processed.push_str(&current);
    }
    result = code_processed;

    // Handle bold **text**
    while let Some(start) = result.find("**") {
        if let Some(end) = result[start + 2..].find("**") {
            let before = &result[..start];
            let bold_text = &result[start + 2..start + 2 + end];
            let after = &result[start + 2 + end + 2..];
            result = format!("{}<strong>{}</strong>{}", before, bold_text, after);
        } else {
            break;
        }
    }

    // Handle italic *text*
    let mut italic_processed = String::new();
    let mut chars = result.chars().peekable();
    let mut in_italic = false;

    while let Some(ch) = chars.next() {
        if ch == '*' && chars.peek() != Some(&'*') {
            if in_italic {
                italic_processed.push_str("</em>");
            } else {
                italic_processed.push_str("<em>");
            }
            in_italic = !in_italic;
        } else {
            italic_processed.push(ch);
        }
    }
    result = italic_processed;

    // Handle links [text](url)
    while let Some(start) = result.find('[') {
        if let Some(mid) = result[start..].find("](") {
            if let Some(end) = result[start + mid..].find(')') {
                let before = &result[..start];
                let link_text = &result[start + 1..start + mid];
                let link_url = &result[start + mid + 2..start + mid + end];
                let after = &result[start + mid + end + 1..];
                result = format!(
                    "{}<a href=\"{}\">{}</a>{}",
                    before, link_url, link_text, after
                );
            } else {
                break;
            }
        } else {
            break;
        }
    }

    result
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

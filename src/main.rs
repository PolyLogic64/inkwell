use std::{
    fs::{copy, create_dir_all, read_to_string, remove_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use comrak::Options;
use tracing::debug;
use walkdir::WalkDir;

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    // TODO: maybe add log_level in here
}

#[derive(Debug, Subcommand)]
enum Commands {
    Serve {
        content_path: PathBuf,
    },
    Generate {
        input_path: PathBuf,
        output_path: PathBuf,
        template_path: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_ansi(false)
        .init();

    match args.command {
        Commands::Serve { content_path } => todo!("not implemented yet"),
        Commands::Generate {
            input_path,
            output_path,
            template_path,
        } => {
            debug!(
                "Command::Generate({:?}, {:?}, {:?})",
                &input_path, &output_path, &template_path
            );
            generate_site(&input_path, &output_path, &template_path)
                .context("Error generating site")?
        }
    }

    Ok(())
}

// TODO!!: this was made with ChatGPT; So I want to refactor is later on
fn generate_site(input_path: &Path, output_path: &Path, template_path: &Path) -> Result<()> {
    // TODO!!: this is not that great I think
    if output_path.exists() {
        remove_dir_all(output_path).context("Failed to delete output directory")?;
    }

    create_dir_all(output_path).context("Failed to create output directory")?;

    // Traverse through the input directory
    for entry in WalkDir::new(input_path) {
        let entry = entry.context("Error reading directory entry")?;
        debug!(?entry);
        let full_path = entry.path();

        // Compute relative path with respect to the input_path (e.g., "content/")
        let relative_path = full_path
            .strip_prefix(input_path)
            .context("Failed to strip prefix from input path")?;

        // If the relative path is empty, we're at the root (e.g., "content/") – skip it.
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        // Determine the output file path
        let mut output_file_path = output_path.join(relative_path);

        if entry.file_type().is_dir() {
            // Create the directory in the output if it doesn't exist
            if !output_file_path.exists() {
                create_dir_all(&output_file_path)
                    .context("Failed to create subdirectory in output")?;
            }
        } else {
            // For Markdown files, convert to HTML and change extension to .html
            if full_path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                output_file_path.set_extension("html");

                if let Some(parent_dir) = output_file_path.parent() {
                    create_dir_all(parent_dir)
                        .context("Failed to create parent directory for HTML file")?;
                }

                let html = convert_md_to_html(full_path, template_path)
                    .context("Failed to convert markdown to HTML")?;
                write_to_file(&output_file_path, html).context("Failed to write HTML to file")?;
            } else {
                // Otherwise, just copy the file (images, CSS, etc.)
                if let Some(parent_dir) = output_file_path.parent() {
                    create_dir_all(parent_dir)
                        .context("Failed to create parent directory for file copy")?;
                }
                // TODO: I don't know if this is, *that good* of an idea
                copy(full_path, &output_file_path).context("Failed to copy file")?;
            }
        }
    }

    Ok(())
}

fn convert_md_to_html(md_path: &Path, template_path: &Path) -> Result<String> {
    // FIXME!!: add templating
    let md_content = std::fs::read_to_string(md_path).context("Failed to read markdown file")?;
    let html_content = comrak::markdown_to_html(&md_content, &Options::default());

    let template = read_to_string(template_path).context("Failed to read template file")?;
    let full_html = template.replace("__content__", &html_content);
    Ok(full_html)
}

fn write_to_file(path: &Path, data: String) -> Result<()> {
    let mut file = File::create(path).context("Failed to create file")?;
    file.write_all(data.as_bytes())
        .context("Failed to write to file")?;
    Ok(())
}

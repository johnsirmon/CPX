use std::ffi::OsString;
use std::fs;
use std::io::{self, Read};
use std::process;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use cpx_core::ingest::{ingest, IngestRequest};
use cpx_core::project::project;
use cpx_core::symbolize::symbolize;
use cpx_core::{
    EXIT_CODE_FORMAT_MISMATCH, EXIT_CODE_GENERAL_ERROR, EXIT_CODE_INPUT_ERROR,
    EXIT_CODE_SAFETY_FAILURE, EXIT_CODE_SUCCESS, FORMAT_VERSION,
};

fn main() {
    let exit_code = match try_run(std::env::args_os()) {
        Ok(exit_code) => exit_code,
        Err(RunError::Clap(error)) => {
            let exit_code = error.exit_code();
            let _ = error.print();
            exit_code
        }
        Err(RunError::Failure(failure)) => {
            eprintln!("{:#}", failure.error);
            failure.exit_code
        }
        Err(RunError::Io(error)) => {
            eprintln!("failed to render command output: {error}");
            EXIT_CODE_GENERAL_ERROR
        }
    };

    if exit_code != 0 {
        process::exit(exit_code);
    }
}

fn try_run<I, T>(args: I) -> std::result::Result<i32, RunError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::try_parse_from(args).map_err(RunError::Clap)?;

    match cli.command {
        Some(Commands::Ingest(args)) => handle_ingest(args).map_err(RunError::Failure),
        Some(Commands::Project(args)) => handle_project(args).map_err(RunError::Failure),
        Some(Commands::Rehydrate(args)) => handle_rehydrate(args).map_err(RunError::Failure),
        None => {
            Cli::command().print_help().map_err(RunError::Io)?;
            println!();
            Ok(EXIT_CODE_SUCCESS)
        }
    }
}

fn handle_ingest(args: CommonArgs) -> std::result::Result<i32, CliFailure> {
    let request = read_input(args.input.as_deref()).map_err(|error| CliFailure {
        exit_code: EXIT_CODE_INPUT_ERROR,
        error,
    })?;

    let document = ingest(request).map_err(|error| CliFailure {
        exit_code: EXIT_CODE_INPUT_ERROR,
        error: error.into(),
    })?;

    let mut summary = String::new();
    summary.push_str("SOURCE ");
    summary.push_str(&document.source_name);
    summary.push('\n');
    summary.push_str("LINES ");
    summary.push_str(&document.line_count.to_string());
    summary.push('\n');
    summary.push_str("CHARS ");
    summary.push_str(&document.contents.chars().count().to_string());
    summary.push('\n');

    write_output(args.output.as_deref(), &summary).map_err(|error| CliFailure {
        exit_code: EXIT_CODE_GENERAL_ERROR,
        error,
    })?;

    Ok(EXIT_CODE_SUCCESS)
}

fn handle_project(args: ProjectArgs) -> std::result::Result<i32, CliFailure> {
    if args.format != FORMAT_VERSION {
        return Err(CliFailure {
            exit_code: EXIT_CODE_FORMAT_MISMATCH,
            error: anyhow::anyhow!(
                "unsupported format '{}'; only '{}' is currently accepted",
                args.format,
                FORMAT_VERSION
            ),
        });
    }

    let request = read_input(args.input.as_deref()).map_err(|error| CliFailure {
        exit_code: EXIT_CODE_INPUT_ERROR,
        error,
    })?;

    let document = ingest(request).map_err(|error| CliFailure {
        exit_code: EXIT_CODE_INPUT_ERROR,
        error: error.into(),
    })?;

    let symbolized = symbolize(&document).map_err(|error| CliFailure {
        exit_code: EXIT_CODE_SAFETY_FAILURE,
        error: error.into(),
    })?;

    let projection = project(&symbolized).map_err(|error| CliFailure {
        exit_code: EXIT_CODE_GENERAL_ERROR,
        error: error.into(),
    })?;

    write_output(args.output.as_deref(), &projection.body).map_err(|error| CliFailure {
        exit_code: EXIT_CODE_GENERAL_ERROR,
        error,
    })?;

    Ok(EXIT_CODE_SUCCESS)
}

fn handle_rehydrate(_args: CommonArgs) -> std::result::Result<i32, CliFailure> {
    Err(CliFailure {
        exit_code: EXIT_CODE_GENERAL_ERROR,
        error: anyhow::anyhow!(
            "rehydration is intentionally not implemented yet; M1 focuses on safe ingest and projection"
        ),
    })
}

fn read_input(input: Option<&str>) -> Result<IngestRequest> {
    match input {
        Some("-") | None => {
            let mut contents = String::new();
            io::stdin()
                .read_to_string(&mut contents)
                .context("failed to read stdin")?;

            Ok(IngestRequest {
                source_name: "stdin".to_owned(),
                contents,
            })
        }
        Some(path) => {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("failed to read '{path}'"))?;

            Ok(IngestRequest {
                source_name: path.to_owned(),
                contents,
            })
        }
    }
}

fn write_output(path: Option<&str>, contents: &str) -> Result<()> {
    if let Some(path) = path {
        fs::write(path, contents).with_context(|| format!("failed to write '{path}'"))?;
    } else {
        print!("{contents}");
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(name = "cpx", version, about = "Local-first support case preparation for AI workflows.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Ingest(CommonArgs),
    Project(ProjectArgs),
    Rehydrate(CommonArgs),
}

#[derive(clap::Args, Debug)]
struct CommonArgs {
    input: Option<String>,
    #[arg(long)]
    output: Option<String>,
}

#[derive(clap::Args, Debug)]
struct ProjectArgs {
    input: Option<String>,
    #[arg(long)]
    output: Option<String>,
    #[arg(long, default_value = FORMAT_VERSION)]
    format: String,
}

#[derive(Debug)]
enum RunError {
    Clap(clap::Error),
    Failure(CliFailure),
    Io(io::Error),
}

#[derive(Debug)]
struct CliFailure {
    exit_code: i32,
    error: anyhow::Error,
}

#[cfg(test)]
mod tests {
    use super::try_run;
    use cpx_core::{EXIT_CODE_FORMAT_MISMATCH, EXIT_CODE_SUCCESS};

    #[test]
    fn no_subcommand_prints_help_and_succeeds() {
        let exit_code = try_run(["cpx"]).expect("expected success");

        assert_eq!(exit_code, EXIT_CODE_SUCCESS);
    }

    #[test]
    fn project_rejects_unsupported_format_with_specific_exit_code() {
        let error = try_run(["cpx", "project", "-", "--format", "future-format"])
            .expect_err("expected format mismatch");

        match error {
            super::RunError::Failure(failure) => {
                assert_eq!(failure.exit_code, EXIT_CODE_FORMAT_MISMATCH);
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}


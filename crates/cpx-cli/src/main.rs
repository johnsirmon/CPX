use std::ffi::OsString;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use cpx_core::ingest::{ingest, IngestRequest};
use cpx_core::project::project;
use cpx_core::rehydrate::{detect_case_id, rehydrate, RehydrateError, RehydrateRequest};
use cpx_core::symbolize::symbolize;
use cpx_core::vault::{open as open_vault, store as store_vault, StoreRequest};
use cpx_core::{
    EXIT_CODE_FORMAT_MISMATCH, EXIT_CODE_GENERAL_ERROR, EXIT_CODE_INPUT_ERROR,
    EXIT_CODE_SAFETY_FAILURE, EXIT_CODE_SUCCESS, EXIT_CODE_VAULT_ERROR, FORMAT_VERSION,
};

const DEFAULT_PASSPHRASE_ENV: &str = "CPX_PASSPHRASE";

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
    let request = read_input(args.input.as_deref()).map_err(input_failure)?;
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

    write_output(args.output.as_deref(), &summary).map_err(general_failure)?;

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

    let request = read_input(args.input.as_deref()).map_err(input_failure)?;
    let document = ingest(request).map_err(|error| CliFailure {
        exit_code: EXIT_CODE_INPUT_ERROR,
        error: error.into(),
    })?;
    let symbolized = symbolize(&document).map_err(|error| CliFailure {
        exit_code: EXIT_CODE_SAFETY_FAILURE,
        error: error.into(),
    })?;
    let projection = project(&symbolized).map_err(general_failure)?;

    if let Some(vault_output) = resolve_project_vault(&args, &projection.case_id)? {
        store_vault(
            &vault_output.path,
            &StoreRequest {
                case_id: &projection.case_id,
                entries: &symbolized.entries,
                passphrase: &vault_output.passphrase,
            },
        )
        .map_err(vault_failure)?;
        eprintln!("vault written to {}", vault_output.path.display());
    }

    write_output(args.output.as_deref(), &projection.body).map_err(general_failure)?;

    Ok(EXIT_CODE_SUCCESS)
}

fn handle_rehydrate(args: RehydrateArgs) -> std::result::Result<i32, CliFailure> {
    let request = read_input(args.input.as_deref()).map_err(input_failure)?;
    let vault_path = resolve_rehydrate_vault_path(&args, &request).map_err(vault_failure)?;
    let passphrase = resolve_required_passphrase(&args.passphrase_env).map_err(vault_failure)?;
    let vault = open_vault(&vault_path, &passphrase).map_err(vault_failure)?;
    let rehydrated = rehydrate(&RehydrateRequest {
        projection_response: request.contents,
        vault,
    })
    .map_err(map_rehydrate_failure)?;

    write_output(args.output.as_deref(), &rehydrated).map_err(general_failure)?;

    Ok(EXIT_CODE_SUCCESS)
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
            let contents =
                fs::read_to_string(path).with_context(|| format!("failed to read '{path}'"))?;

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

fn resolve_project_vault(
    args: &ProjectArgs,
    case_id: &str,
) -> std::result::Result<Option<ResolvedVaultOutput>, CliFailure> {
    let passphrase = std::env::var(&args.passphrase_env).ok();

    match (args.vault_output.as_deref(), passphrase) {
        (None, None) => Ok(None),
        (_, Some(passphrase)) if passphrase.is_empty() => Err(vault_failure(anyhow::anyhow!(
            "environment variable '{}' was empty",
            args.passphrase_env
        ))),
        (Some(_), None) => Err(vault_failure(anyhow::anyhow!(
            "vault output requested but passphrase environment variable '{}' was not set",
            args.passphrase_env
        ))),
        (vault_output, Some(passphrase)) => {
            let path = vault_output.map_or_else(
                || default_vault_path(args.output.as_deref(), case_id),
                PathBuf::from,
            );

            Ok(Some(ResolvedVaultOutput { path, passphrase }))
        }
    }
}

fn resolve_rehydrate_vault_path(args: &RehydrateArgs, request: &IngestRequest) -> Result<PathBuf> {
    if let Some(path) = args.vault.as_deref() {
        return Ok(PathBuf::from(path));
    }

    if request.source_name == "stdin" {
        anyhow::bail!("--vault is required when rehydrating from stdin");
    }

    let case_id = detect_case_id(&request.contents).context(
        "could not infer a vault path from the rehydration input; pass --vault explicitly",
    )?;
    let input_path = Path::new(&request.source_name);
    let parent = input_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    Ok(parent.join(format!("{case_id}.cpxvault")))
}

fn resolve_required_passphrase(env_name: &str) -> Result<String> {
    let value = std::env::var(env_name)
        .with_context(|| format!("environment variable '{env_name}' was not set"))?;

    if value.is_empty() {
        anyhow::bail!("environment variable '{env_name}' was empty");
    }

    Ok(value)
}

fn default_vault_path(output_path: Option<&str>, case_id: &str) -> PathBuf {
    let base_dir = output_path
        .and_then(|path| Path::new(path).parent())
        .filter(|path| !path.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    base_dir.join(format!("{case_id}.cpxvault"))
}

fn input_failure(error: impl Into<anyhow::Error>) -> CliFailure {
    CliFailure {
        exit_code: EXIT_CODE_INPUT_ERROR,
        error: error.into(),
    }
}

fn general_failure(error: impl Into<anyhow::Error>) -> CliFailure {
    CliFailure {
        exit_code: EXIT_CODE_GENERAL_ERROR,
        error: error.into(),
    }
}

fn vault_failure(error: impl Into<anyhow::Error>) -> CliFailure {
    CliFailure {
        exit_code: EXIT_CODE_VAULT_ERROR,
        error: error.into(),
    }
}

fn map_rehydrate_failure(error: RehydrateError) -> CliFailure {
    match error {
        RehydrateError::EmptyInput => CliFailure {
            exit_code: EXIT_CODE_INPUT_ERROR,
            error: error.into(),
        },
        RehydrateError::FormatMismatch { .. } => CliFailure {
            exit_code: EXIT_CODE_FORMAT_MISMATCH,
            error: error.into(),
        },
        RehydrateError::CaseMismatch { .. } => CliFailure {
            exit_code: EXIT_CODE_VAULT_ERROR,
            error: error.into(),
        },
    }
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
    Rehydrate(RehydrateArgs),
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
    #[arg(long)]
    vault_output: Option<String>,
    #[arg(long, default_value = DEFAULT_PASSPHRASE_ENV)]
    passphrase_env: String,
}

#[derive(clap::Args, Debug)]
struct RehydrateArgs {
    input: Option<String>,
    #[arg(long)]
    output: Option<String>,
    #[arg(long)]
    vault: Option<String>,
    #[arg(long, default_value = DEFAULT_PASSPHRASE_ENV)]
    passphrase_env: String,
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

#[derive(Debug)]
struct ResolvedVaultOutput {
    path: PathBuf,
    passphrase: String,
}

#[cfg(test)]
mod tests {
    use super::{default_vault_path, try_run};
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

    #[test]
    fn default_vault_path_uses_the_output_directory() {
        let output_path = std::path::PathBuf::from("fixtures").join("projection.txt");
        let expected_path = std::path::PathBuf::from("fixtures").join("canonical-case.cpxvault");
        let vault_path = default_vault_path(output_path.to_str(), "canonical-case");

        assert_eq!(vault_path, expected_path);
    }
}

//! Easy integration with tools that already use `wasm-opt` via CLI.

use std::process::Command;
use std::iter::Iterator;
use std::result::Result;
use std::path::PathBuf;
use std::ffi::{OsStr, OsString};
use thiserror::Error;
use crate::api::{OptimizationOptions, FileType};
use crate::run::OptimizationError;
use crate::profiles::Profile;

/// Interpret a pre-built [`Command`] as an [`OptimizationOptions`],
/// then call [`OptimizationOptions::run`] on it.
///
/// This function is meant for easy integration with tools that already
/// call `wasm-opt` via the command line, allowing them to use either
/// the command-line tool or the integrated API from a single `Command` builder.
/// New programs that just need to optimize wasm should use `OptimizationOptions` directly.
///
/// This function is provided on a best-effort bases to support programs
/// trying to integrate with the crate.
/// In general, it should support any command line options that are also supported
/// by the `OptimizationOptions` API,
/// but it may not parse &mdash; and in some cases may not even interpret &mdash;
/// those commands in exactly the same way.
/// It is meant to make it _possible_ to produce a single command-line that works
/// with both the CLI and the API,
/// not to reproduce the behavior of the CLI perfectly.
///
/// The `-o` argument is required, followed by a path &mdash;
/// the `wasm-opt` tool writes the optimized module to stdout by default,
/// but this library is not currently capable of that.
/// `-o` specifies a file in which to write the module.
/// If `-o` is not provided, [`Error::OutputFileRequired`] is returned.
///
/// Only the arguments to `command` are interpreted;
/// environment variables and other settings are ignored.
///
/// # Errors
///
/// - Returns [`Error::Unsupported`] if any argument is not understood.
/// - Returns [`Error::OutputFileRequired`] if the `-o` argument and subsequent path
///   are not provided.
pub fn run_from_command_args(command: Command) -> Result<(), Error> {
    let parsed = parse_command_args(command)?;

    parsed.opts.run_with_sourcemaps(
        parsed.input_file,
        parsed.input_sourcemap,
        parsed.output_file,
        parsed.output_sourcemap,
    )?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("An input file is required")]
    InputFileRequired,
    #[error("The `-o` option to `wasm-opt` is required")]
    OutputFileRequired,
    #[error("The `wasm-opt` argument list ended while expecting another argument")]
    UnexpectedEndOfArgs,
    #[error("Unsupported `wasm-opt` command-line arguments: {args:?}")]
    Unsupported {
        args: Vec<OsString>,
    },
    #[error("Error while optimization wasm modules")]
    ExecutionError(
        #[from]
        #[source]
        OptimizationError
    )
}

struct ParsedCliArgs {
    opts: OptimizationOptions,
    input_file: PathBuf,
    input_sourcemap: Option<PathBuf>,
    output_file: PathBuf,
    output_sourcemap: Option<PathBuf>,
}

#[rustfmt::skip]
fn parse_command_args(command: Command) -> Result<ParsedCliArgs, Error> {
    let mut opts = OptimizationOptions::default();

    let mut args = command.get_args();

    let mut input_file: Option<PathBuf> = None;
    let mut input_sourcemap: Option<PathBuf> = None;
    let mut output_file: Option<PathBuf> = None;
    let mut output_sourcemap: Option<PathBuf> = None;

    let mut unsupported: Vec<OsString> = vec![];

    while let Some(arg) = args.next() {
        let arg = if let Some(arg) = arg.to_str() {
            arg
        } else {
            // Not unicode. Might still be the infile.
            parse_infile_path(arg, &mut input_file, &mut unsupported);
            continue;
        };

        // Keep these cases in order they are listed in the original cpp files.
        match arg {
            /* from wasm-opt.cpp */

            "--output" | "-o" => {
                parse_path_into(&mut args, &mut output_file, &mut unsupported)?;
            }
            "--emit-text" | "-S" => {
                opts.writer_file_type(FileType::Wat);
            }
            "--input-source-map" | "-ism" => {
                parse_path_into(&mut args, &mut input_sourcemap, &mut unsupported)?;
            }
            "--output-source-map" | "-osm" => {
                parse_path_into(&mut args, &mut output_sourcemap, &mut unsupported)?;
            }
            "--output-source-map-url" | "-osu" => {
                todo!()
            }

            /* from optimization-options.h */

            "-O" => {
                Profile::default().apply_to_opts(&mut opts);
            }
            "-O0" => {
                Profile::opt_level_0().apply_to_opts(&mut opts);
            }
            "-O1" => {
                Profile::opt_level_1().apply_to_opts(&mut opts);
            }
            "-O2" => {
                Profile::opt_level_2().apply_to_opts(&mut opts);
            }
            "-O3" => {
                Profile::opt_level_3().apply_to_opts(&mut opts);
            }
            "-O4" => {
                Profile::opt_level_4().apply_to_opts(&mut opts);
            }
            "-Os" => {
                Profile::optimize_for_size().apply_to_opts(&mut opts);
            }
            "-Oz" => {
                Profile::optimize_for_size_aggressively().apply_to_opts(&mut opts);
            }
            "--optimize-level" | "-ol" => {
                todo!()
            }
            "--shrink-level" | "-s" => {
                todo!()
            }
            "--debuginfo" | "-g" => {
                todo!()
            }
            "--always-inline-max-function-size" | "-aimfs" => {
                todo!()
            }
            "--flexible-inline-max-function-size" | "-fimfs" => {
                todo!()
            }
            "--one-caller-inline-max-function-size" | "-ocifms" => {
                todo!()
            }
            "--inline-functions-with-loops" | "-ifwl" => {
                todo!()
            }
            "--partial-inlining-ifs" | "-pii" => {
                todo!()
            }
            "--traps-never-happen" | "-tnh" => {
                todo!()
            }
            "--low-memory-unused" | "-lmu" => {
                todo!()
            }
            "--fast-math" | "-ffm" => {
                todo!()
            }
            "--zero-filled-memory" | "-uim" => {
                todo!()
            }

            /* from tool-options.h */

            "--mvp-features" | "-mvp" => {
                todo!()
            }
            "--all-features" | "-all" => {
                todo!()
            }
            "--quiet" | "-q" => {
                /* pass */
            }
            "--no-validation" | "-n" => {
                todo!()
            }
            "--pass-arg" | "-pa" => {
                todo!()
            }

            /* fallthrough */

            _ => {
                // todo parse pass names
                // todo parse enable/disable feature names

                parse_infile_path(OsStr::new(arg), &mut input_file, &mut unsupported);
            }
        }
    }

    let input_file = if let Some(input_file) = input_file {
        input_file
    } else {
        return Err(Error::InputFileRequired);
    };
    let output_file = if let Some(output_file) = output_file {
        output_file
    } else {
        return Err(Error::OutputFileRequired);
    };

    if unsupported.len() > 0 {
        return Err(Error::Unsupported {
            args: unsupported,
        });
    }

    Ok(ParsedCliArgs {
        opts,
        input_file,
        input_sourcemap,
        output_file,
        output_sourcemap,
    })
}    

fn parse_infile_path(
    arg: &OsStr,
    maybe_input_file: &mut Option<PathBuf>,
    unsupported: &mut Vec<OsString>,
) {
    if maybe_input_file.is_none() {
        *maybe_input_file = Some(PathBuf::from(arg));
    } else {
        unsupported.push(OsString::from(arg));
    }
}

fn parse_path_into<'item>(
    args: &mut impl Iterator<Item = &'item OsStr>,
    maybe_path: &mut Option<PathBuf>,
    unsupported: &mut Vec<OsString>,
) -> Result<(), Error> {
    if let Some(arg) = args.next() {
        if maybe_path.is_none() {
            *maybe_path = Some(PathBuf::from(arg));
        } else {
            unsupported.push(OsString::from(arg));
        }

        Ok(())
    } else {
        Err(Error::UnexpectedEndOfArgs)
    }
}

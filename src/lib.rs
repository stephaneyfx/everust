// Copyright (C) 2017 Stephane Raux. Distributed under the MIT license.

#![deny(warnings)]
#![deny(missing_docs)]

//! Rust code evaluation

extern crate tempdir;

use std::error::Error;
use std::fmt::{Display, self};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use tempdir::TempDir;

/// Type of errors that can occur when calling `eval`.
#[derive(Debug)]
pub struct EvalError(EvalCodeError);

impl EvalError {
    /// Indicates if the error was caused by a build failure.
    pub fn build_failed(&self) -> bool {
        if let EvalCodeError::Build(_) = self.0 {true} else {false}
    }
}

impl Error for EvalError {
    fn cause(&self) -> Option<&Error> {
        match self.0 {
            EvalCodeError::Build(_) => None,
            EvalCodeError::FileIo(ref e) => Some(e),
            EvalCodeError::RunProgram(_) => None,
            EvalCodeError::StartBuild(ref e) => Some(e),
            EvalCodeError::StartProgram(ref e) => Some(e),
        }
    }

    fn description(&self) -> &str {
        match self.0 {
            EvalCodeError::Build(_) => "Build failed",
            EvalCodeError::FileIo(_) => "File IO error",
            EvalCodeError::RunProgram(_) => "Program didn't terminate \
                successfully",
            EvalCodeError::StartBuild(_) => "Failed to start build",
            EvalCodeError::StartProgram(_) => "Failed to start program",
        }
    }
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())?;
        let s = match self.0 {
            EvalCodeError::Build(ref s) => s,
            EvalCodeError::RunProgram(ref s) => s,
            _ => return Ok(()),
        };
        write!(f, "\n{}", s)
    }
}

/// Evaluates rust code.
///
/// The code is implicitly enclosed in braces to make it an expression.
/// The type of the expression must be `Debug`. If successful, the result of the
/// evaluation is formatted with `Debug` and returned.
///
/// # Errors
///
/// If the evaluation fails because of a build error, the returned error value
/// can be formatted with `Display` to inspect the build errors.
///
/// # Examples
///
/// ```rust
/// use everust::eval;
/// assert_eq!("2", eval("let n = 1; n + 1").unwrap());
/// ```
pub fn eval(code: &str) -> Result<String, EvalError> {
    eval_code(code).map_err(EvalError)
}

#[derive(Debug)]
enum EvalCodeError {
    Build(String),
    FileIo(io::Error),
    RunProgram(String),
    StartBuild(io::Error),
    StartProgram(io::Error),
}

fn eval_code(code: &str) -> Result<String, EvalCodeError> {
    let temp = TempDir::new("").map_err(EvalCodeError::FileIo)?;
    let code_path = temp.path().join("main.rs");
    write_source_file(&code_path, code).map_err(EvalCodeError::FileIo)?;
    let out_path = temp.path().join("main");
    let out = Command::new("rustc")
        .arg("-o")
        .arg(&out_path)
        .arg(&code_path)
        .output()
        .map_err(EvalCodeError::StartBuild)?;
    if !out.status.success() {
        return Err(EvalCodeError::Build(
            String::from_utf8_lossy(&out.stderr).into_owned()))
    }
    let out = Command::new(&out_path).output()
        .map_err(EvalCodeError::StartProgram)?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(EvalCodeError::RunProgram(String::from_utf8_lossy(&out.stderr)
            .into_owned()))
    }
}

fn write_source_file(path: &Path, code: &str) -> io::Result<()> {
    let mut f = File::create(path)?;
    write!(&mut f, r##"
fn main() {{
    let expr = {{{}}};
    print!("{{:?}}", expr);
}}
    "##, code)?;
    f.sync_all()?;
    Ok(())
}

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
pub enum EvalError {
    /// The string contains the build messages.
    BuildFailed(String),
    /// Other type of error.
    OtherErr(OtherFailure),
    /// The string contains what was written by the program to stderr.
    ProgExitedWithErr(String),
}

impl Error for EvalError {
    fn cause(&self) -> Option<&Error> {
        match *self {
            EvalError::OtherErr(ref e) => Some(&e.0),
            _ => None,
        }
    }

    fn description(&self) -> &str {
        match *self {
            EvalError::BuildFailed(_) => "Build failed",
            EvalError::OtherErr(_) => "Other error",
            EvalError::ProgExitedWithErr(_) => "Program exited with error",
        }
    }
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())?;
        let s = match *self {
            EvalError::BuildFailed(ref s) => s,
            EvalError::ProgExitedWithErr(ref s) => s,
            _ => return Ok(()),
        };
        write!(f, "\n{}", s)
    }
}

/// Other type of errors that can occur when evaluating rust code.
#[derive(Debug)]
pub struct OtherFailure(OtherError);

#[derive(Debug)]
enum OtherError {
    FailedToCreateTempDir(io::Error),
    FailedToSpawnProg(io::Error),
    FailedToSpawnRustc(io::Error),
    FailedToWriteSrcFile(io::Error),
}

impl Error for OtherError {
    fn cause(&self) -> Option<&Error> {
        match *self {
            OtherError::FailedToCreateTempDir(ref e) => Some(e),
            OtherError::FailedToSpawnProg(ref e) => Some(e),
            OtherError::FailedToSpawnRustc(ref e) => Some(e),
            OtherError::FailedToWriteSrcFile(ref e) => Some(e),
        }
    }

    fn description(&self) -> &str {
        match *self {
            OtherError::FailedToCreateTempDir(_) => "Failed to create \
                temporary directory",
            OtherError::FailedToSpawnProg(_) => "Failed to spawn program",
            OtherError::FailedToSpawnRustc(_) => "Failed to spawn rustc",
            OtherError::FailedToWriteSrcFile(_) => "Failed to write \
                source file",
        }
    }
}

impl Display for OtherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<OtherError> for EvalError {
    fn from(e: OtherError) -> EvalError {
        EvalError::OtherErr(OtherFailure(e))
    }
}

/// Evaluates rust code.
///
/// The code is implicitly enclosed in braces to make it an expression.
/// The type of the expression must be `Debug`. If successful, the result of the
/// evaluation is formatted with `Debug` and returned.
///
/// # Limitations
///
/// * Building is delegated to rustc.
/// * rustc needs to be in the PATH.
/// * It is slow.
/// * External crates are not supported.
///
/// # Examples
///
/// ```rust
/// use everust::eval;
/// assert_eq!("2", eval("let n = 1; n + 1").unwrap());
/// ```
pub fn eval(code: &str) -> Result<String, EvalError> {
    let temp = TempDir::new("").map_err(OtherError::FailedToCreateTempDir)?;
    let code_path = temp.path().join("main.rs");
    write_source_file(&code_path, code)
        .map_err(OtherError::FailedToWriteSrcFile)?;
    let out_path = temp.path().join("main");
    let out = Command::new("rustc")
        .arg("-o")
        .arg(&out_path)
        .arg(&code_path)
        .output()
        .map_err(OtherError::FailedToSpawnRustc)?;
    if !out.status.success() {
        return Err(EvalError::BuildFailed(String::from_utf8_lossy(
            &out.stderr).into_owned()))
    }
    let out = Command::new(&out_path).output()
        .map_err(OtherError::FailedToSpawnProg)?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(EvalError::ProgExitedWithErr(String::from_utf8_lossy(
            &out.stderr).into_owned()))
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
    f.sync_all()
}

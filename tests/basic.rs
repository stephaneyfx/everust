// Copyright (C) 2017 Stephane Raux. Distributed under the MIT license.

#![deny(warnings)]

extern crate everust;

use everust::eval;

#[test]
fn eval_invalid() {
    let error = eval(r##""blah" + 4"##).unwrap_err();
    let failure = match error {
        everust::EvalError::BuildFailed(_) => true,
        _ => false,
    };
    assert!(failure);
}

#[test]
fn eval_number() {
    assert_eq!("4", eval("2 + 2").unwrap());
}

#[test]
fn eval_string() {
    let code = r##"let s = "Hello, ".to_string(); s + "World!""##;
    assert_eq!(r##""Hello, World!""##, eval(code).unwrap());
}

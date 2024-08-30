use std::{path::PathBuf, process::Command};

pub fn preprocess(input: &PathBuf, output: &PathBuf) {
    let command_output = Command::new("gcc")
        .arg("-E")
        .arg("-P")
        .arg(input)
        .arg("-o")
        .arg(output)
        .output()
        .unwrap();

    assert!(command_output.status.success());
}

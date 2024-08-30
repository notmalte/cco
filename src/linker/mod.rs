use std::{path::PathBuf, process::Command};

pub fn link(input: &PathBuf, output: &PathBuf) {
    let command_output = Command::new("gcc")
        .arg(input)
        .arg("-o")
        .arg(output)
        .output()
        .unwrap();

    assert!(command_output.status.success());
}

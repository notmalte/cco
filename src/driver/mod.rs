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

    if !command_output.status.success() {
        panic!("Failed to preprocess: {:?}", command_output);
    }
}

pub fn assemble(input: &PathBuf, output: &PathBuf) {
    let command_output = Command::new("gcc")
        .arg("-c")
        .arg(input)
        .arg("-o")
        .arg(output)
        .output()
        .unwrap();

    if !command_output.status.success() {
        panic!("Failed to assemble: {:?}", command_output);
    }
}

pub fn assemble_and_link(input: &PathBuf, output: &PathBuf) {
    let command_output = Command::new("gcc")
        .arg(input)
        .arg("-o")
        .arg(output)
        .output()
        .unwrap();

    if !command_output.status.success() {
        panic!("Failed to link: {:?}", command_output);
    }
}

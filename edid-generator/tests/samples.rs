use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use rstest::rstest;

#[derive(Debug)]
struct TestCommand(Command);

impl TestCommand {
    fn new() -> Self {
        let test_bin = std::env::current_exe().unwrap();

        let bin = test_bin
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(env!("CARGO_PKG_NAME"));

        Self(Command::new(bin))
    }

    fn add_sample<T>(mut self, sample: T) -> Self
    where
        T: AsRef<Path>,
    {
        self.0.arg(sample.as_ref().as_os_str());
        self
    }

    fn output(&mut self) -> Result<Output, std::io::Error> {
        self.0.output()
    }
}

fn edid_equals(actual: &[u8], expected: &[u8]) -> bool {
    if actual.len() != expected.len() {
        println!(
            "Sizes don't match: current {} vs expected {}",
            actual.len(),
            expected.len()
        );

        return false;
    }

    let mut identical = true;
    for (idx, (actual_byte, expected_byte)) in
        Iterator::zip(actual.iter(), expected.iter()).enumerate()
    {
        if actual_byte != expected_byte {
            println!(
                "Index {:#x} is different: actual {:#x} vs expected {:#x}",
                idx, actual_byte, expected_byte
            );

            identical = false;
        }
    }

    identical
}

#[rstest]
fn test(#[files("samples/*.yaml")] sample: PathBuf) {
    println!("Sample Path {}", sample.display());

    let output = TestCommand::new()
        .add_sample(&sample)
        .output()
        .expect("Couldn't execute our command");

    assert!(
        output.status.success(),
        "Command execution failed: {}.",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut expected_path =
        PathBuf::from("tests/data").join(sample.file_name().expect("Sample doesn't have a name?!"));
    expected_path.set_extension("yaml.bin");

    println!("Expected Binary Content Path {}", expected_path.display());

    if expected_path.exists() {
        let expected = std::fs::read(&expected_path).expect("Expected File couldn't be accessed");
        let expected_md5 = md5::compute(&expected);
        println!("Expected output MD5 {:x}", expected_md5);

        assert!(edid_equals(&output.stdout, &expected))
    } else {
        println!("File doesn't exist, skipping check.");
    }
}

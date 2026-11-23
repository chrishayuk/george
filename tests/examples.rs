//! Every program in examples/ is run with its data, and what it punches
//! must match its .out file; where there is a .stop file, the run must end
//! on that stop.

use std::path::Path;
use std::process::Command;

#[test]
fn examples_punch_what_they_should() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut programs: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "george"))
        .collect();
    programs.sort();
    assert!(programs.len() >= 15);
    for program in programs {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_george"));
        cmd.arg("run").arg(&program);
        let data = program.with_extension("in");
        if data.exists() {
            cmd.arg("--data").arg(&data);
        }
        let out = cmd.output().unwrap();
        let expected = std::fs::read_to_string(program.with_extension("out")).unwrap();
        let name = program.file_name().unwrap().to_string_lossy();
        assert_eq!(String::from_utf8_lossy(&out.stdout), expected, "{name}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        match std::fs::read_to_string(program.with_extension("stop")) {
            Ok(stop) => {
                assert!(
                    stderr.starts_with(&format!("stop {}:", stop.trim())),
                    "{name}: {stderr}"
                );
                assert_eq!(out.status.code(), Some(1), "{name}");
            }
            Err(_) => assert!(out.status.success(), "{name}: {stderr}"),
        }
    }
}

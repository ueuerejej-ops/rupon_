use std::process::Command;

// Black-box regression test: runs the actual compiled binary and checks
// its real stdout output, exactly as a user running the program would see it.
// No internal functions are called and no project source files are touched.
//
// Covers the current hardcoded sample program in main.rs, which exercises
// while / if / break / float. See PR description for manual verification
// notes (including the matching LLVM IR check).

#[test]
fn compiler_runs_sample_program_with_expected_control_flow() {
    let exe = env!("CARGO_BIN_EXE_compiler");

    let output = Command::new(exe)
        .output()
        .expect("failed to run compiler binary");

    assert!(
        output.status.success(),
        "compiler exited with a non-zero status: {:?}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // The loop increments i from 1 to 8, and breaks the moment i == 8
    // *before* the print("ee") call, so exactly 7 "ee" prints are expected.
    let ee_count = stdout.lines().filter(|line| *line == "ee").count();
    assert_eq!(
        ee_count, 7,
        "expected exactly 7 'ee' prints (break should fire on i == 8 before the print), got {ee_count}"
    );

    // fs(23) runs after the loop and should print "dd" as the final output line.
    let last_line = stdout.lines().rev().find(|line| !line.is_empty());
    assert_eq!(
        last_line,
        Some("dd"),
        "expected the final printed line to be 'dd' from the fs() call"
    );
}

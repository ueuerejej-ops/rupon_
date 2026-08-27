use std::process::Command;

// Black-box regression test: runs the actual compiled binary and checks
// its real stdout output, exactly as a user running the program would see it.
// No internal functions are called and no project source files are touched.
//
// Covers the current sample program in main.rs, which exercises
// loop / continue / break.

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
    let printed_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| *line == "A" || *line == "END")
        .collect();

    // The loop counts i from 1 to 7. continue skips the print on i == 3.
    // break exits the loop on i == 7, BEFORE that iteration's print, so
    // exactly 5 "A" prints are expected (i = 1,2,4,5,6), followed by "END".
    let a_count = printed_lines.iter().filter(|line| **line == "A").count();
    assert_eq!(
        a_count, 5,
        "expected exactly 5 'A' prints (continue skips i==3, break exits before printing on i==7), got {a_count}"
    );

    let last_line = printed_lines.last();
    assert_eq!(
        last_line,
        Some(&"END"),
        "expected the final printed line to be 'END'"
    );
}

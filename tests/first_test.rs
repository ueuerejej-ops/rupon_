use std::process::Command;

// Black-box regression test: runs the actual compiled binary and checks
// its real stdout output, exactly as a user running the program would see it.
// No internal functions are called and no project source files are touched.
//
// Covers the current sample program in main.rs, which exercises
// if / else / float comparisons / while / continue.

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
        .filter(|line| *line == "work" || *line == "ee")
        .collect();

    // Two float comparisons (423.3 == 23.4, and 32 == 32.00000000001) are
    // both false, so both if/else blocks should print "work", in order,
    // before the loop runs.
    assert_eq!(
        &printed_lines[0..2],
        &["work", "work"],
        "expected both float comparisons to print 'work' (comparisons should be false)"
    );

    // The loop counts i from 1 to 10, using `continue` (not break) to skip
    // the print on i == 2 and i == 4, so exactly 8 "ee" prints are expected.
    let ee_count = printed_lines.iter().filter(|line| **line == "ee").count();
    assert_eq!(
        ee_count, 8,
        "expected exactly 8 'ee' prints (continue should skip the print on i == 2 and i == 4, not exit the loop), got {ee_count}"
    );
}

//! The shared §2.2 source gate covers every physical line, including comments,
//! whitespace, attributes, and tests, under the 800-line hard limit.

#[test]
fn source_files_stay_under_the_limit() {
    macroquad_toolkit::source_gate::assert_source_files_within_limit(
        env!("CARGO_MANIFEST_DIR"),
        &[],
    );
}

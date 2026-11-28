//! Compile-time contract fixtures for the command declarations.
//!
//! The accepted `MapArgs`, `ValidateArgs`, and `ExtractArgs` shapes must keep
//! compiling under the `clap` and `ortho_config` derive stack, and the
//! `OrthoConfig` derive must keep rejecting two fields whose clap argument
//! identifiers normalize to the same Fluent identifier. `trybuild` compiles
//! one fixture per contract; the compile-fail expectation pins the diagnostic
//! shape for the `ortho_config` 0.11.0 and `trybuild` 1.0.125 versions named in
//! the fixture, not a public error contract.

#[test]
fn the_declaration_derives_keep_their_compile_time_contracts() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/cli/ui/pass/declarations.rs");
    tests.compile_fail("tests/cli/ui/fail/duplicate_identifier.rs");
}

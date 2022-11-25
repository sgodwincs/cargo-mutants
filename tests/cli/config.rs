// Copyright 2022 Martin Pool.

//! Test handling of `mutants.toml` configuration.

use std::fs::{create_dir, write};

use predicates::prelude::*;
use tempfile::TempDir;

use super::{copy_of_testdata, run_assert_cmd};

fn write_config_file(tempdir: &TempDir, config: &str) {
    let path = tempdir.path();
    // This will error if it exists, which today it never will,
    // but perhaps later we should ignore that.
    create_dir(path.join(".cargo")).unwrap();
    write(path.join(".cargo/mutants.toml"), config.as_bytes()).unwrap();
}

#[test]
fn invalid_toml_rejected() {
    let testdata = copy_of_testdata("well_tested");
    write_config_file(
        &testdata,
        r#"what even is this?
        "#,
    );
    run_assert_cmd()
        .args(["mutants", "--list-files", "-d"])
        .arg(testdata.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error: parse toml from "));
}

#[test]
fn invalid_field_rejected() {
    let testdata = copy_of_testdata("well_tested");
    write_config_file(
        &testdata,
        r#"wobble = false
        "#,
    );
    run_assert_cmd()
        .args(["mutants", "--list-files", "-d"])
        .arg(testdata.path())
        .assert()
        .failure()
        .stderr(
            predicates::str::contains("Error: parse toml from ")
                .and(predicates::str::contains("unknown field `wobble`")),
        );
}

#[test]
fn list_with_config_file_exclusion() {
    let testdata = copy_of_testdata("well_tested");
    write_config_file(
        &testdata,
        r#"exclude_globs = ["src/*_mod.rs"]
        "#,
    );
    run_assert_cmd()
        .args(["mutants", "--list-files", "-d"])
        .arg(testdata.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("_mod.rs").not());
    run_assert_cmd()
        .args(["mutants", "--list", "-d"])
        .arg(testdata.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("_mod.rs").not());
}

#[test]
fn list_with_config_file_inclusion() {
    let testdata = copy_of_testdata("well_tested");
    write_config_file(
        &testdata,
        r#"examine_globs = ["src/*_mod.rs"]
        "#,
    );
    run_assert_cmd()
        .args(["mutants", "--list-files", "-d"])
        .arg(testdata.path())
        .assert()
        .success()
        .stdout(predicates::str::diff(
            "src/inside_mod.rs
src/item_mod.rs\n",
        ));
    run_assert_cmd()
        .args(["mutants", "--list", "-d"])
        .arg(testdata.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("simple_fns.rs").not());
}
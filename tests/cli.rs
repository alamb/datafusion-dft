// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Tests for the CLI (e.g. run from files)

use assert_cmd::Command;
use std::path::PathBuf;

mod util;

#[test]
fn test_help() {
    let assert = Command::cargo_bin("dft")
        .unwrap()
        .arg("--help")
        .assert()
        .success();

    assert.stdout(util::contains_str("dft"));
}

#[test]
#[ignore]
fn test_logging() {
    // currently fails with
    // Error: Device not configured (os error 6)
    let assert = Command::cargo_bin("dft")
        .unwrap()
        .env("RUST_LOG", "info")
        .assert()
        .success();

    assert.stdout(util::contains_str("INFO"));
}

#[test]
fn test_command_in_file() {
    let expected = r##"
+---------------------+
| Int64(1) + Int64(1) |
+---------------------+
| 2                   |
+---------------------+
    "##;

    let file = util::sql_in_file("SELECT 1 + 1");
    util::assert_output_contains(vec![file], expected);

    // same test but with a semicolon at the end
    let file = util::sql_in_file("SELECT 1 + 1;");
    util::assert_output_contains(vec![file], expected);
}

#[test]
fn test_multiple_commands_in_file() {
    let expected = r##"
+---------+
| column1 |
+---------+
| 42      |
+---------+
+------------------------+
| foo.column1 + Int64(2) |
+------------------------+
| 44                     |
+------------------------+
    "##;

    let sql = r#"
-- The first line is a comment
CREATE TABLE foo as values (42);
-- lets ignore some whitespace

    SELECT column1 FROM foo;

-- Another comment
SELECT column1 + 2 FROM foo
    "#;

    let file = util::sql_in_file(sql);
    util::assert_output_contains(vec![file], expected);

    // same test but with a semicolon at the end of second command
    let file = util::sql_in_file(format!("{sql};"));
    util::assert_output_contains(vec![file], expected);
}

#[test]
fn test_multiple_commands_in_multiple_files() {
    let expected = r##"
+---------------------+
| Int64(1) + Int64(2) |
+---------------------+
| 3                   |
+---------------------+
+----------+
| Int64(1) |
+----------+
| 1        |
+----------+
+----------+
| Int64(2) |
+----------+
| 2        |
+----------+
    "##;

    let file1 = util::sql_in_file("SELECT 1 + 2");
    let file2 = util::sql_in_file("SELECT 1;\nselect 2;");
    util::assert_output_contains(vec![file1, file2], expected);
}

#[test]
fn test_non_existent_file() {
    let file = util::sql_in_file("SELECT 1 + 1");
    let p = PathBuf::from(file.path());
    // dropping the file makes it non existent
    drop(file);

    let assert = Command::cargo_bin("dft")
        .unwrap()
        .arg("-f")
        .arg(&p)
        .assert()
        .failure();

    let expected = format!("File does not exist: '{}'", p.to_string_lossy());
    assert.code(2).stderr(util::contains_str(&expected));
}

#[test]
fn test_one_existent_and_one_non_existent_file() {
    let file1 = util::sql_in_file("SELECT 1 + 1");
    let file2 = util::sql_in_file("SELECT 3 + 4");
    let p1 = PathBuf::from(file1.path());
    let p2 = PathBuf::from(file2.path());
    // dropping the file makes it non existent
    drop(file2);

    let assert = Command::cargo_bin("dft")
        .unwrap()
        .arg("-f")
        .arg(p1)
        .arg("-f")
        .arg(&p2)
        .assert()
        .failure();

    let expected_err = format!("File does not exist: '{}'", p2.to_string_lossy());
    assert.code(2).stderr(util::contains_str(&expected_err));
}

#[test]
fn test_sql_err_in_file() {
    let file = util::sql_in_file("SELECT this is not valid SQL");

    let assert = Command::cargo_bin("dft")
        .unwrap()
        .arg("-f")
        .arg(file.path())
        .assert()
        .failure();

    let expected_err =
        "Expected: [NOT] NULL or TRUE|FALSE or [NOT] DISTINCT FROM after IS, found: not";
    assert.code(101).stderr(util::contains_str(expected_err));
}

#[test]
fn test_sql_err_in_file_after_first() {
    let file = util::sql_in_file(
        r#"
-- First line is valid SQL
SELECT 1 + 1;
-- Second line is not
SELECT this is not valid SQL
    "#,
    );

    let assert = Command::cargo_bin("dft")
        .unwrap()
        .arg("-f")
        .arg(file.path())
        .assert()
        .failure();

    let expected_err =
        "Expected: [NOT] NULL or TRUE|FALSE or [NOT] DISTINCT FROM after IS, found: not";
    assert.code(101).stderr(util::contains_str(expected_err));
}

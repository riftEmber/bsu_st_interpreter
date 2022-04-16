// Rust integration tests here

use st_interpret;
use st_interpret::prog_handle::{st_program_load, ProgContext};
use st_interpret::read_file;

/// Test lalrpop functionality
#[test]
fn example_parser() {
    assert!(st_interpret::parser_test());
}

#[test]
/// Test the ability to read in a file
fn test_open_file() {
    let a = read_file("tests/test_inputs/st_subset_1/01_Bool.st").unwrap();
    assert_eq!(a.is_empty(), false);
}

#[test]
/// Test loading in a valid program
fn test_load_program() {
    st_program_load(
        "tests/test_inputs/st_subset_1/01_Bool.st",
        ProgContext::new(),
    )
    .unwrap();
}

#[test]
/// Test loading in a valid program with functions
fn test_load_program_with_functions() {
    let context = ProgContext::new();
    let mut handle = st_program_load("tests/test_inputs/misc/09_Main_Add.st", context).unwrap();
    handle
        .context
        .get_function(String::from("SimpleAddFunction"))
        .unwrap();
}

#[test]
/// Test loading in a valid program with functions that call other functions
fn test_load_program_with_nested_functions() {
    let context = ProgContext::new();
    let mut handle =
        st_program_load("tests/test_inputs/misc/09_Main_Add_Nested.st", context).unwrap();
    handle
        .context
        .get_function(String::from("NestedAddFunction"))
        .unwrap();
    handle
        .context
        .get_function(String::from("IntIdentityFunction"))
        .unwrap();
}

#[test]
#[should_panic(expected = "Unrecognized token `Hello`")]
/// Test that loading an invalid program fails
fn test_load_invalid_program() {
    st_program_load("tests/test_inputs/misc/read_test.txt", ProgContext::new()).unwrap();
}

#[test]
/// Example test that does addition
fn it_adds_two_integration() {
    assert_eq!(4, st_interpret::lib_function_example_add(2, 2));
}

#[test]
/// Example test that does addition
fn it_adds_15_integration() {
    assert_eq!(17, st_interpret::lib_function_example_add(15, 2));
}

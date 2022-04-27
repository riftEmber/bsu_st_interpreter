# bsu_st_interpreter

[![build and test workflow status badge](https://github.com/GoWestRobotics/bsu_st_interpreter/actions/workflows/main.yml/badge.svg)](https://github.com/GoWestRobotics/bsu_st_interpreter/actions/workflows/main.yml)

BSU Capstone senior project - Structured Text Interpreter

## Dev Environment Setup
From inside st_interpret directory:

Install cargo-c into cargo for automatic C header file and build management:

```shell
$ cargo install cargo-c
```

It then can be built by calling:

```shell
$ cargo cbuild
```


Tests can be run by calling:

```shell
$ cargo ctest
```

Ref: https://crates.io/crates/cargo-c

## Built Library Location
After calling cargo cbuild, the library file can be found in the target folder.

## Rust API
#### st_program_load(filename: &str) -> InterpreterResult<ProgHandle>
Load a Structured Text program from a file.
  
Arguments: 
  filename - the name of the file to load

#### st_program_step(program_handle: &mut ProgHandle) -> InterpreterResult<bool>
Step forward one line in the program.

  Arguments:
program_handle

#### st_program_run(program_handle: &mut ProgHandle) -> InterpreterResult<()>
Run the whole program

  Arguments:
	program_handle

#### get_all_vars(program_handle: &ProgHandle) -> Iter<'\_, String, VariableInfo>
Retrieve all the currently in scope variables and return an iterator.

  Arguments:
	program_handle

#### get_var(program_handle: &ProgHandle, name: String) -> Option<&VariableInfo>
Retrieve a variable with the given name.

  Arguments:
	program_handle, 
	name - the name of the variable to retrieve 

#### update_var(program_handle: &mut ProgHandle, name: &str, new_value: VariableValue) -> InterpreterResult<()>
Change the value of a variable.

  Arguments:
program_handle,
name - the name of the variable to update,
new_value - the new value for the variable

#### add_var(program_handle: &mut ProgHandle, name: String, kind: VariableKind, value: VariableValue) -> InterpreterResult<()>
Add a new variable.

  Arguments:
program_handle,
name - the name of the variable to add,
Kind - the kind of variable,
Value - the value for the variable

## C API
#### st_program_load(filename: *const c_char) -> ProgHandlePointer
Loads a ST program from a file and returns a pointer to a program handle

  Arguments: 
filename - the name of the file to load

#### st_program_step(program_handle: &mut ProgHandlePointer) -> bool
Executes one line of a program then returns true if the program has finished execution and false otherwise.

  Arguments:
program_handle - a pointer to a program handle that was generated by st_program_load


#### st_program_run(program_handle: &mut ProgHandlePointer) 
Runs an entire ST program.

  Arguments:
program_handle - a pointer to a program handle that was generated by st_program_load

#### get_all_vars(program_handle: &ProgHandlePointer) -> Option<Box<VariableNameInfo>>
Returns a linked list of all the currently in scope variables.

  Arguments:
program_handle - a pointer to a program handle that was generated by st_program_load

get_var(program_handle: &ProgHandlePointer, name: *mut c_char) -> Option<Box<VariableNameInfo>>
Looks for a variable with the given name, and returns it if found.
Arguments:
program_handle - a pointer to a program handle that was generated by st_program_load
name - the name of the variable

#### update_var(program_handle: &mut ProgHandlePointer, name: *const c_char, value_type: *const c_char, value: *const c_char)
Updates a variable with a new value.

  Arguments:
program_handle - a pointer to a program handle that was generated by st_program_load,
name - the name of the variable,
value_type - the type of the variable,
value - the new value for the variable

#### add_var(program_handle: &mut ProgHandlePointer, name: *const c_char, kind: *const c_char, value_type: *const c_char, value: *const c_char)
Adds a new variable with the given value.

  Arguments:
program_handle - a pointer to a program handle that was generated by st_program_load,
name - the name of the variable,
kind - the kind of variable,
value_type - the type of the variable,
value - the new value for the variable

#### destroy_variable_name_info(var_name_info: Box<VariableNameInfo>)
Passes the strings in the VariableNameInfo struct back to Rust code so that the memory can be managed. This needs to be called in order to avoid a memory leak. The strings should not be freed in the C code. More information can be found here: https://doc.rust-lang.org/std/ffi/struct.CString.html#method.from_raw.

  Arguments:
var_name_info - the struct to destroy


# ISA
## Registers
A scoreboard objective called "registers" contains all registers. Scoreboard are perfect since
mathematical operations can be done between them in an x86 style (first source operand is also destination).
Registers are named T<0..5>, S<0..5>. For a total of 16 registers. The calling convention specifies how these registers should be used.

## Data Location
Data location is the closest thing we have to a pointer in real computers, it consists of a storage name, and a path.
Datapacks normally only access data in their stack frame, or their local storage, however accessing arbitrary data is
also possible.

### Global Storage
Configuration that can be shared among multiple Logicraft datapacks can be found in the data storage `lc:config`.

Data that can be shared among multiple Logicraft datapacks can be found in the data storage `lc:data`

### Local Storage
Configuration of any datapack will be stored in the data storage `lc:config/<datapackid>`

Data of any datapack will be stored in the data storage `lc:data/<datapackid>`

### Stack
The stack is represented as a field `frames:[]` in the global storage, which is an array.
It is put in the global storage to allow for future expansions to Logicraft allowing cross datapack calls.
On each function call we prepend the array with an object of the form `{locals:[[]],fname:"...",source:"..."}`.
The layout of `locals` is determined on the fly at compile time.

On function exit we remove the first item in the array and return.

## Instructions
- Add Register, Register
- Sub Register, Register
- Mul Register, Register
- Div Register, Register
- Mod Register, Register
- Tell DataLocation

# Calling convention
## Registers
| Category | Function    | Saved by |
| -------- | ----------- | -------- |
| T        | Temporaries | Caller   |
| S        | Temporaries | Callee   |

## Call Storage
Function call storage that is used to pass other parameters is found in the global management storage as a field `params:[[]]`.

## Function Call Procedure
- Caller saves any T registers it is needing in its stack frame in a new sub-field called `presaved:[]`. The layout of `presaved` is made up  for each call at compile time.
- Caller fills call storage & T registers (checkout Arguments section) with parameters.
- Pass control to callee with macro context as call storage.
- Callee creates its stack frame with initial values for local variables and proper metadata.
- Callee saves any S registers it will use in its stack frame in a new sub-field called `postsaved:[]`. Tha layout of `postsaved` is made up on the fly for each function at compile time.

## Function Return Procedure
- Callee restores any S registers it previously saved.
- Callee removes its stack frame.
- Callee sets the return value(depends on type checkout Return Value section).
- Return control to caller.
- Caller moves return value to where it wants it to be.
- Caller restores any T registers it previously saved.

## Arguments
The first 6 integer or boolean arguments are passed through T registers. Other types of arguments and subsequent integer or boolean arguments are
stored in their order of declaration in the call storage.

## Return Value
If the return value is a single integer or boolean value, it is directly returned with the `return` command, and the caller is supposed
to extract it using `execute store`.

If there are 6 or less more integer or boolean return values, they are put in T registers.

Other types of return values, and subsequent integer or boolean return values are put in the call storage in their order of declaration.

# Interface
Compiler input:
- Source Code(with file structure metadata)
- Output destination(defaults to pwd)
- Output type (folder/Zip)
- Config:
  - LC version
  - datapack name
  - datapack version

Compiler Output:
- Datapack folder/Zip

# LogiCraft
A simple, strongly typed, language that compiles to minecraft datapacks

Checkout [design](design.md)

Still in prototype phase.

# Next Iteration Plan
For this next iteration, we will use a semi global struct called Context, which will
be a centralized center of information first filled by the lexer, then filled more and used by
the parser, semifier, and codegen. The output of the lexer, parser, and semifier all only contain
IDs within.

## File
FileData will represent a source file, it contains the PathBuf representation of the path, and owns the source as a String.
A FileData can be refered to by multiple objects. FileData is immutable.

FileTable is the table that manages all FileData's. It allocates space for them, however, it only
keeps weak references, allowing the data it is referring to to be deallocated once it is no longer referenced.

FileId is a type that encapsulates a strong reference to a specific FileData. As long as a FileId exists for
a FileData, the FileData exists.

## Symbol
Symbol will be what was referred to formely as Name, it should contain only the literal
representation of a symbol. It can be referenced by multiple SymbolId's. A Symbol is immutable.

SymbolTable is the type that manages all the symbols. It can allocate space for Symbol's.
The SymbolTable however does not keep a strong reference to its data, allowing it to be deallocated if nobody is
referring to it anymore(which should not happen).

SymbolId represents a combination between a strong reference to a Symbol, a FileId, and line and column information.
This object should never be made up, and all instances of SymbolInstance should contain data such that if it is
verified in the source code, that specific symbol will be found. As long as there is at least one SymbolId the Symbol it
is referring to exists.

## Errors
There is one error enum which contains all the possible errors that could ever be returned by any
function from argument parsing to semifying.

The errors that are defined are as follows:
- InputError: Error reading input file
- OutputError: Error writing to output file
- RemoveError: Error while removing file/directory
- CreateDir: Error creating a directory
- DirExists: Expecting directory to not exist but it exists
- ConfigParse: Error while parsing config file
- Bug: Compiler bug

This list is non comprehensive, and this enum specifically we can leave its details out until implementation.

The guidelines in error management is that components should never quit due to an error, but return it in a Result, and the top level handlers that deal with the user interaction would be responsible for handling those errors.

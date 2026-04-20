# Error Handling in Rust

The Rust language provides two complementary systems for constructing / representing, propagating, reacting to, and discarding errors. These responsibilities are known as "error handling".

The first system, panic runtime and interfaces, are used to represent bugs that have been detected in the program.

The second system, Result, the error traits, and user defined types, are used to represent anticipated runtime failure modes of the program.

Result is an enumeration, which is a type that can be in one of multiple possible states. We call each possible state a variant.

Result's variants are Ok and Err. 
The Ok variant indicates the operation was successful, and it contains the successfully generated value.
The Err variant indicates the operation failed, and it contains information about how or why the operation failed.
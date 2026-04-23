#Tests
To test the SPDUs, there are two ways that this is performed

First by using the assert_eq!(left, right) which is a built-in macro.
It checks that two values are equal (it uses ==, which comes from PartialEq)
If they're equal: the test continues.
If they're not: the test fails and prints a message that includes left and right.

Second, by using assert_spdu_bytes_match_vector(...)
This is a custom-helper that checks:
- actual = the bytes the encoder produced (pdu.to_bytes()?)
It compares encoder output bytes against 
- with the reference vector = (a frozen expected byte sequence written as a hex string)
It converts the hex string into bytes, compares byte-for-byte, and if they differ it fails the test with a message with the expected and actual value in both hex and raw byte form.
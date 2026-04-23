What are SPDUs?



# How do you interface with the SPDU layer?
The recommended way to interface with the SPDU layer is:

For decoding, implement the WireDecode Trait for the SPDU enum
For encoding, implement the WireEncode Trait for the SPDU enum

When you use the Wire traits, you use the result enum, so that if there is a failure in decoding or decoding, that you can also have an error output.

# Example
## WireDecode trait
Here are an example of using the Wire Trait for interfacing with the SPDU
Implements the WireDecode trait for the SPDU enum
// The SPDU already implements the from_bytes function, so we can use that to implement the WireDecode trait.
// The trait implementations are thin wrappers: they do not add a second encoding path; they just forward to those methods so other code can depend on WireEncode/WireDecode without naming SPDU specifically.
impl WireDecode for SPDU {
    type Error = SpduError;

    fn from_wire_bytes(data: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(data)
    }
}

## WireEncode trait
// Implements the WireEncode trait for the SPDU enum
// The SPDU already implements the to_bytes function, so we can use that to implement the WireEncode trait.
// The trait implementations are thin wrappers: they do not add a second encoding path; they just forward to those methods so other code can depend on WireEncode/WireDecode without naming SPDU specifically.
impl WireEncode for SPDU {
    type Error = SpduError;

    fn to_wire_bytes(&self) -> Result<Vec<u8>, Self::Error> {
        self.to_bytes()
    }
}

pub mod wire;
pub use wire::*;

# The SPDU layer's "public boundary" is the SPDU enum and its encode/decode

# How the other layers use the SPDU layer
## How the frame layer sees the SPDU layer
The 
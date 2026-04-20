/// Variable-length **Type 3** SPDU payload: Status Reports.
///
/// CCSDS 235.1 defines Type 3 as a container for status report information; the internal
/// structure of the report content is enterprise/implementation specific. This
/// implementation preserves the raw bytes without interpreting them.
#[derive(Debug, Clone, PartialEq)]
pub struct StatusReports {
    pub raw: Vec<u8>,
}

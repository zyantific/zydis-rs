use super::*;

#[repr(C)]
pub struct RegisterContext {
    pub values: [u64; REGISTER_MAX_VALUE + 1],
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct InstructionSegments {
    /// The number of logical instruction segments.
    pub count: u8,
    pub segments: [InstructionSegmentsElement; MAX_INSTRUCTION_SEGMENT_COUNT],
}

impl<'a> IntoIterator for &'a InstructionSegments {
    type Item = &'a InstructionSegmentsElement;
    type IntoIter = slice::Iter<'a, InstructionSegmentsElement>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.segments[..self.count as usize]).into_iter()
    }
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct InstructionSegmentsElement {
    /// The type of this segment.
    pub ty: InstructionSegment,
    /// The offset relative to the start of the instruction.
    pub offset: u8,
    /// The size of the segment, in bytes.
    pub size: u8,
}

extern "C" {
    pub fn ZydisCalcAbsoluteAddress(
        instruction: *const DecodedInstruction,
        operand: *const DecodedOperand,
        runtime_address: u64,
        result_address: *mut u64,
    ) -> Status;

    pub fn ZydisCalcAbsoluteAddressEx(
        instruction: *const DecodedInstruction,
        operand: *const DecodedOperand,
        runtime_address: u64,
        register_context: *const RegisterContext,
        result_address: *mut u64,
    ) -> Status;

    pub fn ZydisGetInstructionSegments(
        instruction: *const DecodedInstruction,
        segments: *mut InstructionSegments,
    ) -> Status;
}

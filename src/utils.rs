//! Miscellaneous utility functions.

use gen::*;
use status::ZydisResult;


/// Calculates the absolute target-address of a relative instruction operand.
pub fn calc_abs_target_addr(
    instruction: &ZydisDecodedInstruction,
    operand: &ZydisDecodedOperand
) -> ZydisResult<u64> {
    unsafe {
        let mut address = 0u64;
        let status = ZydisUtilsCalcAbsoluteTargetAddress(
            instruction,
            operand,
            &mut address,
        );
        match status {
            ZYDIS_STATUS_SUCCESS => Ok(address),
            _ => Err(status)
        }
    }
}
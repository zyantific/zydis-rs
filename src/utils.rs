//! Miscellaneous utility functions.

use std::mem::uninitialized;

use gen::*;
use status::{Result, ZydisError};

impl ZydisDecodedInstruction {
    pub fn calc_absolute_target_addr(&self, operand: &ZydisDecodedOperand) -> Result<u64> {
        unsafe {
            let mut address = 0u64;
            check!(
                ZydisCalcAbsoluteAddress(self, operand, &mut address),
                address
            )
        }
    }

    pub fn get_cpu_flags_by_action(&self, action: ZydisCPUFlagAction) -> Result<ZydisCPUFlagMask> {
        unsafe {
            let mut code = uninitialized();
            check!(ZydisGetAccessedFlagsByAction(self, action, &mut code), code)
        }
    }
}

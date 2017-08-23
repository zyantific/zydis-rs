//! Miscellaneous utility functions.

use std::mem::uninitialized;

use gen::*;
use status::ZydisResult;

impl ZydisDecodedInstruction {
    pub fn calc_absolute_target_addr(&self, operand: &ZydisDecodedOperand) -> ZydisResult<u64> {
        unsafe {
            let mut address = 0u64;
            check!(
                ZydisUtilsCalcAbsoluteTargetAddress(self, operand, &mut address),
                address
            )
        }
    }

    pub fn get_cpu_flags_by_action(
        &self,
        action: ZydisCPUFlagAction,
    ) -> ZydisResult<ZydisCPUFlagMask> {
        unsafe {
            let mut code = uninitialized();
            check!(ZydisGetCPUFlagsByAction(self, action, &mut code), code)
        }
    }
}

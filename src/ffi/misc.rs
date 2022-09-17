use super::*;

#[deprecated(note = "use `StackWidth` instead")]
pub type AddressWidth = StackWidth;

#[repr(C)]
pub struct ShortString {
    pub data: *const c_char,
    pub size: u8,
}

// Zydis.h
extern "C" {
    pub fn ZydisGetVersion() -> u64;
    pub fn ZydisIsFeatureEnabled(feature: Feature) -> Status;
}

// Register.h
extern "C" {
    pub fn ZydisRegisterEncode(register_class: RegisterClass, id: u8) -> Register;
    pub fn ZydisRegisterGetId(regster: Register) -> i8;
    pub fn ZydisRegisterGetClass(register: Register) -> RegisterClass;
    pub fn ZydisRegisterGetWidth(mode: MachineMode, register: Register) -> RegisterWidth;
    pub fn ZydisRegisterGetString(register: Register) -> *const c_char;
    pub fn ZydisRegisterGetStringWrapped(register: Register) -> *const ShortString;
    pub fn ZydisRegisterGetLargestEnclosing(mode: MachineMode, reg: Register) -> Register;
    pub fn ZydisRegisterClassGetWidth(mode: MachineMode, class: RegisterClass) -> RegisterWidth;
}

// MetaInfo.h
extern "C" {
    pub fn ZydisCategoryGetString(category: InstructionCategory) -> *const c_char;
    pub fn ZydisISASetGetString(isa_set: ISASet) -> *const c_char;
    pub fn ZydisISAExtGetString(isa_ext: ISAExt) -> *const c_char;
}

// Mnemonic.h
extern "C" {
    pub fn ZydisMnemonicGetString(mnemonic: Mnemonic) -> *const c_char;
    pub fn ZydisMnemonicGetShortString(mnemonic: Mnemonic) -> *const ShortString;
}

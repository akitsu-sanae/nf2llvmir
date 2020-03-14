use llvm::core::*;
use std::collections::HashMap;
use std::sync::RwLock;

use super::*;

pub fn validate_module(module: LModule) -> Result<(), CodegenError> {
    use llvm::analysis::*;
    let mut err_msg = 0 as *mut i8;
    let buf: *mut *mut i8 = &mut err_msg;
    let ok = unsafe {
        LLVMVerifyModule(
            module,
            LLVMVerifierFailureAction::LLVMReturnStatusAction,
            buf,
        )
    };
    if ok != 0 {
        let msg_str = unsafe { CString::from_raw(err_msg).into_string().unwrap() };
        Err(CodegenError::ModuleValidation(msg_str))
    } else {
        Ok(())
    }
}

pub fn print_module(module: LModule) -> Result<String, CodegenError> {
    unsafe {
        let ir = LLVMPrintModuleToString(module);
        let len = libc::strlen(ir);
        let result = String::from_raw_parts(ir as *mut u8, len + 1, len + 1);
        Ok(result)
    }
}

pub fn add_function(module: LModule, name: &str, typ: LType) -> LValue {
    let name = cstring(name);
    unsafe { LLVMAddFunction(module, name.as_ptr(), typ) }
}

pub fn get_func_param(func: LValue, idx: usize) -> LValue {
    unsafe { LLVMGetParam(func, idx as libc::c_uint) }
}

pub fn add_entry_block(func: LValue, base: &Base) -> LBasicBlock {
    unsafe {
        let block =
            LLVMAppendBasicBlockInContext(base.context, func, b"entry\0".as_ptr() as *const _);
        LLVMPositionBuilderAtEnd(base.builder, block);
        block
    }
}

pub fn position_at_end(block: LBasicBlock, builder: LBuilder) {
    unsafe {
        LLVMPositionBuilderAtEnd(builder, block);
    }
}

pub fn insertion_block(builder: LBuilder) -> LBasicBlock {
    unsafe { LLVMGetInsertBlock(builder) }
}

pub fn append_block(prev_block: LBasicBlock, base: &Base) -> LBasicBlock {
    unsafe {
        let f = LLVMGetBasicBlockParent(LLVMGetInsertBlock(base.builder));
        let block = LLVMAppendBasicBlockInContext(base.context, f, b"\0".as_ptr() as *const _);
        LLVMMoveBasicBlockAfter(block, prev_block);
        block
    }
}

pub fn cstring(s: &str) -> CString {
    CString::new(s.as_bytes()).unwrap()
}

pub mod i_know_what_i_do {
    pub fn clear_name_counter() {
        let mut name_counter = super::NAME_COUNTER.write().unwrap();
        name_counter.clear();
    }
}

lazy_static! {
    static ref NAME_COUNTER: RwLock<HashMap<String, i32>> = RwLock::new(HashMap::new());
}

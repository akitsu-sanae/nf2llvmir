use llvm::core::*;
use std::ffi::CString;

use super::*;

pub fn validate_module(module: LModule) -> Result<(), Error> {
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
        Err(Error::Validation(msg_str))
    } else {
        Ok(())
    }
}

pub fn print_module(module: LModule) -> Result<String, Error> {
    unsafe {
        let ir = LLVMPrintModuleToString(module);
        let len = libc::strlen(ir);
        let result = String::from_raw_parts(ir as *mut u8, len + 1, len + 1);
        Ok(result)
    }
}

pub fn add_function(module: LModule, name: &str, typ: LType) -> LValue {
    let name = CString::new(name).unwrap();
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

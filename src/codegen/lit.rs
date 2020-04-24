use super::*;
use llvm::core::*;
use std::ffi::CString;

pub fn bool(b: bool, context: LContext) -> LValue {
    unsafe { LLVMConstInt(LLVMInt1TypeInContext(context), b as u64, 0) }
}

pub fn char(c: char, context: LContext) -> LValue {
    unsafe { LLVMConstInt(typ::char(context), c as u64, 0) }
}

pub fn str(str: &str, context: LContext) -> LValue {
    let mut bytes: Vec<_> = str.bytes().map(|c| char(c as char, context)).collect();
    unsafe {
        LLVMConstArray(
            typ::char(context),
            bytes.as_mut_ptr(),
            bytes.len() as libc::c_uint,
        )
    }
}

pub fn int32(n: i32, context: LContext) -> LValue {
    unsafe { LLVMConstInt(typ::int32(context), n as u64, 0) }
}

pub fn array(mut elems: Vec<LValue>, typ: LType, base: &Base) -> LValue {
    let arr_type = typ::array(typ, elems.len());
    unsafe {
        if elems.iter().all(|v| LLVMIsConstant(*v) != 0) {
            LLVMConstArray(typ, elems.as_mut_ptr(), elems.len() as u32)
        } else {
            let var = LLVMBuildAlloca(base.builder, arr_type, b"\0".as_ptr() as *const _);
            for (idx, elem) in elems.into_iter().enumerate() {
                let elem_var = build::gep(var, lit::int32(idx as i32, base.context), base);
                build::store(elem_var, elem, base.builder);
            }
            build::load(var, base.builder)
        }
    }
}

pub fn tuple(mut fields: Vec<LValue>, base: &Base) -> LValue {
    unsafe {
        if fields.iter().all(|v| LLVMIsConstant(*v) != 0) {
            LLVMConstStruct(fields.as_mut_ptr(), fields.len() as libc::c_uint, 0)
        // packed
        } else {
            let typ = typ::tuple(fields.iter().map(|v| type_of(*v)).collect());
            let var = LLVMBuildAlloca(base.builder, typ, b"\0".as_ptr() as *const _);
            for (idx, field) in fields.into_iter().enumerate() {
                let field_var = build::gep(var, lit::int32(idx as i32, base.context), base);
                build::store(field_var, field, base.builder);
            }
            build::load(var, base.builder)
        }
    }
}
pub fn external_func(name: String, typ: LType, module: LModule) -> LValue {
    let name = CString::new(name.into_bytes()).unwrap();
    unsafe { llvm::core::LLVMAddFunction(module, name.as_ptr(), typ) }
}

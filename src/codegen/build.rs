use super::*;
use llvm::core::*;
use llvm::LLVMIntPredicate;
use std::ffi::CString;

pub fn declare(name: &str, typ: LType, init: LValue, builder: LBuilder) -> LValue {
    let name = CString::new(name).unwrap();
    unsafe {
        let var = LLVMBuildAlloca(builder, typ, name.as_ptr());
        self::store(var, init, builder);
        var
    }
}

pub fn declare_array(name: &str, typ: LType, init: LValue, base: &Base) -> LValue {
    let name = CString::new(name).unwrap();
    let memcpy_name = CString::new("memcpy").unwrap();
    unsafe {
        let var = LLVMBuildAlloca(base.builder, typ, name.as_ptr());

        let var_ptr = LLVMBuildBitCast(
            base.builder,
            var,
            typ::char_ptr(base.context),
            b"\0".as_ptr() as *const _,
        );
        let init_ptr = LLVMBuildBitCast(
            base.builder,
            init,
            typ::char_ptr(base.context),
            b"\0".as_ptr() as *const _,
        );
        let len = typ::size_of(typ);

        let memcpy = LLVMGetNamedFunction(base.module, memcpy_name.as_ptr());
        let mut args = vec![var_ptr, init_ptr, len];
        LLVMBuildCall(
            base.builder,
            memcpy,
            args.as_mut_ptr(),
            args.len() as libc::c_uint,
            b"\0".as_ptr() as *const _,
        );

        var
    }
}

pub fn declare_tuple(name: &str, typ: LType, init: LValue, base: &Base) -> LValue {
    let name = CString::new(name).unwrap();
    let memcpy_name = CString::new("memcpy").unwrap();
    unsafe {
        let var = LLVMBuildAlloca(base.builder, typ, name.as_ptr());

        let var_ptr = LLVMBuildBitCast(
            base.builder,
            var,
            typ::char_ptr(base.context),
            b"\0".as_ptr() as *const _,
        );
        let init_ptr = LLVMBuildBitCast(
            base.builder,
            init,
            typ::char_ptr(base.context),
            b"\0".as_ptr() as *const _,
        );
        let len = typ::size_of(typ);

        let memcpy = LLVMGetNamedFunction(base.module, memcpy_name.as_ptr());
        let mut args = vec![var_ptr, init_ptr, len];
        LLVMBuildCall(
            base.builder,
            memcpy,
            args.as_mut_ptr(),
            args.len() as libc::c_uint,
            b"\0".as_ptr() as *const _,
        );

        var
    }
}

pub fn store(var: LValue, expr: LValue, builder: LBuilder) -> LValue {
    unsafe { LLVMBuildStore(builder, expr, var) }
}

pub fn load(var: LValue, builder: LBuilder) -> LValue {
    unsafe { LLVMBuildLoad(builder, var, b"\0".as_ptr() as *const _) }
}

pub fn ret(value: LValue, builder: LBuilder) {
    unsafe {
        LLVMBuildRet(builder, value);
    }
}

pub fn add(lhs: LValue, rhs: LValue, builder: LBuilder) -> LValue {
    unsafe { LLVMBuildAdd(builder, lhs, rhs, b"\0".as_ptr() as *const _) }
}

pub fn sub(lhs: LValue, rhs: LValue, builder: LBuilder) -> LValue {
    unsafe { LLVMBuildSub(builder, lhs, rhs, b"\0".as_ptr() as *const _) }
}

pub fn mult(lhs: LValue, rhs: LValue, builder: LBuilder) -> LValue {
    unsafe { LLVMBuildMul(builder, lhs, rhs, b"\0".as_ptr() as *const _) }
}

pub fn div(lhs: LValue, rhs: LValue, builder: LBuilder) -> LValue {
    unsafe { LLVMBuildSDiv(builder, lhs, rhs, b"\0".as_ptr() as *const _) }
}

pub fn eq(lhs: LValue, rhs: LValue, builder: LBuilder) -> LValue {
    unsafe {
        LLVMBuildICmp(
            builder,
            LLVMIntPredicate::LLVMIntEQ,
            lhs,
            rhs,
            b"\0".as_ptr() as *const _,
        )
    }
}

pub fn neq(lhs: LValue, rhs: LValue, builder: LBuilder) -> LValue {
    unsafe {
        LLVMBuildICmp(
            builder,
            LLVMIntPredicate::LLVMIntNE,
            lhs,
            rhs,
            b"\0".as_ptr() as *const _,
        )
    }
}

pub fn gt(lhs: LValue, rhs: LValue, builder: LBuilder) -> LValue {
    unsafe {
        LLVMBuildICmp(
            builder,
            LLVMIntPredicate::LLVMIntSGT, // assume signed
            lhs,
            rhs,
            b"\0".as_ptr() as *const _,
        )
    }
}

pub fn geq(lhs: LValue, rhs: LValue, builder: LBuilder) -> LValue {
    unsafe {
        LLVMBuildICmp(
            builder,
            LLVMIntPredicate::LLVMIntSGE, // assume signed
            lhs,
            rhs,
            b"\0".as_ptr() as *const _,
        )
    }
}

pub fn lt(lhs: LValue, rhs: LValue, builder: LBuilder) -> LValue {
    unsafe {
        LLVMBuildICmp(
            builder,
            LLVMIntPredicate::LLVMIntSLT, // assume signed
            lhs,
            rhs,
            b"\0".as_ptr() as *const _,
        )
    }
}

pub fn leq(lhs: LValue, rhs: LValue, builder: LBuilder) -> LValue {
    unsafe {
        LLVMBuildICmp(
            builder,
            LLVMIntPredicate::LLVMIntSLE, // assume signed
            lhs,
            rhs,
            b"\0".as_ptr() as *const _,
        )
    }
}

pub fn branch(block: LBasicBlock, builder: LBuilder) {
    unsafe {
        LLVMBuildBr(builder, block);
    }
}

pub fn cond_branch(
    cond: LValue,
    then: LBasicBlock,
    else_: LBasicBlock,
    builder: LBuilder,
) -> LValue {
    unsafe { LLVMBuildCondBr(builder, cond, then, else_) }
}

pub fn phi(typ: LType, incoming: Vec<(LValue, LBasicBlock)>, builder: LBuilder) -> LValue {
    let len = incoming.len();
    let (mut values, mut blocks): (Vec<LValue>, Vec<LBasicBlock>) = incoming.into_iter().unzip();
    unsafe {
        let phi = LLVMBuildPhi(builder, typ, b"\0".as_ptr() as *const _);
        LLVMAddIncoming(
            phi,
            values.as_mut_ptr(),
            blocks.as_mut_ptr(),
            len as libc::c_uint,
        );
        phi
    }
}

pub fn call(func: LValue, args: &mut Vec<LValue>, builder: LBuilder) -> LValue {
    let len = args.len() as libc::c_uint;
    unsafe {
        LLVMBuildCall(
            builder,
            func,
            args.as_mut_ptr(),
            len,
            b"\0".as_ptr() as *const _,
        )
    }
}

pub fn gep(arr: LValue, idx: LValue, base: &Base) -> LValue {
    let mut indices = vec![lit::int32(0, base.context), idx];
    unsafe {
        LLVMBuildGEP(
            base.builder,
            arr,
            indices.as_mut_ptr(),
            indices.len() as libc::c_uint,
            b"\0".as_ptr() as *const _,
        )
    }
}

pub mod builtin {
    use super::*;
    pub fn print_num(value: LValue, base: &Base) -> LValue {
        unsafe {
            let printf_name = CString::new("printf").unwrap();
            let printf = LLVMGetNamedFunction(base.module, printf_name.as_ptr());

            let format = LLVMGetNamedGlobal(
                base.module,
                CString::new(".builtin.format.num").unwrap().as_ptr(),
            );
            let format_ptr_name = CString::new("format_ptr").unwrap();
            let format_ptr = LLVMBuildBitCast(
                base.builder,
                format,
                typ::char_ptr(base.context),
                format_ptr_name.as_ptr(),
            );
            let mut args = vec![format_ptr, value];
            call(printf, &mut args, base.builder)
        }
    }
}

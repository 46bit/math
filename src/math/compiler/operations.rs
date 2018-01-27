use super::*;
use std::ffi::CString;
use llvm::prelude::*;
use llvm::core::{LLVMBuildCall, LLVMBuildGEP, LLVMBuildGlobalStringPtr, LLVMBuildLoad,
                 LLVMConstInt, LLVMGetNamedFunction, LLVMInt64TypeInContext};

pub unsafe fn llvm_global_string_ptr(
    builder: LLVMBuilderRef,
    name: CString,
    string: CString,
) -> LLVMValueRef {
    LLVMBuildGlobalStringPtr(builder, string.as_ptr(), name.as_ptr())
}

pub unsafe fn llvm_getelement(
    ctx: LLVMContextRef,
    builder: LLVMBuilderRef,
    array: LLVMValueRef,
    index: u64,
    name: CString,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let index = &mut [LLVMConstInt(i64_type, index, 0)];
    LLVMBuildGEP(builder, array, index.as_mut_ptr(), 1, name.as_ptr())
}

pub unsafe fn llvm_load(
    builder: LLVMBuilderRef,
    from: LLVMValueRef,
    name: CString,
) -> LLVMValueRef {
    LLVMBuildLoad(builder, from, name.as_ptr())
}

pub unsafe fn llvm_call(
    builder: LLVMBuilderRef,
    function: LLVMValueRef,
    args: &mut [LLVMValueRef],
    name: CString,
) -> LLVMValueRef {
    LLVMBuildCall(
        builder,
        function,
        args.as_mut_ptr(),
        args.len() as u32,
        name.as_ptr(),
    )
}

pub unsafe fn llvm_sscanf(
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    string: LLVMValueRef,
    template: LLVMValueRef,
    destination: LLVMValueRef,
    name: CString,
) -> LLVMValueRef {
    let sscanf_name = llvm_name("sscanf");
    let sscanf_fn = LLVMGetNamedFunction(module, sscanf_name.as_ptr());
    let args = &mut [string, template, destination];
    llvm_call(builder, sscanf_fn, args, name)
}

pub unsafe fn llvm_printf(
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    template: LLVMValueRef,
    output: LLVMValueRef,
    name: CString,
) -> LLVMValueRef {
    let printf_name = llvm_name("printf");
    let printf_fn = LLVMGetNamedFunction(module, printf_name.as_ptr());
    let args = &mut [template, output];
    llvm_call(builder, printf_fn, args, name)
}

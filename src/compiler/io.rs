use super::*;
use std::ffi::CString;
use llvm::prelude::*;
use llvm::core::*;

pub unsafe fn define_sscanf(ctx: LLVMContextRef, module: LLVMModuleRef) -> LLVMValueRef {
    let i32_type = LLVMInt32TypeInContext(ctx);
    let i64_type = LLVMInt64TypeInContext(ctx);
    let i64_ptr_type = LLVMPointerType(i64_type, 0);
    let string_type = LLVMPointerType(LLVMInt8Type(), 0);

    let fn_name = llvm_name("sscanf");
    let param_types = &mut [string_type, string_type, i64_ptr_type];
    let fn_type = assert_not_nil(LLVMFunctionType(i32_type, param_types.as_mut_ptr(), 3, 1));
    assert_not_nil(LLVMAddFunction(module, fn_name.as_ptr(), fn_type))
}

pub unsafe fn sscanf(
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    string: LLVMValueRef,
    template: LLVMValueRef,
    destination: LLVMValueRef,
    name: CString,
) -> LLVMValueRef {
    let sscanf_name = llvm_name("sscanf");
    let sscanf_fn = assert_not_nil(LLVMGetNamedFunction(module, sscanf_name.as_ptr()));
    let args = &mut [string, template, destination];
    function_call(builder, sscanf_fn, args, name)
}

pub unsafe fn define_printf(ctx: LLVMContextRef, module: LLVMModuleRef) -> LLVMValueRef {
    let i32_type = LLVMInt32TypeInContext(ctx);
    let i64_type = LLVMInt64TypeInContext(ctx);
    let string_type = LLVMPointerType(LLVMInt8Type(), 0);

    let fn_name = llvm_name("printf");
    let param_types = &mut [string_type, i64_type];
    let fn_type = assert_not_nil(LLVMFunctionType(i32_type, param_types.as_mut_ptr(), 2, 1));
    assert_not_nil(LLVMAddFunction(module, fn_name.as_ptr(), fn_type))
}

pub unsafe fn printf(
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    template: LLVMValueRef,
    output: LLVMValueRef,
    name: CString,
) -> LLVMValueRef {
    let printf_name = llvm_name("printf");
    let printf_fn = assert_not_nil(LLVMGetNamedFunction(module, printf_name.as_ptr()));
    let args = &mut [template, output];
    function_call(builder, printf_fn, args, name)
}

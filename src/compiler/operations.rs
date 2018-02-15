use super::*;
use std::ffi::CString;
use llvm::prelude::*;
use llvm::core::*;

pub unsafe fn global_string_ptr(
    builder: LLVMBuilderRef,
    name: CString,
    string: CString,
) -> LLVMValueRef {
    assert_not_nil(LLVMBuildGlobalStringPtr(
        builder,
        string.as_ptr(),
        name.as_ptr(),
    ))
}

pub unsafe fn getelementptr(
    ctx: LLVMContextRef,
    builder: LLVMBuilderRef,
    array: LLVMValueRef,
    index: u64,
    name: CString,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let index = &mut [LLVMConstInt(i64_type, index, 0)];
    assert_not_nil(LLVMBuildGEP(
        builder,
        array,
        index.as_mut_ptr(),
        1,
        name.as_ptr(),
    ))
}

pub unsafe fn getelement(
    ctx: LLVMContextRef,
    builder: LLVMBuilderRef,
    array: LLVMValueRef,
    index: u64,
    name: CString,
) -> LLVMValueRef {
    let ptr_name = llvm_name(&format!("{}_ptr", name.clone().into_string().unwrap()));
    let ptr = getelementptr(ctx, builder, array, index, ptr_name);
    load(builder, ptr, name)
}

pub unsafe fn allocate(builder: LLVMBuilderRef, t: LLVMTypeRef, name: CString) -> LLVMValueRef {
    assert_not_nil(LLVMBuildAlloca(builder, t, name.as_ptr()))
}

pub unsafe fn load(builder: LLVMBuilderRef, from: LLVMValueRef, name: CString) -> LLVMValueRef {
    assert_not_nil(LLVMBuildLoad(builder, from, name.as_ptr()))
}

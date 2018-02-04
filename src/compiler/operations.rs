use super::*;
use std::ffi::CString;
use llvm::prelude::*;
use llvm::core::*;
use llvm::LLVMIntPredicate;

pub unsafe fn global_string_ptr(
    builder: LLVMBuilderRef,
    name: CString,
    string: CString,
) -> LLVMValueRef {
    LLVMBuildGlobalStringPtr(builder, string.as_ptr(), name.as_ptr())
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
    LLVMBuildGEP(builder, array, index.as_mut_ptr(), 1, name.as_ptr())
}

pub unsafe fn allocate(builder: LLVMBuilderRef, t: LLVMTypeRef, name: CString) -> LLVMValueRef {
    LLVMBuildAlloca(builder, t, name.as_ptr())
}

pub unsafe fn load(builder: LLVMBuilderRef, from: LLVMValueRef, name: CString) -> LLVMValueRef {
    LLVMBuildLoad(builder, from, name.as_ptr())
}

pub unsafe fn call(
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

pub unsafe fn define_saturating_div(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);

    let fn_name = llvm_name("saturating_div");
    let param_types = vec![
        (Name::new("numerator"), i64_type),
        (Name::new("denominator"), i64_type),
    ];
    let (function, param_values) = function_definition(module, fn_name, param_types, i64_type);
    let numerator = param_values[&Name::new("numerator")];
    let denominator = param_values[&Name::new("denominator")];

    let name = llvm_name("entry");
    let entry_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());
    let name = llvm_name("then");
    let then_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());
    let name = llvm_name("else");
    let else_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());

    LLVMPositionBuilderAtEnd(builder, entry_block);
    let cmp_name = llvm_name("do_not_divide_by_zero_cmp");
    let cmp = LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntNE,
        denominator,
        LLVMConstInt(i64_type, 0, 0),
        cmp_name.as_ptr(),
    );
    LLVMBuildCondBr(builder, cmp, then_block, else_block);

    LLVMPositionBuilderAtEnd(builder, then_block);
    let llvm_name = llvm_name("tmp_div");
    let sdiv = LLVMBuildSDiv(builder, numerator, denominator, llvm_name.as_ptr());
    function_return(builder, sdiv);

    LLVMPositionBuilderAtEnd(builder, else_block);
    function_return(builder, LLVMConstInt(i64_type, i64::max_value() as u64, 0));

    function
}

pub unsafe fn saturating_div(
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    numerator: LLVMValueRef,
    denominator: LLVMValueRef,
    name: CString,
) -> LLVMValueRef {
    let saturating_div_name = llvm_name("saturating_div");
    let saturating_div_fn = LLVMGetNamedFunction(module, saturating_div_name.as_ptr());
    let args = &mut [numerator, denominator];
    call(builder, saturating_div_fn, args, name)
}

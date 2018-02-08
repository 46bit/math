use super::*;
use std::ffi::CString;
use llvm::prelude::*;
use llvm::core::*;

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

pub unsafe fn switch(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    function: LLVMValueRef,
    source: LLVMValueRef,
    matchers: Vec<(Matcher, Expression)>,
    vars: &HashMap<Name, LLVMValueRef>,
    functions: &HashMap<Name, LLVMValueRef>,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);

    // Make a new block
    let matcher_blocks: Vec<LLVMBasicBlockRef> = matchers
        .iter()
        .map(|_| {
            let name = llvm_name("match");
            LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr())
        })
        .collect();
    let name = llvm_name("match_none");
    let unmatched_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());
    let name = llvm_name("match_final");
    let final_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());

    let name = llvm_name("match_dest");
    let dest = allocate(builder, i64_type, name);

    // Jump to the new block
    // Make one new block for each flat_matcher

    let switch = LLVMBuildSwitch(builder, source, unmatched_block, matchers.len() as u32);
    for (i, &(matcher, _)) in matchers.iter().enumerate() {
        match matcher {
            Matcher::Value(value) => {
                let value_const = LLVMConstInt(i64_type, value as u64, 0);
                LLVMAddCase(switch, value_const, matcher_blocks[i])
            }
        }
    }

    for (i, (_, expression)) in matchers.into_iter().enumerate() {
        LLVMPositionBuilderAtEnd(builder, matcher_blocks[i]);
        let value = synthesise_expression(ctx, module, builder, &expression, &vars, &functions);
        LLVMBuildStore(builder, value, dest);
        LLVMBuildBr(builder, final_block);
    }

    LLVMPositionBuilderAtEnd(builder, unmatched_block);
    LLVMBuildUnreachable(builder);

    LLVMPositionBuilderAtEnd(builder, final_block);
    dest
}

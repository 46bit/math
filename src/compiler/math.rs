use super::*;
use llvm::prelude::*;
use llvm::core::*;
use llvm::LLVMIntPredicate;

pub unsafe fn define_saturating_add(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
) -> LLVMValueRef {
    let i1_type = LLVMInt1TypeInContext(ctx);
    let i64_type = LLVMInt64TypeInContext(ctx);
    let agg_types = &mut [i64_type, i1_type];
    let agg_type = assert_not_nil(LLVMStructType(agg_types.as_mut_ptr(), 2, 0));

    let fn_name = llvm_name("llvm.sadd.with.overflow.i64");
    let param_types = &mut [i64_type, i64_type];
    let fn_type = assert_not_nil(LLVMFunctionType(agg_type, param_types.as_mut_ptr(), 2, 0));
    let sadd_overflow = assert_not_nil(LLVMAddFunction(module, fn_name.as_ptr(), fn_type));

    let fn_name = llvm_name("saturating_add");
    let param_types = vec![(Name::new("lhs"), i64_type), (Name::new("rhs"), i64_type)];
    let (function, param_values) = function_definition(module, fn_name, param_types, i64_type);
    let lhs = assert_not_nil(param_values[&Name::new("lhs")]);
    let rhs = assert_not_nil(param_values[&Name::new("rhs")]);

    let name = llvm_name("entry");
    let entry_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("ok");
    let ok_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate");
    let saturate_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate_max");
    let saturate_max_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate_min");
    let saturate_min_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));

    LLVMPositionBuilderAtEnd(builder, entry_block);
    let result = function_call(
        builder,
        sadd_overflow,
        &mut [lhs, rhs],
        llvm_name("tmp_sadd_overflow"),
    );
    let value_name = llvm_name("value");
    let value = assert_not_nil(LLVMBuildExtractValue(
        builder,
        result,
        0,
        value_name.as_ptr(),
    ));
    let overflowed_name = llvm_name("overflowed");
    let overflowed = assert_not_nil(LLVMBuildExtractValue(
        builder,
        result,
        1,
        overflowed_name.as_ptr(),
    ));
    let cmp_name = llvm_name("cmp");
    let overflow_cmp = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntNE,
        overflowed,
        LLVMConstInt(i1_type, 0, 0),
        cmp_name.as_ptr(),
    ));
    assert_not_nil(LLVMBuildCondBr(
        builder,
        overflow_cmp,
        saturate_block,
        ok_block,
    ));

    LLVMPositionBuilderAtEnd(builder, ok_block);
    function_return(builder, value);

    LLVMPositionBuilderAtEnd(builder, saturate_block);
    let cmp_name = llvm_name("cmp");
    let saturate_cmp = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntSGE,
        rhs,
        LLVMConstInt(i64_type, 0, 0),
        cmp_name.as_ptr(),
    ));
    assert_not_nil(LLVMBuildCondBr(
        builder,
        saturate_cmp,
        saturate_max_block,
        saturate_min_block,
    ));

    LLVMPositionBuilderAtEnd(builder, saturate_max_block);
    function_return(builder, LLVMConstInt(i64_type, i64::max_value() as u64, 0));

    LLVMPositionBuilderAtEnd(builder, saturate_min_block);
    function_return(builder, LLVMConstInt(i64_type, i64::min_value() as u64, 0));

    function
}

pub unsafe fn define_saturating_sub(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
) -> LLVMValueRef {
    let i1_type = LLVMInt1TypeInContext(ctx);
    let i64_type = LLVMInt64TypeInContext(ctx);
    let agg_types = &mut [i64_type, i1_type];
    let agg_type = LLVMStructType(agg_types.as_mut_ptr(), 2, 0);

    let fn_name = llvm_name("llvm.ssub.with.overflow.i64");
    let param_types = &mut [i64_type, i64_type];
    let fn_type = assert_not_nil(LLVMFunctionType(agg_type, param_types.as_mut_ptr(), 2, 0));
    let sadd_overflow = assert_not_nil(LLVMAddFunction(module, fn_name.as_ptr(), fn_type));

    let fn_name = llvm_name("saturating_sub");
    let param_types = vec![(Name::new("lhs"), i64_type), (Name::new("rhs"), i64_type)];
    let (function, param_values) = function_definition(module, fn_name, param_types, i64_type);
    let lhs = assert_not_nil(param_values[&Name::new("lhs")]);
    let rhs = assert_not_nil(param_values[&Name::new("rhs")]);

    let name = llvm_name("entry");
    let entry_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("ok");
    let ok_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate");
    let saturate_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate_min");
    let saturate_min_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate_max");
    let saturate_max_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));

    LLVMPositionBuilderAtEnd(builder, entry_block);
    let result = function_call(
        builder,
        sadd_overflow,
        &mut [lhs, rhs],
        llvm_name("tmp_sadd_overflow"),
    );
    let value_name = llvm_name("value");
    let value = assert_not_nil(LLVMBuildExtractValue(
        builder,
        result,
        0,
        value_name.as_ptr(),
    ));
    let overflowed_name = llvm_name("overflowed");
    let overflowed = assert_not_nil(LLVMBuildExtractValue(
        builder,
        result,
        1,
        overflowed_name.as_ptr(),
    ));
    let cmp_name = llvm_name("cmp");
    let overflow_cmp = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntNE,
        overflowed,
        LLVMConstInt(i1_type, 0, 0),
        cmp_name.as_ptr(),
    ));
    assert_not_nil(LLVMBuildCondBr(
        builder,
        overflow_cmp,
        saturate_block,
        ok_block,
    ));

    LLVMPositionBuilderAtEnd(builder, ok_block);
    function_return(builder, value);

    LLVMPositionBuilderAtEnd(builder, saturate_block);
    let cmp_name = llvm_name("cmp");
    let saturate_cmp = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntSGE,
        rhs,
        LLVMConstInt(i64_type, 0, 0),
        cmp_name.as_ptr(),
    ));
    assert_not_nil(LLVMBuildCondBr(
        builder,
        saturate_cmp,
        saturate_min_block,
        saturate_max_block,
    ));

    LLVMPositionBuilderAtEnd(builder, saturate_min_block);
    function_return(builder, LLVMConstInt(i64_type, i64::min_value() as u64, 0));

    LLVMPositionBuilderAtEnd(builder, saturate_max_block);
    function_return(builder, LLVMConstInt(i64_type, i64::max_value() as u64, 0));

    function
}

pub unsafe fn define_saturating_mul(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
) -> LLVMValueRef {
    let i1_type = LLVMInt1TypeInContext(ctx);
    let i64_type = LLVMInt64TypeInContext(ctx);
    let agg_types = &mut [i64_type, i1_type];
    let agg_type = LLVMStructType(agg_types.as_mut_ptr(), 2, 0);

    let fn_name = llvm_name("llvm.smul.with.overflow.i64");
    let param_types = &mut [i64_type, i64_type];
    let fn_type = assert_not_nil(LLVMFunctionType(agg_type, param_types.as_mut_ptr(), 2, 0));
    let sadd_overflow = assert_not_nil(LLVMAddFunction(module, fn_name.as_ptr(), fn_type));

    let fn_name = llvm_name("saturating_mul");
    let param_types = vec![(Name::new("lhs"), i64_type), (Name::new("rhs"), i64_type)];
    let (function, param_values) = function_definition(module, fn_name, param_types, i64_type);
    let lhs = assert_not_nil(param_values[&Name::new("lhs")]);
    let rhs = assert_not_nil(param_values[&Name::new("rhs")]);

    let name = llvm_name("entry");
    let entry_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("ok");
    let ok_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate");
    let saturate_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate_max");
    let saturate_max_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate_min");
    let saturate_min_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));

    LLVMPositionBuilderAtEnd(builder, entry_block);
    let result = function_call(
        builder,
        sadd_overflow,
        &mut [lhs, rhs],
        llvm_name("tmp_sadd_overflow"),
    );
    let value_name = llvm_name("value");
    let value = assert_not_nil(LLVMBuildExtractValue(
        builder,
        result,
        0,
        value_name.as_ptr(),
    ));
    let overflowed_name = llvm_name("overflowed");
    let overflowed = assert_not_nil(LLVMBuildExtractValue(
        builder,
        result,
        1,
        overflowed_name.as_ptr(),
    ));
    let cmp_name = llvm_name("cmp");
    let overflow_cmp = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntNE,
        overflowed,
        LLVMConstInt(i1_type, 0, 0),
        cmp_name.as_ptr(),
    ));
    assert_not_nil(LLVMBuildCondBr(
        builder,
        overflow_cmp,
        saturate_block,
        ok_block,
    ));

    LLVMPositionBuilderAtEnd(builder, ok_block);
    function_return(builder, value);

    LLVMPositionBuilderAtEnd(builder, saturate_block);
    let lhs_neg_cmp_name = llvm_name("lhs_neg_cmp");
    let lhs_neg = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntSLT,
        lhs,
        LLVMConstInt(i64_type, 0, 0),
        lhs_neg_cmp_name.as_ptr(),
    ));
    let rhs_neg_cmp_name = llvm_name("rhs_neg_cmp");
    let rhs_neg = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntSLT,
        rhs,
        LLVMConstInt(i64_type, 0, 0),
        rhs_neg_cmp_name.as_ptr(),
    ));
    let cmp_name = llvm_name("cmp");
    let saturate_cmp = assert_not_nil(LLVMBuildXor(builder, lhs_neg, rhs_neg, cmp_name.as_ptr()));
    assert_not_nil(LLVMBuildCondBr(
        builder,
        saturate_cmp,
        saturate_min_block,
        saturate_max_block,
    ));

    LLVMPositionBuilderAtEnd(builder, saturate_max_block);
    function_return(builder, LLVMConstInt(i64_type, i64::max_value() as u64, 0));

    LLVMPositionBuilderAtEnd(builder, saturate_min_block);
    function_return(builder, LLVMConstInt(i64_type, i64::min_value() as u64, 0));

    function
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
    let numerator = assert_not_nil(param_values[&Name::new("numerator")]);
    let denominator = assert_not_nil(param_values[&Name::new("denominator")]);

    let name = llvm_name("entry");
    let entry_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("ok");
    let ok_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate");
    let saturate_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate_min");
    let saturate_min_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("saturate_max");
    let saturate_max_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));

    LLVMPositionBuilderAtEnd(builder, entry_block);
    let zero_cmp_name = llvm_name("denominator_is_zero");
    let zero_cmp = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntEQ,
        denominator,
        LLVMConstInt(i64_type, 0, 0),
        zero_cmp_name.as_ptr(),
    ));
    let lhs_is_min_cmp_name = llvm_name("numerator_is_min");
    let lhs_is_min_cmp = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntEQ,
        numerator,
        LLVMConstInt(i64_type, i64::min_value() as u64, 0),
        lhs_is_min_cmp_name.as_ptr(),
    ));
    let rhs_is_minus1_cmp_name = llvm_name("denominator_is_minus_1");
    let rhs_is_minus1_cmp = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntEQ,
        denominator,
        LLVMConstInt(i64_type, (-1i64) as u64, 0),
        rhs_is_minus1_cmp_name.as_ptr(),
    ));
    let overflow_from_min_cmp_name = llvm_name("wrapping");
    let overflow_from_min_cmp = assert_not_nil(LLVMBuildAnd(
        builder,
        lhs_is_min_cmp,
        rhs_is_minus1_cmp,
        overflow_from_min_cmp_name.as_ptr(),
    ));
    let cmp_name = llvm_name("overflowing");
    let cmp = assert_not_nil(LLVMBuildOr(
        builder,
        zero_cmp,
        overflow_from_min_cmp,
        cmp_name.as_ptr(),
    ));
    assert_not_nil(LLVMBuildCondBr(builder, cmp, saturate_block, ok_block));

    LLVMPositionBuilderAtEnd(builder, ok_block);
    let name = llvm_name("div");
    let sdiv = assert_not_nil(LLVMBuildSDiv(
        builder,
        numerator,
        denominator,
        name.as_ptr(),
    ));
    function_return(builder, sdiv);

    LLVMPositionBuilderAtEnd(builder, saturate_block);
    let lhs_neg_cmp_name = llvm_name("numerator_is_negative");
    let lhs_neg = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntSLT,
        numerator,
        LLVMConstInt(i64_type, 0, 0),
        lhs_neg_cmp_name.as_ptr(),
    ));
    let rhs_neg_cmp_name = llvm_name("denominator_is_negative");
    let rhs_neg = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntSLT,
        denominator,
        LLVMConstInt(i64_type, 0, 0),
        rhs_neg_cmp_name.as_ptr(),
    ));
    let cmp_name = llvm_name("saturating_to_min");
    let saturate_cmp = assert_not_nil(LLVMBuildXor(builder, lhs_neg, rhs_neg, cmp_name.as_ptr()));
    assert_not_nil(LLVMBuildCondBr(
        builder,
        saturate_cmp,
        saturate_min_block,
        saturate_max_block,
    ));

    LLVMPositionBuilderAtEnd(builder, saturate_min_block);
    function_return(builder, LLVMConstInt(i64_type, i64::min_value() as u64, 0));

    LLVMPositionBuilderAtEnd(builder, saturate_max_block);
    function_return(builder, LLVMConstInt(i64_type, i64::max_value() as u64, 0));

    function
}

pub unsafe fn saturating_add(
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    lhs: LLVMValueRef,
    rhs: LLVMValueRef,
    name: CString,
) -> LLVMValueRef {
    let saturating_add_name = llvm_name("saturating_add");
    let saturating_add_fn =
        assert_not_nil(LLVMGetNamedFunction(module, saturating_add_name.as_ptr()));
    let args = &mut [lhs, rhs];
    function_call(builder, saturating_add_fn, args, name)
}

pub unsafe fn saturating_sub(
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    lhs: LLVMValueRef,
    rhs: LLVMValueRef,
    name: CString,
) -> LLVMValueRef {
    let saturating_sub_name = llvm_name("saturating_sub");
    let saturating_sub_fn =
        assert_not_nil(LLVMGetNamedFunction(module, saturating_sub_name.as_ptr()));
    let args = &mut [lhs, rhs];
    function_call(builder, saturating_sub_fn, args, name)
}

pub unsafe fn saturating_mul(
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    lhs: LLVMValueRef,
    rhs: LLVMValueRef,
    name: CString,
) -> LLVMValueRef {
    let saturating_mul_name = llvm_name("saturating_mul");
    let saturating_mul_fn =
        assert_not_nil(LLVMGetNamedFunction(module, saturating_mul_name.as_ptr()));
    let args = &mut [lhs, rhs];
    function_call(builder, saturating_mul_fn, args, name)
}

pub unsafe fn saturating_div(
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    numerator: LLVMValueRef,
    denominator: LLVMValueRef,
    name: CString,
) -> LLVMValueRef {
    let saturating_div_name = llvm_name("saturating_div");
    let saturating_div_fn =
        assert_not_nil(LLVMGetNamedFunction(module, saturating_div_name.as_ptr()));
    let args = &mut [numerator, denominator];
    function_call(builder, saturating_div_fn, args, name)
}

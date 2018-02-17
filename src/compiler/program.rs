use super::*;
use llvm::prelude::*;
use llvm::LLVMIntPredicate;
use llvm::core::*;

pub unsafe fn define_main(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    input_names: Vec<Name>,
    output_names: Vec<Name>,
    assigns: Vec<(Name, Expression, HashMap<Name, LLVMValueRef>)>,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let argv_type = LLVMPointerType(LLVMPointerType(LLVMInt8Type(), 0), 0);

    let main_params = vec![
        (Name::new("argc"), i64_type),
        (Name::new("argv"), argv_type),
    ];
    let (function, args) = function_definition(module, llvm_name("main"), main_params, i64_type);
    let argc = args[&Name::new("argc")];
    let argv = args[&Name::new("argv")];

    let name = llvm_name("entry");
    let entry_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("argv_abort");
    let argv_abort_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("input");
    let input_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("run");
    let run_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("output");
    let output_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));

    LLVMPositionBuilderAtEnd(builder, entry_block);
    let args_cmp_name = llvm_name("args_cmp");
    let args_cmp = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntNE,
        LLVMConstInt(i64_type, (input_names.len() + 1) as u64, 0),
        argc,
        args_cmp_name.as_ptr(),
    ));
    assert_not_nil(LLVMBuildCondBr(
        builder,
        args_cmp,
        argv_abort_block,
        input_block,
    ));

    LLVMPositionBuilderAtEnd(builder, argv_abort_block);
    let args_cmp_tmpl = global_string_ptr(
        builder,
        llvm_name("args_cmp_tmpl"),
        llvm_name(&format!(
            "Program expects {} inputs but was provided with %lld\n",
            input_names.len()
        )),
    );
    let name = llvm_name("args_cmp_sub");
    printf(
        module,
        builder,
        args_cmp_tmpl,
        LLVMBuildSub(builder, argc, LLVMConstInt(i64_type, 1, 0), name.as_ptr()),
        llvm_name("args_cmp_printf"),
    );
    function_return(builder, LLVMConstInt(i64_type, 1, 0));

    LLVMPositionBuilderAtEnd(builder, input_block);
    let i64_tmpl = global_string_ptr(builder, llvm_name("i64_tmpl"), llvm_name("%lld"));
    let mut vars = HashMap::new();
    for (i, input_name) in input_names.clone().into_iter().enumerate() {
        let name = llvm_name(&format!("{}_ptr", input_name));
        let var = allocate(builder, i64_type, name);
        let argv_el = getelement(
            ctx,
            builder,
            argv,
            1 + i as u64,
            llvm_name(""),
            llvm_name(""),
        );
        sscanf(module, builder, argv_el, i64_tmpl, var, llvm_name(""));
        vars.insert(input_name, var);
    }
    assert_not_nil(LLVMBuildBr(builder, run_block));

    LLVMPositionBuilderAtEnd(builder, run_block);

    // FIXME: Assign unchanging inputs to outputs.
    for &(ref var_name, _, _) in &assigns {
        if !vars.contains_key(var_name) {
            let var = allocate(builder, i64_type, llvm_name(&format!("{}_ptr", var_name)));
            vars.insert(var_name.clone(), var);
        }
    }
    for &(ref var_name, ref expression, ref functions) in &assigns {
        let value =
            synthesise_expression(ctx, module, builder, function, expression, &vars, functions);
        assert_not_nil(LLVMBuildStore(builder, value, vars[var_name]));
    }
    assert_not_nil(LLVMBuildBr(builder, output_block));

    LLVMPositionBuilderAtEnd(builder, output_block);
    let i64_line_tmpl = global_string_ptr(builder, llvm_name("i64_line_tmpl"), llvm_name("%lld\n"));
    for output_name in output_names {
        let var = vars[&output_name];
        let output = load(builder, var, output_name.cstring());
        printf(module, builder, i64_line_tmpl, output, llvm_name(""));
    }

    function_return(builder, LLVMConstInt(i64_type, 0, 0));
    function
}

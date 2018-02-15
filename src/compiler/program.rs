use super::*;
use llvm::prelude::*;
use llvm::LLVMIntPredicate;
use llvm::core::*;
use std::iter;

pub unsafe fn define_input(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    inputs: Vec<Name>,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let i64_ptr_type = LLVMPointerType(i64_type, 0);
    let argv_type = LLVMPointerType(LLVMPointerType(LLVMInt8Type(), 0), 0);
    let void_type = LLVMVoidTypeInContext(ctx);
    let i64_tmpl = global_string_ptr(builder, llvm_name("i64_tmpl"), llvm_name("%lld"));

    let input_ptrs = inputs
        .clone()
        .into_iter()
        .map(|input_name| Name::new(&format!("{}_ptr", input_name)))
        .zip(iter::repeat(i64_ptr_type));
    let mut params = vec![
        (Name::new("argc"), i64_type),
        (Name::new("argv"), argv_type),
    ];
    params.extend(input_ptrs);
    let fn_name = llvm_name("input");
    let (function, params) = function_definition(module, fn_name, params, void_type);
    let argv = assert_not_nil(params[&Name::new("argv")]);

    function_entry(ctx, builder, llvm_name("entry"), function);
    for (i, input_name) in inputs.into_iter().enumerate() {
        let argv_el_name = Name::new(&format!("{}_argv_el", input_name)).cstring();
        let argv_el = getelement(ctx, builder, argv, 1 + i as u64, argv_el_name);

        let sscanf_name = Name::new(&format!("{}_sscanf", input_name)).cstring();
        let input_ptr = assert_not_nil(LLVMGetParam(function, 2 + i as u32));
        sscanf(module, builder, argv_el, i64_tmpl, input_ptr, sscanf_name);
    }

    assert_not_nil(LLVMBuildRetVoid(builder));
    function
}

pub unsafe fn define_output(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    outputs: Vec<Name>,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let i64_ptr_type = LLVMPointerType(i64_type, 0);
    let void_type = LLVMVoidTypeInContext(ctx);
    let i64_line_tmpl = global_string_ptr(builder, llvm_name("i64_line_tmpl"), llvm_name("%lld\n"));

    let param_types = outputs
        .iter()
        .map(|output_name| Name::new(&format!("{}_ptr", output_name)))
        .zip(iter::repeat(i64_ptr_type))
        .collect();
    let fn_name = llvm_name("output");
    let (function, params) = function_definition(module, fn_name, param_types, void_type);

    function_entry(ctx, builder, llvm_name("entry"), function);
    for output_name in outputs {
        let output_ptr = params[&Name::new(&format!("{}_ptr", output_name))];
        let output = load(builder, output_ptr, output_name.clone().cstring());
        let printf_name = Name::new(&format!("{}_printf", output_name)).cstring();
        printf(module, builder, i64_line_tmpl, output, printf_name);
    }
    assert_not_nil(LLVMBuildRetVoid(builder));
    function
}

pub unsafe fn define_run(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    params: Vec<Param>,
    assigns: Vec<(Name, Expression, HashMap<Name, LLVMValueRef>)>,
) -> LLVMValueRef {
    let void_type = LLVMVoidTypeInContext(ctx);
    let i64_type = LLVMInt64TypeInContext(ctx);
    let i64_ptr_type = LLVMPointerType(i64_type, 0);

    let param_types = params
        .iter()
        .map(|param| Name::new(&format!("{}_ptr", param.name())))
        .zip(iter::repeat(i64_ptr_type))
        .collect();
    let fn_name = llvm_name("run");
    let (function, param_values) = function_definition(module, fn_name, param_types, void_type);

    let mut vars = HashMap::new();
    for param in &params {
        if param.pre_initialised() {
            let value = assert_not_nil(param_values[&Name::new(&format!("{}_ptr", param.name()))]);
            vars.insert(param.name().clone(), value);
        }
    }

    function_entry(ctx, builder, llvm_name("entry"), function);
    for &(ref var_name, ref expression, ref functions) in &assigns {
        if !vars.contains_key(var_name) {
            if let Some(param_value) = param_values.get(&Name::new(&format!("{}_ptr", var_name))) {
                vars.insert(var_name.clone(), *param_value);
            } else {
                let var = allocate(builder, i64_type, var_name.clone().cstring());
                vars.insert(var_name.clone(), var);
            }
        }
        let value =
            synthesise_expression(ctx, module, builder, function, expression, &vars, functions);
        assert_not_nil(LLVMBuildStore(builder, value, vars[var_name]));
    }

    assert_not_nil(LLVMBuildRetVoid(builder));
    function
}

pub unsafe fn define_main(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    params: Vec<Param>,
    input_function: LLVMValueRef,
    run_function: LLVMValueRef,
    output_function: LLVMValueRef,
    input_count: u64,
    outputs: Vec<Name>,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let argv_type = LLVMPointerType(LLVMPointerType(LLVMInt8Type(), 0), 0);

    let main_params = vec![
        (Name::new("argc"), i64_type),
        (Name::new("argv"), argv_type),
    ];
    let (function, param_values) =
        function_definition(module, llvm_name("main"), main_params, i64_type);
    let argc = assert_not_nil(param_values[&Name::new("argc")]);
    let argv = assert_not_nil(param_values[&Name::new("argv")]);

    let name = llvm_name("entry");
    let entry_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("then");
    let then_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("else");
    let else_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));

    LLVMPositionBuilderAtEnd(builder, entry_block);
    let args_cmp_name = llvm_name("args_cmp");
    let args_cmp = assert_not_nil(LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntEQ,
        LLVMConstInt(i64_type, input_count + 1, 0),
        argc,
        args_cmp_name.as_ptr(),
    ));
    assert_not_nil(LLVMBuildCondBr(builder, args_cmp, then_block, else_block));
    LLVMPositionBuilderAtEnd(builder, else_block);
    let args_cmp_tmpl = global_string_ptr(
        builder,
        llvm_name("args_cmp_tmpl"),
        llvm_name(&format!(
            "Program expects {} inputs but was provided with %lld\n",
            input_count
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

    LLVMPositionBuilderAtEnd(builder, then_block);
    let mut vars = HashMap::new();
    for param in &params {
        if param.pre_initialised() {
            let var_name = llvm_name(&format!("{}_ptr", param.name().0));
            let var = allocate(builder, i64_type, var_name);
            vars.insert(param.name().clone(), var);
        }
    }
    let mut input_args = vec![argc, argv];
    for param in &params {
        if param.pre_initialised() {
            input_args.push(vars[param.name()]);
        }
    }
    let input_args = input_args.as_mut_slice();
    function_call(builder, input_function, input_args, llvm_name(""));

    for param in &params {
        if !param.pre_initialised() {
            let var_name = llvm_name(&format!("{}_ptr", param.name().0));
            let var = allocate(builder, i64_type, var_name);
            vars.insert(param.name().clone(), var);
        }
    }
    let mut run_args = vec![];
    for param in &params {
        run_args.push(vars[param.name()]);
    }
    let run_args = run_args.as_mut_slice();
    function_call(builder, run_function, run_args, llvm_name(""));

    let mut params_map = HashMap::new();
    for param in &params {
        params_map.insert(param.name().clone(), vars[param.name()]);
    }
    let mut output_args: Vec<LLVMValueRef> = outputs
        .iter()
        .map(|output_name| params_map[output_name])
        .collect();
    let output_args = output_args.as_mut_slice();
    function_call(builder, output_function, output_args, llvm_name(""));

    function_return(builder, LLVMConstInt(i64_type, 0, 0));
    function
}

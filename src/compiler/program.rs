use super::*;
use llvm::prelude::*;
use llvm::LLVMIntPredicate;
use llvm::core::*;

pub unsafe fn llvm_define_input(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    inputs: Vec<Name>,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let i64_ptr_type = LLVMPointerType(i64_type, 0);
    let argv_type = LLVMPointerType(LLVMPointerType(LLVMInt8Type(), 0), 0);

    let mut params = vec![
        (Name::new("argc"), i64_type),
        (Name::new("argv"), argv_type),
    ];
    params.extend(
        inputs
            .iter()
            .map(|input_name| (Name::new(&format!("{}_ptr", input_name)), i64_ptr_type)),
    );

    let (function, params) = llvm_function_definition(
        module,
        llvm_name("input"),
        params,
        LLVMVoidTypeInContext(ctx),
    );
    llvm_function_entry(ctx, builder, llvm_name("entry"), function);

    let i64_tmpl = llvm_global_string_ptr(builder, llvm_name("i64_tmpl"), llvm_name("%lld"));
    for (i, input_name) in inputs.into_iter().enumerate() {
        let input_ptr = LLVMGetParam(function, 2 + i as u32);
        let argv_el_ptr = llvm_getelement(
            ctx,
            builder,
            params[&Name::new("argv")],
            1 + i as u64,
            Name::new(&format!("{}_argv_ptr", input_name)).cstring(),
        );
        let argv_el = llvm_load(
            builder,
            argv_el_ptr,
            Name::new(&format!("{}_argv", input_name)).cstring(),
        );
        llvm_sscanf(
            module,
            builder,
            argv_el,
            i64_tmpl,
            input_ptr,
            Name::new(&format!("{}_sscanf", input_name)).cstring(),
        );
    }
    LLVMBuildRetVoid(builder);
    function
}

pub unsafe fn llvm_define_output(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    outputs: Vec<Name>,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let i64_ptr_type = LLVMPointerType(i64_type, 0);

    let param_types = outputs
        .iter()
        .map(|output_name| (Name::new(&format!("{}_ptr", output_name)), i64_ptr_type))
        .collect();

    let (function, params) = llvm_function_definition(
        module,
        llvm_name("output"),
        param_types,
        LLVMVoidTypeInContext(ctx),
    );
    llvm_function_entry(ctx, builder, llvm_name("entry"), function);

    let i64_line_tmpl =
        llvm_global_string_ptr(builder, llvm_name("i64_line_tmpl"), llvm_name("%lld\n"));
    for output_name in outputs {
        let output_ptr = params[&Name::new(&format!("{}_ptr", output_name))];
        let output = llvm_load(builder, output_ptr, output_name.clone().cstring());
        llvm_printf(
            module,
            builder,
            i64_line_tmpl,
            output,
            Name::new(&format!("{}_printf", output_name)).cstring(),
        );
    }
    LLVMBuildRetVoid(builder);
    function
}

pub unsafe fn llvm_define_run(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    params: Vec<Param>,
    assigns: Vec<(Name, Expression)>,
    functions: &HashMap<Name, LLVMValueRef>,
) -> LLVMValueRef {
    let void_type = LLVMVoidTypeInContext(ctx);
    let i64_type = LLVMInt64TypeInContext(ctx);
    let i64_ptr_type = LLVMPointerType(i64_type, 0);

    let param_types = params
        .iter()
        .map(|param| (Name::new(&format!("{}_ptr", param.name())), i64_ptr_type))
        .collect();
    let (function, param_values) =
        llvm_function_definition(module, llvm_name("run"), param_types, void_type);
    llvm_function_entry(ctx, builder, llvm_name("entry"), function);

    let mut initialised_vars = HashMap::new();
    for param in &params {
        if param.pre_initialised() {
            initialised_vars.insert(
                param.name().clone(),
                param_values[&Name::new(&format!("{}_ptr", param.name()))],
            );
        }
    }

    for &(ref var_name, ref var_expression) in &assigns {
        if !initialised_vars.contains_key(var_name) {
            if let Some(param_value) = param_values.get(&Name::new(&format!("{}_ptr", var_name))) {
                initialised_vars.insert(var_name.clone(), *param_value);
            } else {
                let var_name_cstring = var_name.clone().cstring();
                let var = LLVMBuildAlloca(builder, i64_ptr_type, var_name_cstring.as_ptr());
                initialised_vars.insert(var_name.clone(), var);
            }
        }
        let value = ExpressionSynthesiser::synthesise(
            ctx,
            module,
            builder,
            var_expression,
            &initialised_vars,
            &functions,
        );
        LLVMBuildStore(builder, value, initialised_vars[var_name]);
    }

    LLVMBuildRetVoid(builder);
    function
}

pub unsafe fn llvm_define_main(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    params: Vec<Param>,
    input_function: LLVMValueRef,
    run_function: LLVMValueRef,
    output_function: LLVMValueRef,
    input_count: u64,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let argv_type = LLVMPointerType(LLVMPointerType(LLVMInt8Type(), 0), 0);

    let main_params = vec![
        (Name::new("argc"), i64_type),
        (Name::new("argv"), argv_type),
    ];
    let (function, param_values) =
        llvm_function_definition(module, llvm_name("main"), main_params, i64_type);
    let argc = param_values[&Name::new("argc")];
    let argv = param_values[&Name::new("argv")];

    let name = llvm_name("entry");
    let entry_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());
    let name = llvm_name("then");
    let then_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());
    let name = llvm_name("else");
    let else_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());

    LLVMPositionBuilderAtEnd(builder, entry_block);
    let args_cmp_name = llvm_name("args_cmp");
    let args_cmp = LLVMBuildICmp(
        builder,
        LLVMIntPredicate::LLVMIntEQ,
        LLVMConstInt(i64_type, input_count + 1, 0),
        argc,
        args_cmp_name.as_ptr(),
    );
    LLVMBuildCondBr(builder, args_cmp, then_block, else_block);
    LLVMPositionBuilderAtEnd(builder, else_block);
    let args_cmp_tmpl = llvm_global_string_ptr(
        builder,
        llvm_name("args_cmp_tmpl"),
        llvm_name(&format!(
            "Program expects {} inputs but was provided with %lld\n",
            input_count
        )),
    );
    let name = llvm_name("args_cmp_sub");
    llvm_printf(
        module,
        builder,
        args_cmp_tmpl,
        LLVMBuildSub(builder, argc, LLVMConstInt(i64_type, 1, 0), name.as_ptr()),
        llvm_name("args_cmp_printf"),
    );
    llvm_function_return(builder, LLVMConstInt(i64_type, 1, 0));

    LLVMPositionBuilderAtEnd(builder, then_block);
    let mut vars = HashMap::new();
    for param in &params {
        if param.pre_initialised() {
            let var_name = llvm_name(&format!("{}_ptr", param.name().0));
            let var = llvm_allocate(builder, i64_type, var_name);
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
    llvm_call(builder, input_function, input_args, llvm_name(""));

    for param in &params {
        if !param.pre_initialised() {
            let var_name = llvm_name(&format!("{}_ptr", param.name().0));
            let var = llvm_allocate(builder, i64_type, var_name);
            vars.insert(param.name().clone(), var);
        }
    }
    let mut run_args = vec![];
    for param in &params {
        run_args.push(vars[param.name()]);
    }
    let run_args = run_args.as_mut_slice();
    llvm_call(builder, run_function, run_args, llvm_name(""));

    let mut output_args = vec![];
    for param in &params {
        if param.outputted() {
            output_args.push(vars[param.name()]);
        }
    }
    let output_args = output_args.as_mut_slice();
    llvm_call(builder, output_function, output_args, llvm_name(""));

    llvm_function_return(builder, LLVMConstInt(i64_type, 0, 0));
    function
}

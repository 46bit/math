use super::*;
use llvm::prelude::*;
use llvm::LLVMIntPredicate;
use llvm::core::{LLVMAppendBasicBlockInContext, LLVMBuildAlloca, LLVMBuildCondBr, LLVMBuildICmp,
                 LLVMBuildRetVoid, LLVMBuildSub, LLVMConstInt, LLVMGetParam,
                 LLVMPositionBuilderAtEnd, LLVMVoidTypeInContext};

pub unsafe fn llvm_input(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    inputs: Vec<String>,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let i64_ptr_type = LLVMPointerType(i64_type, 0);
    let argv_type = LLVMPointerType(LLVMPointerType(LLVMInt8Type(), 0), 0);

    let mut params = vec![
        (llvm_name("argc"), i64_type),
        (llvm_name("argv"), argv_type),
    ];
    params.extend(
        inputs
            .iter()
            .map(|input_name| (llvm_name(&(input_name.clone() + "_ptr")), i64_ptr_type)),
    );

    let function = llvm_function_definition(
        module,
        llvm_name("input"),
        params,
        LLVMVoidTypeInContext(ctx),
    );
    let argv = LLVMGetParam(function, 1 as u32);
    llvm_function_entry(ctx, builder, llvm_name("entry"), function);

    let i64_tmpl = llvm_global_string_ptr(builder, llvm_name("i64_tmpl"), llvm_name("%lld"));
    for (i, input_name) in inputs.into_iter().enumerate() {
        let input_ptr = LLVMGetParam(function, 2 + i as u32);
        let argv_el_ptr = llvm_getelement(
            ctx,
            builder,
            argv,
            1 + i as u64,
            llvm_name(&(input_name.clone() + "_argv_ptr")),
        );
        let argv_el = llvm_load(
            builder,
            argv_el_ptr,
            llvm_name(&(input_name.clone() + "_argv")),
        );
        llvm_sscanf(
            module,
            builder,
            argv_el,
            i64_tmpl,
            input_ptr,
            llvm_name(&(input_name + "_sscanf")),
        );
    }
    LLVMBuildRetVoid(builder);
    function
}

pub unsafe fn llvm_output(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    outputs: Vec<String>,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);
    let i64_ptr_type = LLVMPointerType(i64_type, 0);

    let params = outputs
        .iter()
        .map(|output_name| (llvm_name(&(output_name.clone() + "_ptr")), i64_ptr_type))
        .collect();

    let function = llvm_function_definition(
        module,
        llvm_name("output"),
        params,
        LLVMVoidTypeInContext(ctx),
    );
    llvm_function_entry(ctx, builder, llvm_name("entry"), function);

    let i64_line_tmpl =
        llvm_global_string_ptr(builder, llvm_name("i64_line_tmpl"), llvm_name("%lld\n"));
    for (i, output_name) in outputs.into_iter().enumerate() {
        let output_ptr = LLVMGetParam(function, i as u32);
        let output = llvm_load(builder, output_ptr, llvm_name(&output_name));
        llvm_printf(
            module,
            builder,
            i64_line_tmpl,
            output,
            llvm_name(&(output_name + "_printf")),
        );
    }
    LLVMBuildRetVoid(builder);
    function
}

pub unsafe fn llvm_run(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    params: Vec<Param>,
    assigns: Vec<(String, Expression)>,
    functions: &HashMap<String, LLVMValueRef>,
) -> LLVMValueRef {
    let void_type = LLVMVoidTypeInContext(ctx);
    let i64_type = LLVMInt64TypeInContext(ctx);
    let i64_ptr_type = LLVMPointerType(i64_type, 0);

    let param_types = params
        .iter()
        .map(|param| (llvm_name(&(param.name().clone() + "_ptr")), i64_ptr_type))
        .collect();
    let function = llvm_function_definition(module, llvm_name("run"), param_types, void_type);
    llvm_function_entry(ctx, builder, llvm_name("entry"), function);
    let param_values: HashMap<String, LLVMValueRef> = params
        .iter()
        .enumerate()
        .map(|(i, param)| (param.name().clone(), LLVMGetParam(function, i as u32)))
        .collect();

    let mut initialised_vars: HashMap<String, Var> = params
        .iter()
        .filter(|param| param.pre_initialised())
        .map(|param| (param.name().clone(), Var::Stack(param_values[param.name()])))
        .collect();

    for &(ref var_name, ref var_expression) in &assigns {
        if !initialised_vars.contains_key(var_name) {
            if let Some(param_value) = param_values.get(var_name) {
                initialised_vars.insert(var_name.clone(), Var::Stack(*param_value));
            } else {
                let var_name_cstring = llvm_name(var_name);
                let var = Var::Stack(LLVMBuildAlloca(
                    builder,
                    i64_ptr_type,
                    var_name_cstring.as_ptr(),
                ));
                initialised_vars.insert(var_name.clone(), var);
            }
        }
        let value = ExpressionSynthesiser::synthesise(
            ctx,
            builder,
            var_expression,
            &initialised_vars,
            &functions,
        );
        initialised_vars[var_name].synthesise_assignment(builder, value);
    }

    LLVMBuildRetVoid(builder);
    function
}

pub unsafe fn llvm_main(
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
        (llvm_name("argc"), i64_type),
        (llvm_name("argv"), argv_type),
    ];
    let function = llvm_function_definition(module, llvm_name("main"), main_params, i64_type);
    let argc = LLVMGetParam(function, 0 as u32);
    let argv = LLVMGetParam(function, 1 as u32);

    let name = llvm_name("entry");
    let entry_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());
    let name = llvm_name("then");
    let then_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());
    let name = llvm_name("else");
    let else_block = LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr());

    // FIXME: Synthesise LLVM IR that's the equivalent of
    // if inputs.len() != argc {
    //     printf("Expected %d arguments but received %d.", inputs.len(), argc);
    //     return 1;
    // }
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
            let var_name = Name(param.name().clone());
            let var = Var::synthesise_allocation(builder, var_name, i64_type);
            vars.insert(param.name().clone(), var);
        }
    }

    let mut input_args = vec![argc, argv];
    for param in &params {
        if param.pre_initialised() {
            input_args.push(vars[param.name()].synthesise_pointer());
        }
    }
    llvm_call(
        builder,
        input_function,
        input_args.as_mut_slice(),
        llvm_name(""),
    );

    for param in &params {
        if !param.pre_initialised() {
            let var_name = Name(param.name().clone());
            let var = Var::synthesise_allocation(builder, var_name, i64_type);
            vars.insert(param.name().clone(), var);
        }
    }

    let mut run_args = vec![];
    for param in &params {
        run_args.push(vars[param.name()].synthesise_pointer());
    }
    llvm_call(
        builder,
        run_function,
        run_args.as_mut_slice(),
        llvm_name(""),
    );

    let mut output_args = vec![];
    for param in &params {
        if param.outputted() {
            output_args.push(vars[param.name()].synthesise_pointer());
        }
    }
    llvm_call(
        builder,
        output_function,
        output_args.as_mut_slice(),
        llvm_name(""),
    );

    llvm_function_return(builder, LLVMConstInt(i64_type, 0, 0));
    function
}

use super::*;
use llvm::prelude::*;
use llvm::core::*;
use std::collections::HashMap;

pub unsafe fn synthesise_expression(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    function: LLVMValueRef,
    expression: &Expression,
    vars: &HashMap<Name, LLVMValueRef>,
    fns: &HashMap<Name, LLVMValueRef>,
) -> LLVMValueRef {
    ExpressionSynthesiser {
        ctx: ctx,
        module: module,
        builder: builder,
        function: function,
        vars: vars,
        fns: fns,
    }.synthesise(expression)
}

struct ExpressionSynthesiser<'a> {
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    function: LLVMValueRef,
    vars: &'a HashMap<Name, LLVMValueRef>,
    fns: &'a HashMap<Name, LLVMValueRef>,
}

impl<'a> ExpressionSynthesiser<'a> {
    unsafe fn synthesise(&self, expression: &Expression) -> LLVMValueRef {
        match expression {
            &Expression::Operand(ref operand) => self.synthesise_operand(operand),
            &Expression::Operation(operator, ref lhs, ref rhs) => {
                self.synthesise_operation(operator, lhs, rhs)
            }
        }
    }

    unsafe fn synthesise_operation(
        &self,
        operator: Operator,
        lhs: &Expression,
        rhs: &Expression,
    ) -> LLVMValueRef {
        let module = self.module;
        let builder = self.builder;
        let lhs = self.synthesise(lhs);
        let rhs = self.synthesise(rhs);
        match operator {
            Operator::Subtract => saturating_sub(module, builder, lhs, rhs, llvm_name("tmp_sub")),
            Operator::Add => saturating_add(module, builder, lhs, rhs, llvm_name("tmp_add")),
            Operator::Divide => saturating_div(module, builder, lhs, rhs, llvm_name("tmp_div")),
            Operator::Multiply => saturating_mul(module, builder, lhs, rhs, llvm_name("tmp_mul")),
        }
    }

    unsafe fn synthesise_operand(&self, operand: &Operand) -> LLVMValueRef {
        let i64_type = LLVMInt64TypeInContext(self.ctx);
        let i64_ptr_type = LLVMPointerType(i64_type, 0);
        match operand {
            &Operand::I64(n) => LLVMConstInt(i64_type, n as u64, 0),
            &Operand::Group(ref expression) => self.synthesise(expression),
            &Operand::VarSubstitution(ref name) => {
                let var = self.vars[&name];
                let var_type = assert_not_nil(LLVMTypeOf(var));
                if var_type == i64_type {
                    var
                } else if var_type == i64_ptr_type {
                    let name = into_llvm_name(name.clone());
                    assert_not_nil(LLVMBuildLoad(self.builder, var, name.as_ptr()))
                } else {
                    unimplemented!();
                }
            }
            &Operand::FnApplication(ref name, ref args) => synthesise_fn_application(
                self.builder,
                name,
                args.iter().map(|e| self.synthesise(e)).collect(),
                self.fns,
            ),
            &Operand::Match(ref match_) => synthesise_match(
                self.ctx,
                self.module,
                self.builder,
                self.function,
                self.synthesise(&match_.with),
                &match_.clauses,
                &match_.default,
                self.vars,
                self.fns,
            ),
        }
    }
}

pub unsafe fn synthesise_match(
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    function: LLVMValueRef,
    with: LLVMValueRef,
    matchers: &Vec<(Matcher, Expression)>,
    default: &Expression,
    vars: &HashMap<Name, LLVMValueRef>,
    functions: &HashMap<Name, LLVMValueRef>,
) -> LLVMValueRef {
    let i64_type = LLVMInt64TypeInContext(ctx);

    // Make a new block
    let matcher_blocks: Vec<LLVMBasicBlockRef> = matchers
        .iter()
        .map(|_| {
            let name = llvm_name("match");
            assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()))
        })
        .collect();
    let name = llvm_name("match_none");
    let unmatched_block =
        assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
    let name = llvm_name("match_final");
    let final_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));

    let name = llvm_name("match_dest");
    let dest = allocate(builder, i64_type, name);

    // Jump to the new block
    // Make one new block for each flat_matcher

    let switch = assert_not_nil(LLVMBuildSwitch(
        builder,
        with,
        unmatched_block,
        matchers.len() as u32,
    ));
    for (i, &(ref matcher, _)) in matchers.iter().enumerate() {
        match matcher {
            &Matcher::Value(ref expression) => {
                let value = synthesise_expression(
                    ctx,
                    module,
                    builder,
                    function,
                    expression,
                    vars,
                    functions,
                );
                LLVMAddCase(switch, value, matcher_blocks[i]);
            }
        }
    }

    for (i, &(_, ref expression)) in matchers.into_iter().enumerate() {
        LLVMPositionBuilderAtEnd(builder, matcher_blocks[i]);
        let value = synthesise_expression(
            ctx,
            module,
            builder,
            function,
            &expression,
            &vars,
            &functions,
        );
        assert_not_nil(LLVMBuildStore(builder, value, dest));
        assert_not_nil(LLVMBuildBr(builder, final_block));
    }

    LLVMPositionBuilderAtEnd(builder, unmatched_block);
    let value = synthesise_expression(
        ctx,
        module,
        builder,
        function,
        default,
        &vars,
        &functions,
    );
    assert_not_nil(LLVMBuildStore(builder, value, dest));
    assert_not_nil(LLVMBuildBr(builder, final_block));

    LLVMPositionBuilderAtEnd(builder, final_block);
    let name = llvm_name("tmp_match");
    assert_not_nil(LLVMBuildLoad(builder, dest, name.as_ptr()))
}

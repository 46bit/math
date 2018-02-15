use super::*;
use llvm::prelude::*;
use llvm::core::*;
use llvm::LLVMIntPredicate;
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
            &Operand::FnApplication(ref name, ref arg_exprs) => {
                let function = *self.fns.get(&name).unwrap();
                let mut args: Vec<_> = arg_exprs.iter().map(|e| self.synthesise(e)).collect();
                let call_name = llvm_name(&format!("{}_call", name));
                function_call(self.builder, function, args.as_mut_slice(), call_name)
            }
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

    let name = llvm_name("match_dest");
    let dest = allocate(builder, i64_type, name);

    let name = llvm_name("match_final");
    let final_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));

    for &(ref matcher, ref expression) in matchers {
        let name = llvm_name("match_assignment");
        let assignment_block =
            assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));
        let name = llvm_name("match_cmp");
        let cmp_block = assert_not_nil(LLVMAppendBasicBlockInContext(ctx, function, name.as_ptr()));

        match matcher {
            &Matcher::Value(ref cmp_expression) => {
                // Evaluate matcher expression.
                let cmp_value = synthesise_expression(
                    ctx,
                    module,
                    builder,
                    function,
                    cmp_expression,
                    vars,
                    functions,
                );
                let cmp_name = llvm_name("cmp");
                let cmp = assert_not_nil(LLVMBuildICmp(
                    builder,
                    LLVMIntPredicate::LLVMIntEQ,
                    with,
                    cmp_value,
                    cmp_name.as_ptr(),
                ));
                assert_not_nil(LLVMBuildCondBr(builder, cmp, assignment_block, cmp_block));
            }
        }

        LLVMPositionBuilderAtEnd(builder, assignment_block);
        let value = synthesise_expression(
            ctx,
            module,
            builder,
            function,
            expression,
            &vars,
            &functions,
        );
        assert_not_nil(LLVMBuildStore(builder, value, dest));
        assert_not_nil(LLVMBuildBr(builder, final_block));

        LLVMPositionBuilderAtEnd(builder, cmp_block);
    }

    let default_value =
        synthesise_expression(ctx, module, builder, function, default, &vars, &functions);
    assert_not_nil(LLVMBuildStore(builder, default_value, dest));
    assert_not_nil(LLVMBuildBr(builder, final_block));

    LLVMPositionBuilderAtEnd(builder, final_block);
    let name = llvm_name("tmp_match");
    assert_not_nil(LLVMBuildLoad(builder, dest, name.as_ptr()))
}

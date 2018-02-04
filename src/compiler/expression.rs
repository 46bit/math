use super::*;
use llvm::prelude::*;
use llvm::core::*;
use std::collections::HashMap;

pub struct ExpressionSynthesiser<'a> {
    ctx: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    vars: &'a HashMap<Name, LLVMValueRef>,
    fns: &'a HashMap<Name, LLVMValueRef>,
}

impl<'a> ExpressionSynthesiser<'a> {
    pub unsafe fn synthesise(
        ctx: LLVMContextRef,
        module: LLVMModuleRef,
        builder: LLVMBuilderRef,
        expression: &Expression,
        vars: &'a HashMap<Name, LLVMValueRef>,
        fns: &HashMap<Name, LLVMValueRef>,
    ) -> LLVMValueRef {
        ExpressionSynthesiser {
            ctx: ctx,
            module: module,
            builder: builder,
            vars: vars,
            fns: fns,
        }.synthesise_expression(expression)
    }

    unsafe fn synthesise_expression(&self, expression: &Expression) -> LLVMValueRef {
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
        let lhs = self.synthesise_expression(lhs);
        let rhs = self.synthesise_expression(rhs);
        match operator {
            Operator::Subtract => {
                let name = llvm_name("tmp_sub");
                LLVMBuildSub(self.builder, lhs, rhs, name.as_ptr())
            }
            Operator::Add => {
                let name = llvm_name("tmp_add");
                LLVMBuildAdd(self.builder, lhs, rhs, name.as_ptr())
            }
            Operator::Divide => {
                let name = llvm_name("tmp_div");
                saturating_div(self.module, self.builder, lhs, rhs, name)
            }
            Operator::Multiply => {
                let name = llvm_name("tmp_mul");
                LLVMBuildMul(self.builder, lhs, rhs, name.as_ptr())
            }
        }
    }

    unsafe fn synthesise_operand(&self, operand: &Operand) -> LLVMValueRef {
        let i64_type = LLVMInt64TypeInContext(self.ctx);
        let i64_ptr_type = LLVMPointerType(i64_type, 0);
        match operand {
            &Operand::I64(n) => LLVMConstInt(i64_type, n as u64, 0),
            &Operand::Group(ref expression) => self.synthesise_expression(expression),
            &Operand::VarSubstitution(ref name) => {
                let var = self.vars[&name];
                let var_type = LLVMTypeOf(var);
                if var_type == i64_type {
                    var
                } else if var_type == i64_ptr_type {
                    let name = into_llvm_name(name.clone());
                    LLVMBuildLoad(self.builder, var, name.as_ptr())
                } else {
                    unimplemented!();
                }
            }
            &Operand::FnApplication(ref name, ref args) => synthesise_fn_application(
                self.builder,
                name,
                args.iter().map(|e| self.synthesise_expression(e)).collect(),
                self.fns,
            ),
        }
    }
}

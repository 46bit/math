mod function;
mod expression;
mod var;

use super::*;
use self::var::*;
use self::function::*;
use self::expression::*;
use llvm;
use llvm::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::ptr;
use std::slice;
use std::ffi::{CStr, CString};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Unknown,
}

pub unsafe fn compile(program: &Program, out_file: &mut File) -> Result<String, Error> {
    let (llvm_ctx, llvm_module) = synthesise(&program)?;

    let c = llvm::core::LLVMPrintModuleToString(llvm_module);
    let llvm_ir = CStr::from_ptr(c).to_string_lossy().into_owned();
    llvm::core::LLVMDisposeMessage(c);

    emit(llvm_module, out_file);

    llvm::core::LLVMDisposeModule(llvm_module);
    llvm::core::LLVMContextDispose(llvm_ctx);
    // FIXME: Error handling.
    return Ok(llvm_ir);
}

unsafe fn emit<W>(llvm_module: LLVMModuleRef, mut out: W)
where
    W: Write,
{
    llvm::target::LLVM_InitializeNativeTarget();
    llvm::target::LLVM_InitializeNativeAsmPrinter();
    llvm::target::LLVM_InitializeNativeAsmParser();

    let llvm_triple = CString::from_raw(llvm::target_machine::LLVMGetDefaultTargetTriple());
    let mut llvm_target = mem::uninitialized();
    assert_eq!(
        llvm::target_machine::LLVMGetTargetFromTriple(
            llvm_triple.as_ptr(),
            &mut llvm_target,
            ptr::null_mut(),
        ),
        0
    );

    let llvm_empty_name = llvm_name("");
    let llvm_target_machine = llvm::target_machine::LLVMCreateTargetMachine(
        llvm_target,
        llvm_triple.as_ptr(),
        llvm_empty_name.as_ptr(),
        llvm_empty_name.as_ptr(),
        llvm::target_machine::LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
        llvm::target_machine::LLVMRelocMode::LLVMRelocDefault,
        llvm::target_machine::LLVMCodeModel::LLVMCodeModelDefault,
    );

    let mut llvm_mem_buf: LLVMMemoryBufferRef = mem::uninitialized();
    llvm::target_machine::LLVMTargetMachineEmitToMemoryBuffer(
        llvm_target_machine,
        llvm_module,
        llvm::target_machine::LLVMCodeGenFileType::LLVMObjectFile,
        ptr::null_mut(),
        &mut llvm_mem_buf,
    );

    let llvm_out = slice::from_raw_parts(
        llvm::core::LLVMGetBufferStart(llvm_mem_buf) as *const _,
        llvm::core::LLVMGetBufferSize(llvm_mem_buf) as usize,
    );
    out.write_all(llvm_out).unwrap();
    llvm::core::LLVMDisposeMemoryBuffer(llvm_mem_buf);
    llvm::target_machine::LLVMDisposeTargetMachine(llvm_target_machine);
}

unsafe fn synthesise(program: &Program) -> Result<(*mut llvm::LLVMContext, LLVMModuleRef), Error> {
    let llvm_ctx = llvm::core::LLVMContextCreate();
    let llvm_module = llvm::core::LLVMModuleCreateWithName(b"module\0".as_ptr() as *const _);
    let llvm_builder = llvm::core::LLVMCreateBuilderInContext(llvm_ctx);

    let mut llvm_functions = HashMap::new();

    for statement in &program.statements.0 {
        if let &Statement::FnDefinition(ref name, ref params, ref expr) = statement {
            let function = Function::new(name, Some(params), None, None, expr);
            let llvm_function =
                function.synthesise(llvm_ctx, llvm_module, llvm_builder, &llvm_functions);
            llvm_functions.insert(name.clone(), llvm_function);
        }
    }

    let llvm_string_type = llvm::core::LLVMArrayType(llvm::core::LLVMInt8Type(), 0);
    let llvm_strings_type = llvm::core::LLVMArrayType(llvm_string_type, 0);
    let llvm_i32_type = llvm::core::LLVMInt32TypeInContext(llvm_ctx);
    let llvm_i64_type = llvm::core::LLVMInt64TypeInContext(llvm_ctx);

    let params_types = &mut [llvm_string_type];
    let llvm_fn_type = llvm::core::LLVMFunctionType(llvm_i32_type, params_types.as_mut_ptr(), 1, 0);
    let llvm_fn_name = llvm_name("printf");
    let llvm_printf_fn =
        llvm::core::LLVMAddFunction(llvm_module, llvm_fn_name.as_ptr(), llvm_fn_type);
    llvm_functions.insert(Name::new("printf"), llvm_printf_fn);

    let params_types = &mut [llvm_strings_type, llvm_string_type, llvm_i64_type];
    let llvm_fn_type = llvm::core::LLVMFunctionType(llvm_i32_type, params_types.as_mut_ptr(), 1, 1);
    let llvm_fn_name = llvm_name("sscanf");
    let llvm_sscanf_fn =
        llvm::core::LLVMAddFunction(llvm_module, llvm_fn_name.as_ptr(), llvm_fn_type);
    llvm_functions.insert(Name::new("sscanf"), llvm_sscanf_fn);

    let main_prints = &program.outputs;
    let main_assigns = program
        .statements
        .0
        .iter()
        .filter_map(|statement| match statement {
            &Statement::VarAssignment(ref name, ref expr) => Some((name, expr)),
            _ => None,
        })
        .collect();
    let main_function_name = Name::new("main");
    let main_function = Function::new(
        &main_function_name,
        Some(&program.inputs),
        Some(main_assigns),
        Some(&main_prints),
        &Expression::Operand(Operand::I64(0)),
    );
    main_function.synthesise(llvm_ctx, llvm_module, llvm_builder, &llvm_functions);

    llvm::core::LLVMDisposeBuilder(llvm_builder);
    // FIXME: Error handling.
    return Ok((llvm_ctx, llvm_module));
}

fn llvm_name(s: &str) -> CString {
    CString::new(s).unwrap()
}

fn into_llvm_name(name: Name) -> CString {
    llvm_name(name.0.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::parser;
    use rand::thread_rng;
    use quickcheck::{QuickCheck, StdGen};

    fn synthesises_successfully_property(program: Program) -> bool {
        eprintln!("{}\n---", program);
        unsafe { synthesise(&program).is_ok() }
    }

    #[test]
    fn synthesises_successfully() {
        // QuickCheck's default size creates infeasibly vast statements, and beyond some
        // point they stop exploring novel code paths. This does a much better job of
        // exploring potential edgecases.
        for size in 1..11 {
            let mut qc = QuickCheck::new().gen(StdGen::new(thread_rng(), size));
            qc.quickcheck(synthesises_successfully_property as fn(Program) -> bool);
        }
    }

    #[test]
    fn can_synthesise_with_no_inputs_or_outputs() {
        unsafe {
            synthesise(&parse(b"inputs; outputs;")).unwrap();
        }
    }

    #[test]
    fn can_synthesise_inputs_into_outputs() {
        unsafe {
            synthesise(&parse(b"inputs a; outputs a;")).unwrap();
        }
    }

    fn parse(s: &[u8]) -> Program {
        parser::parse(s).unwrap()
    }
}

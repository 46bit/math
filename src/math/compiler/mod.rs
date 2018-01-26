mod function;
mod expression;
mod var;

use super::*;
use self::var::*;
use self::function::*;
use self::expression::*;
use llvm;
use llvm::prelude::*;
use libc;
use std::ptr;
use std::ffi::{CStr, CString};
use std::collections::HashMap;
use std::process::Command;
use std::slice;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Unknown,
}

pub unsafe fn compile(program: &Program, out_path: String) -> Result<String, Error> {
    let llvm_ir = synthesise(program)?;
    let object_path = out_path.clone() + ".o";
    objectify(llvm_ir.clone(), object_path.clone());
    link(out_path, object_path);
    // FIXME: Error handling.
    return Ok(llvm_ir);
}

unsafe fn synthesise(program: &Program) -> Result<String, Error> {
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

    let llvm_string_type = llvm::core::LLVMPointerType(llvm::core::LLVMInt8Type(), 0);
    let llvm_i32_type = llvm::core::LLVMInt32TypeInContext(llvm_ctx);
    let llvm_i64_type = llvm::core::LLVMInt64TypeInContext(llvm_ctx);
    let llvm_i64_ptr_type = llvm::core::LLVMPointerType(llvm_i64_type, 0);

    let params_types = &mut [llvm_string_type, llvm_i64_type];
    let llvm_fn_type = llvm::core::LLVMFunctionType(llvm_i32_type, params_types.as_mut_ptr(), 2, 1);
    let llvm_fn_name = llvm_name("printf");
    let llvm_printf_fn =
        llvm::core::LLVMAddFunction(llvm_module, llvm_fn_name.as_ptr(), llvm_fn_type);
    llvm_functions.insert(Name::new("printf"), llvm_printf_fn);

    let params_types = &mut [llvm_string_type, llvm_string_type, llvm_i64_ptr_type];
    let llvm_fn_type = llvm::core::LLVMFunctionType(llvm_i32_type, params_types.as_mut_ptr(), 3, 1);
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

    let llvm_ir_ptr = llvm::core::LLVMPrintModuleToString(llvm_module);
    let llvm_ir = CStr::from_ptr(llvm_ir_ptr).to_string_lossy().into_owned();
    llvm::core::LLVMDisposeMessage(llvm_ir_ptr);

    llvm::core::LLVMDisposeModule(llvm_module);
    llvm::core::LLVMContextDispose(llvm_ctx);
    Ok(llvm_ir)
}

unsafe fn objectify(llvm_ir: String, object_path: String) {
    let llvm_ctx = llvm::core::LLVMContextCreate();
    let llvm_ir_str = llvm_name(&llvm_ir);
    let llvm_ir_buffer_name = llvm_name("llvm_ir_buffer");
    let llvm_ir_buffer = llvm::core::LLVMCreateMemoryBufferWithMemoryRange(
        llvm_ir_str.as_ptr(),
        llvm_ir_str.as_bytes().len(),
        llvm_ir_buffer_name.as_ptr(),
        0,
    );
    let mut llvm_module = ptr::null_mut();
    let mut errors = ptr::null_mut();
    assert_eq!(
        llvm::ir_reader::LLVMParseIRInContext(
            llvm_ctx,
            llvm_ir_buffer,
            &mut llvm_module,
            &mut errors,
        ),
        0
    );
    if errors != ptr::null_mut() {
        println!("{}", CString::from_raw(errors).to_str().unwrap());
    }

    llvm::target::LLVM_InitializeNativeTarget();
    llvm::target::LLVM_InitializeNativeAsmPrinter();
    llvm::target::LLVM_InitializeNativeAsmParser();

    let llvm_triple_s = llvm::target_machine::LLVMGetDefaultTargetTriple();
    let llvm_triple = CStr::from_ptr(llvm_triple_s);
    let mut llvm_target = ptr::null_mut();
    assert_eq!(
        llvm::target_machine::LLVMGetTargetFromTriple(
            llvm_triple.as_ptr(),
            &mut llvm_target,
            ptr::null_mut(),
        ),
        0
    );

    let llvm_target_machine = llvm::target_machine::LLVMCreateTargetMachine(
        llvm_target,
        llvm_triple.as_ptr(),
        ptr::null(),
        ptr::null(),
        llvm::target_machine::LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
        llvm::target_machine::LLVMRelocMode::LLVMRelocDefault,
        llvm::target_machine::LLVMCodeModel::LLVMCodeModelDefault,
    );
    assert_ne!(llvm_target_machine, ptr::null_mut());

    let mut llvm_mem_buf: LLVMMemoryBufferRef = ptr::null_mut();
    assert_eq!(
        llvm::target_machine::LLVMTargetMachineEmitToMemoryBuffer(
            llvm_target_machine,
            llvm_module,
            llvm::target_machine::LLVMCodeGenFileType::LLVMObjectFile,
            ptr::null_mut(),
            &mut llvm_mem_buf,
        ),
        0
    );
    let llvm_out = slice::from_raw_parts(
        llvm::core::LLVMGetBufferStart(llvm_mem_buf) as *const _,
        llvm::core::LLVMGetBufferSize(llvm_mem_buf) as usize,
    );

    let mut object_file = File::create(object_path).unwrap();
    object_file.write_all(llvm_out).unwrap();
    drop(object_file);

    libc::free(llvm_triple_s as *mut libc::c_void);
    llvm::core::LLVMDisposeMemoryBuffer(llvm_mem_buf);
    llvm::target_machine::LLVMDisposeTargetMachine(llvm_target_machine);
    llvm::core::LLVMDisposeModule(llvm_module);
    llvm::core::LLVMContextDispose(llvm_ctx);
}

fn link(out_path: String, object_path: String) {
    assert!(
        Command::new("cc")
            .arg("-o")
            .arg(out_path)
            .arg(object_path)
            .spawn()
            .expect("could not invoke cc for linking")
            .wait()
            .unwrap()
            .success()
    );
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

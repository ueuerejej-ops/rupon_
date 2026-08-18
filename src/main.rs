extern crate core;

mod arena;
mod code_gen;
mod parser;
mod runtime;
mod token;

use crate::arena::Arena;

use crate::code_gen::{Compiler, run_jit};
use crate::parser::ready_code;
use inkwell::AddressSpace;

use crate::runtime::Runtime;
use inkwell::context::Context;

fn main() {
    let mut arena = Arena::new(1000);
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "arm_module");

    let print_type = context.void_type().fn_type(
        &[
            context.i8_type().ptr_type(AddressSpace::default()).into(),
            context.i8_type().ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );

    let _ = compiler.module.add_function("print", print_type, None);

    let code_my = r#"
       func fs(int num){
print("dd")
   }
    func main() {
    int i = 0
float ie = 23.23
float in = 32.88
    float iii  = 67 *2222.222

   while i != 10{
   i = i+1
   if i == 8{
     break
   }
   print("ee")
   }

  fs(23)
}

  "#;
    let stmt = ready_code(&mut arena, code_my);
    println!("{:#?}", stmt);
    compiler.parse_stmts(stmt);
    compiler.module.print_to_stderr();
    run_jit(&compiler.module, &mut Runtime::new());
}

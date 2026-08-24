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
    let mut arena = Arena::new(10000);
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "arm_module");

    let print_type = context.void_type().fn_type(
        &[
            context.i8_type().ptr_type(AddressSpace::default()).into(),
            context.i8_type().ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    let compare_str_type = context.bool_type().fn_type(
        &[
            context.i8_type().ptr_type(AddressSpace::default()).into(),
            context.i8_type().ptr_type(AddressSpace::default()).into(),
            context.i8_type().ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );

    let _ = compiler.module.add_function("print", print_type, None);
    let _ = compiler
        .module
        .add_function("str_cmp", compare_str_type, None);
    let code_my = r#"
func main() {
bool b = true

if 'j' == 'j'{
print("sd")
}
if true{
print("sd")
}
int i = 0
while true{
i = i+1

print("dsd")
if i == 3{
  break
}
}
if "as" == "as"{
print("sd")
}
}
  "#;
    let stmt = ready_code(&mut arena, code_my);
    println!("{:#?}", stmt);
    compiler.parse_stmts(stmt);
    compiler.module.print_to_stderr();
    run_jit(&compiler.module, &mut Runtime::new());
}

mod arena;
mod code_gen;
mod parser;
mod token;

use crate::arena::Arena;

use crate::code_gen::{Compiler, run_jit};
use crate::parser::ready_code;
use inkwell::AddressSpace;

use inkwell::context::Context;

fn main() {
    let mut arena = Arena::new(1000);
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "arm_module");

    let puts_type = context.i32_type().fn_type(
        &[context.i8_type().ptr_type(AddressSpace::default()).into()],
        false,
    );

    let _ = compiler.module.add_function("puts", puts_type, None);

    let code_my = r#"

func add( int a,int b){
return a+b

}
func turnten(int i){
i = 10
}
func main(){
puts("hello world")
}
  "#;
    let stmt = ready_code(&mut arena, code_my);
    compiler.parse_stmts(stmt.clone());
    compiler.module.print_to_stderr();

    run_jit(&compiler.module);
}

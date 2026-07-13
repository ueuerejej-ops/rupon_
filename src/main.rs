use fxhash::FxHashMap;

use inkwell::values::BasicValue;
use inkwell::values::PointerValue;
mod parser;
mod arena;
mod token;
use parser::Type;
use parser::Var;
struct Compiler<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    variables: SymbolHash<'ctx>,
    strint: StringInterner<'ctx>,
}


use crate::arena::Arena;
use crate::parser::Expr;
use crate::parser::Stmt;
use crate::parser::ready_code;
use core::panic;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{
   BasicValueEnum, 
};

#[derive(Debug, Clone)]
struct Varaibeldata<'ctx> {
    ty: Type,
    ptr: PointerValue<'ctx>,
}

impl<'a> StringInterner<'a> {
    fn new() -> Self {
        StringInterner {
            string: Vec::new(),
            map: FxHashMap::default(),
        }
    }

    fn itern(&mut self, name: &'a str) -> usize {
        if let Some(&id) = self.map.get(name) {
            return id;
        } else {
            let id = self.string.len();
            self.string.push(name);
            self.map.insert(name, id);
            id
        }
    }
    fn lookup(&self, name: &'a str) -> usize {
        if let Some(id) = self.map.get(name) {
            *id
        } else {
            panic!("cannot find that name")
        }
    }
}
struct StringInterner<'a> {
    string: Vec<&'a str>,
    map: FxHashMap<&'a str, usize>,
}

struct SymbolHash<'ctx> {
    variebles: FxHashMap<usize, Varaibeldata<'ctx>>,
}
impl<'ctx> SymbolHash<'ctx> {
    fn new() -> Self {
        SymbolHash {
            variebles: FxHashMap::default(),
        }
    }

    fn save(&mut self, var: Varaibeldata<'ctx>, id: usize) {
        self.variebles.insert(id, var);
    }
    fn get_var(&self, id: usize) -> Option<&Varaibeldata<'ctx>> {
        self.variebles.get(&id)
    }
}
impl<'ctx> Compiler<'ctx> {
    fn new(context: &'ctx Context, module_name: &str) -> Self {
        let compiler = Compiler {
            context,
            builder: context.create_builder(),
            module: context.create_module(module_name),
            variables: SymbolHash::new(),
            strint: StringInterner::new(),
        };
        compiler
    }
    fn get_var(&self,name: &str)->&Varaibeldata<'ctx>{
        let id = self.strint.lookup(name);
           self.variables.get_var(id).unwrap()
    }

fn get_expr_type(&self, expr: &Expr) -> Type {
    match expr {
        Expr::Num(_) => Type::Int,

        Expr::Id(name) => {
            self.get_var(name).ty.clone()
        }

        Expr::Str(_) => Type::Str,

        _ => panic!("unknown")
    }
}
    fn create_var<'a>(&mut self, name: &'ctx str, value: BasicValueEnum<'ctx>, ty: Type) {
              let ptr =   match ty{
                  Type::Int=>{
                    self.builder.build_alloca(self.context.i64_type(), name).unwrap()
                  }
                  Type::Str=>{
                    let ptr_ty = value.into_pointer_value().get_type();
                    self.builder.build_alloca(ptr_ty, name).unwrap()
                  }

                  
                };
         self.builder.build_store(ptr, value).unwrap();

         let id = self.strint.itern(name);

         self.variables.save(Varaibeldata { ty, ptr }, id);
        
    }

    fn get_value_of_expr<'a>(&mut self, expr: Expr<'a>) -> Option<BasicValueEnum<'ctx>> {
        match expr {
            Expr::Id(name)=>{
let var = self.get_var(name);
          match var.ty{
            Type::Int=>{
       let value = self.builder.build_load(
    self.context.i64_type(),
    var.ptr,
    "tmp",
).unwrap();

Some(value)
            }
          Type::Str => {
                    let value = self.builder
                        .build_load(
                            var.ptr.get_type(),
                            var.ptr,
                            "tmp"
                        )
                        .unwrap();

                    Some(value)
                }
          }

         
            }
            Expr::Num(n) => {
                let int_value = self.context.i64_type().const_int(n as u64, false);
                Some(int_value.as_basic_value_enum())
            }
            Expr::Str(str)=>{
                let pointer = self.builder.build_global_string_ptr(str,"DFDF").unwrap();

                Some(pointer.as_basic_value_enum())
            }
            _ => panic!("error expr"),
        }
    }
    fn  read_stmt<'a>(&mut self,var: Var<'a>) where 'a: 'ctx{
        unsafe{
          let  expr =  &*var.value;
        let expr_type = self.get_expr_type(expr);

        if var.tipe != expr_type{
            panic!("error type")
        }
   let basicvalue =  self.get_value_of_expr(expr.clone()).unwrap();

   match basicvalue{
    BasicValueEnum::IntValue(value)=>{
       self.create_var( var.name, BasicValueEnum::IntValue(value),Type::Int);
    }


    BasicValueEnum::PointerValue(str_poiner)=>{
        self.create_var(var.name,BasicValueEnum::PointerValue(str_poiner),Type::Str);
    }

    _=>panic!()
   }
        }

}
fn parse_stmt<'a>(&mut self,stmts: Vec<Stmt<'a>>) where 'a : 'ctx{
  for stmt in stmts{
    match stmt{
      Stmt::Int(var)| Stmt::Str(var)=>{
self.read_stmt(var);
  
       }
      
       
           _ =>  panic!("")
        
    }
  }
}



}

fn main() {
    let mut arena = Arena::new(1000);
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "arm_module");



    let fn_type = context.void_type().fn_type(&[], false);
    let function = compiler.module.add_function("test", fn_type, None);


    let entry = compiler.context.append_basic_block(function, "entry");

    compiler.builder.position_at_end(entry);
    let code_my = r#"str df = 3 int he = df

  "#;
let stmt = ready_code(&mut arena as *mut Arena ,code_my);
compiler.parse_stmt(stmt.clone());
println!("{:?}",stmt);
    compiler.module.print_to_stderr();
}


use fxhash::FxHashMap;

use inkwell::values::BasicValue;
use inkwell::values::PointerValue;
use inkwell::values::FunctionValue;

use crate::parser::Type;
use crate::parser::Func_call;
use crate::parser::Param;

use crate::parser::Var;
struct FunctionTable<'ctx> {
    funcs: FxHashMap<usize, FunctionValue<'ctx>>,
}


impl <'ctx>FunctionTable<'ctx>{
    fn new()->Self{
        Self { funcs:  FxHashMap::default()}
    }

}


pub struct Compiler<'ctx,'src> {
   pub context: &'ctx Context,
  pub  builder: Builder<'ctx>,
   pub module: Module<'ctx>,
   pub variables: SymbolHash<'ctx>,
   pub strint: StringInterner<'src>,
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

#[derive(Debug, Clone,PartialEq)]

pub struct Varaibeldata<'ctx> {
    pub ty: Type,
    ptr: PointerValue<'ctx>,
}

impl<'src> StringInterner<'src> {
    fn new() -> Self {
        StringInterner {
            string: Vec::new(),
            map: FxHashMap::default(),
        }
    }

   pub fn itern(&mut self, name: &'src str) -> usize {
        if let Some(&id) = self.map.get(name) {
            return id;
        } else {
            let id = self.string.len();
            self.string.push(name);
            self.map.insert(name, id);
            id
        }
    }
    
 pub   fn lookup(&self, name: &'src str) -> usize {
        if let Some(id) = self.map.get(name) {
            *id
        } else {
            panic!("cannot find that name")
        }
    }
}
#[derive(Debug, PartialEq)]

pub struct StringInterner<'src> {
    pub string: Vec<&'src str>,
    map: FxHashMap<&'src str, usize>,
}

#[derive(Debug, Clone,PartialEq)]

pub struct SymbolHash<'ctx> {
    variebles: FxHashMap<usize, Varaibeldata<'ctx>>,
}
impl<'ctx> SymbolHash<'ctx> {
    fn new() -> Self {
        SymbolHash {
            variebles: FxHashMap::default(),
        }
    }

   pub fn save(&mut self, var: Varaibeldata<'ctx>, id: usize) {
        self.variebles.insert(id, var);
    }
   pub fn get_var(&self, id: usize) -> Option<&Varaibeldata<'ctx>> {
        self.variebles.get(&id)
    }
}
impl<'ctx,'src> Compiler<'ctx,'src> {
   pub  fn new(context: &'ctx Context, module_name: &str) -> Self {
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
    fn create_var(&mut self, name: &'src str, value: BasicValueEnum<'ctx>, ty: Type) {
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
   pub  fn get_type_our_expr<'a>(&mut self,expr: Expr<'a>)->Type{
    match expr{
        Expr::Num(_)=>Type::Int,
        Expr::Str(_)=>Type::Str,
        Expr::Id(name)=>{
           let id = self.strint.lookup(name);
        let var = self.variables.get_var(id).unwrap().clone();
         var.ty
        } 
        _=>panic!("e")
    }
}
    fn func_create(&mut self,parametrs: Param,name: &str){

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
    fn  read_stmt(&mut self,var: Var<'src>){
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
fn do_return(&mut self,expr: *mut Expr){
unsafe {
    let expr = &mut *expr;
    let basic_value = self.get_value_of_expr(expr.clone()).unwrap();
    let _ = self.builder.build_return(Some(&basic_value));
}
}
fn assign_var(&mut self,name: &'src str,expr: *mut Expr<'src>){
   unsafe {
    if self.strint.string.contains(&name){
        let id = self.strint.lookup(name);
    let mut expr = &mut *expr;
        let expr_type = self.get_expr_type(expr);

    let basic_value = self.get_value_of_expr(expr.clone()).unwrap();
   let var = self.variables.get_var(id).unwrap();
   if var.ty != expr_type{
    panic!("type error")
   }
           self.builder.build_store(var.ptr, basic_value).unwrap();

    }
    

   }
}
pub fn parse_stmt(&mut self,stmts: Vec<Stmt<'src>>) {
  for stmt in stmts{
    match stmt{
      Stmt::Int(var)| Stmt::Str(var) =>{
self.read_stmt(var);
  
       }
      Stmt::Assign { name, value }=>{
       self.assign_var(name, value.clone());
    }
       Stmt::ReturnStmt(value)=>{
        self.do_return(value);
       }
           _ =>  panic!("")
        
    }
  }
}



}




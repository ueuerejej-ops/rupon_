use core::panic;
use std::marker::PhantomData;
use crate::parser::Expr;
use std::{alloc::{Layout, alloc}, env::set_var, ptr::{self, null}};
  pub struct Arena {
    buffer: *mut u8,
    capacity: usize,
    offset: usize,
    prev_offset: usize,
}


impl Arena{
  pub fn new(capacity: usize)->Self{
    let layout = Layout::from_size_align(capacity, 8).unwrap();
    let buffer = unsafe {
        alloc(layout)
    };
    Self { buffer, capacity, offset: 0, prev_offset: 0 }
  }
    pub fn alloc_expr<'a>(& mut self, expr: Expr<'a>) -> *mut Expr<'a> {
        self.alloc(expr)
    }

pub fn alloc<T>(&
  mut self,value: T)->* mut   T{
        let ptr = self.arena_alloc_align(
            std::mem::size_of::<T>(),
            std::mem::align_of::<T>(),
        ); 

  unsafe {  
    let mut ptr = ptr as *mut T;
    ptr.write(value);
    ptr
  }

}

fn arena_resize_align(&mut self,old_memory: *mut u8,old_size: usize,new_size: usize,align:usize)->* mut u8{
  let end = unsafe { self.buffer.add(self.capacity) };

  assert!(align.is_power_of_two());
  if old_memory.is_null() || old_size ==0{
    return self.arena_alloc_align(new_size, align);
  }else if self.buffer <= old_memory  && old_memory <end {
    if unsafe{self.buffer.add(self.prev_offset) == old_memory}{

    
      self.offset = self.prev_offset + new_size;

      if new_size > old_size{
           unsafe {std::ptr::write_bytes(old_memory, 0, new_size);}

      }
      return old_memory;

  }else{
    let new_ptr = self.arena_alloc_align(new_size, align);

let copy_size = old_size.min(new_size);
unsafe {std::ptr::copy(old_memory,new_ptr,copy_size);}
return new_ptr;
  }
}else{
  panic!()
}
}

   fn align_forward(& self,ptr: *mut u8,size: usize)->*mut u8{
    assert!(size.is_power_of_two());
    let mut  p = ptr as usize;
   let  a = size;


   let modulo = p & (a-1);

   if modulo != 0{
    p += a- modulo;
   }

   p as *mut u8

  }


  fn reset(&mut self){
     self.offset = 0;
  }



  
  fn arena_alloc_align (&mut  self,size: usize,align:usize)->*mut u8{
    let curr_ptr = (self.offset + self.buffer as usize) as * mut u8;
    let mut offsett = (self.align_forward(curr_ptr, align)) as usize;
    offsett -= self.buffer as usize;
    if offsett + size <= self.capacity{
      let mut ptr = unsafe{self.buffer.add(offsett) as *mut u8 } ;
      self.prev_offset = offsett;
      self.offset  = offsett + size;

    unsafe {  std::ptr::write_bytes(ptr, 0, size);};
return ptr;
    }
    panic!()
  }
  
    fn reset_and_clear(&mut  self) {
    unsafe {
        std::ptr::write_bytes(self.buffer, 0, self.capacity);
    }
    self.offset = 0;
}


}
pub fn alloc_arena_memeory<'a>(cap: usize)-> Arena {
    let mut arena = Arena::new(cap);

  arena
  
}
impl<'a> Drop for Arena {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 8).unwrap();
        unsafe {
            std::alloc::dealloc(self.buffer, layout);
        }
    }
}







pub fn expr_add<'a>(arena: * mut Arena,expr: Expr<'a>)->*mut Expr<'a>{
    unsafe {
      (*arena).alloc_expr(expr)
    }
}


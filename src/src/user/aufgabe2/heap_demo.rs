
use crate::devices::cga as cga;  
use crate::devices::cga_print;       
use crate::devices::key as key;     
use crate::devices::keyboard as keyboard;  
use crate::kernel::allocator as allocator;  
use alloc::{boxed::Box, vec::Vec};



// Hilfsfunktion: Auf Return-Taste warten
fn wait_for_return() {
	
	println!("");
	println!("");
    println!("Weiter mit <ENTER>");

   loop {
      let mut key: key::Key = keyboard::key_hit();
        
      if key.valid() == true {
		  if key.get_ascii() == 13 { break; }
      }
   }
}


fn demo() { 

    println!("Demo: Heap");
    //let test = Box::new(34);
    //allocator::dump_free_list();
    let mut v = Vec::new();


    for i in 0..10{ 
        v.push(i);
        allocator::dump_free_list();
        

    }
    allocator::dump_free_list();
    wait_for_return();

}



pub fn run () {
    allocator::init();
    //demo();

    /* Hier muss Code eingefuegt werden */

}

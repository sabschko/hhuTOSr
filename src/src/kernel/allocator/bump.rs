/*****************************************************************************
 *                                                                           *
 *                                 B U M P                                   *
 *                                                                           *
 *---------------------------------------------------------------------------*
 * Beschreibung:    Eine sehr einfache Heap-Verwaltung, welche freigegebenen *
 *                  Speicher nicht mehr nutzen kann.                         *
 *                                                                           *
 * Autor:           Philipp Oppermann                                        *
 *                  https://os.phil-opp.com/allocator-designs/               *
 *                  Modified by Michael Schoettner, 15.3.2022                *
 *****************************************************************************/

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;
use crate::devices::cga_print;


pub struct BumpAllocator {
   heap_start: usize,
   heap_end: usize,
   next: usize,
   allocations: usize,

  
   
}

impl BumpAllocator {
    // Creates a new empty bump allocator.
    pub const fn new() -> Self {

      BumpAllocator {
         heap_start: 0,
         heap_end: 0,
         next: 0, 
         allocations: 0,
     }

    }

    /*
     * Initializes the bump allocator with the given heap bounds.
     * 
     * This method is unsafe because the caller must ensure that the given
     *  memory range is unused. Also, this method must be called only once.
     */
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
      self.heap_start = heap_start;
      self.heap_end = heap_start + heap_size;
      self.next = heap_start;

    }

    // Dump free list
    pub fn dump_free_list(&mut self) {
      println!("BumpAllocator: heap_start={:#x}, heap_end={:#x}, next={:#x}, allocations={}", 
         self.heap_start, self.heap_end, self.next, self.allocations);

 		
	}

   pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {

     // let mut bump = self.lock(); // get a mutable reference

      let alloc_start = align_up(self.next, layout.align());
      let alloc_end = match alloc_start.checked_add(layout.size()) {
          Some(end) => end,
          None => return ptr::null_mut(),
      };

      if alloc_end > self.heap_end {
          ptr::null_mut() // out of memory
      } else {
          self.next = alloc_end;
          self.allocations += 1;
          alloc_start as *mut u8
      }

   }
   
   pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
      println!("   dealloc: size={}, align={}; not supported", layout.size(), layout.align());
   }

}

// Trait required by the Rust runtime for heap allocations
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
	
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock().dealloc(ptr, layout);
    }

}

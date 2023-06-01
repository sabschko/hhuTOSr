/*****************************************************************************
 *                                                                           *
 *                                 L I S T                                   *
 *                                                                           *
 *---------------------------------------------------------------------------*
 * Beschreibung:    Eine einfache Heap-Verwaltung, welche freie Bloecke      *
 *                  verkettet und wieder benutzen kann.                      *
 *                                                                           *
 * Autor:           Philipp Oppermann                                        *
 *                  https://os.phil-opp.com/allocator-designs/               *
 *                  Modified by Michael Schoettner, 15.3.2022                *
 *****************************************************************************/

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};


// metadata for each free memory block
struct ListNode {
	// size of the memory block
    size: usize,
    
    // &'static mut type semantically describes an owned object behind 
    // a pointer. Basically, it’s a Box without a destructor that frees 
    // the object at the end of the scope.
    next: Option<&'static mut ListNode>,
}


impl ListNode {
	
	// must be 'const', required later when constructing 
	// a static linked list allocator 
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    // return start address of memory block
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    // return end address of memory block
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}


pub struct LinkedListAllocator {
    head: ListNode,
    heap_start: usize,
    heap_end: usize,
}


impl LinkedListAllocator {
	
    // Creates an empty LinkedListAllocator.
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
            heap_start: 0,
            heap_end: 0,
        }
    }


    // Initialize the allocator with the given heap bounds.
    //
    // This function is unsafe because the caller must guarantee that 
    // the given heap bounds are valid and that the heap is unused.
    // This method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {

        self.add_free_block(heap_start, heap_size);
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;

    }


    // Adds the given free memory block 'addr' to the front of the free list.
    unsafe fn add_free_block(&mut self, addr: usize, size: usize) {
        // ensure that the freed region is capable of holding ListNode
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        // create a new list node and append it at the start of the list
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr)

    }
    
    
    // Search a free block with the given size and alignment and remove
    // it from the free list.
    //
    // Return: 'ListNode' or 'None'
    fn find_free_block(&mut self, size: usize, align: usize)
        -> Option<(&'static mut ListNode)>
    {
        
        let mut current = &mut self.head;

        while current.next.is_some() {

            // We have guarantee with loop predicate that current.next is not None
            let region = current.next.as_mut().unwrap();

            // [current] -> [region] -> [region.next] 
            if let Ok(allocation_start) = Self::check_block_for_alloc(&region, size, align){
                // [current] -> [region] -x- [region.next]
                let next = region.next.take();
                // [current] -x- [region] -x- [region.next]
                let ret = Some(current.next.take().unwrap());
                // [current] -- [region] ->  [region.next]
                // Transfer and regions next pointer to current.
                // Thereby isolating region from the free list ergo. deleting it.
                current.next = next;
                return ret;
            } else {
                current = current.next.as_mut().unwrap();
            }

        }

        return None
    


    }
    
    
    // Check if the given 'block' is large enough for an allocation with  
    // 'size' and alignment 'align'
    //
    // Return: OK(allocation start address) or Err 
    fn check_block_for_alloc(block: &ListNode, size: usize, align: usize)
        -> Result<usize, ()>
    {

        let alloc_start = align_up(block.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > block.end_addr() {
            // region too small
            return Err(());
        }

        let excess_size = block.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            // rest of region too small to hold a ListNode (required because the
            // allocation splits the region in a used and a free part)
            return Err(());
        }

        // region suitable for allocation
        Ok(alloc_start)

    }


    // Dump free list
    pub fn dump_free_list(&mut self) {
        let mut current = &mut self.head;
        println!("Heap-Start: 0x{:x}, Heap-End: 0x{:x}", 
            self.heap_start, 
            self.heap_end);

        while current.next.is_some(){
            current = current.next.as_mut().unwrap();
            println!("Block-Start: 0x{:x}, Block-End: 0x{:x}, Block-Groesse:{}", 
                current.start_addr(), 
                current.end_addr(), 
                current.size);
            
        }
    }

    
    // Adjust the given layout so that the resulting allocated memory
    // block is also capable of storing a `ListNode`.
    //
    // Returns the adjusted size and alignment as a (size, align) tuple.
    fn size_align(layout: Layout) -> (usize, usize) {
	    let layout = layout
             .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
    
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
       // print!("   alloc: size={}, align={}", layout.size(), layout.align());

       // perform layout adjustments
       let (size, align) = LinkedListAllocator::size_align(layout);
       let ret_ptr: *mut u8; 

       if let Some(block) = self.find_free_block(size, align) {
           let alloc_end = block.start_addr().checked_add(size).expect("overflow");
            
           // the remaining memory will be inserted as new block
           // the size is large enough to store metadata; this is 
           // checked in 'check_block_for_alloc' called by 'find_free_block'
           let remaining_block_size = block.end_addr() - alloc_end;
           if remaining_block_size > 0 {
               self.add_free_block(alloc_end, remaining_block_size);
           }
           ret_ptr = block.start_addr() as *mut u8;
           // println!(", returning addr=0x{:x}", block.start_addr());
       } else {
           // println!(", *** out of memory ***");
           ret_ptr = ptr::null_mut(); // out of memory
       }
       ret_ptr
   }
    
   pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
   // println!("   dealloc: size={}, align={}; not supported", layout.size(), layout.align());

      let (size, _) = LinkedListAllocator::size_align(layout);
      self.add_free_block(ptr as usize, size)
   }

}

// Trait required by the Rust runtime for heap allocations
unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock().dealloc(ptr, layout);
    }
    
}

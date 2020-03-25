use crate::traits::{Allocator, OwnerCheck};
use core::{
    alloc::{AllocErr, Layout},
    ptr::{drop_in_place, NonNull},
};

struct AllocatorBlock<A> {
    pub allocator: A,
    pub next: Option<NonNull<AllocatorBlock<A>>>,
}

pub struct LinkedChain<A, B>
where
    A: Allocator + OwnerCheck + Default,
    B: Allocator,
{
    head: Option<NonNull<AllocatorBlock<A>>>,
    backing: B,
}

/// A composing allocator
///
/// It maintains a linked list of allocators of type A, created from allocator B
///
/// When a request is made, it walks its list of allocators until one can fulfill the request.
/// If no allocator can, a new A is created with memory allocated from B
impl<A, B> LinkedChain<A, B>
where
    A: Allocator + OwnerCheck + Default,
    B: Allocator,
{
    /// The layout of internal allocation blocks
    ///
    /// Allocations of this layout will be made to the backing allocator
    pub const fn block_layout() -> Layout {
        Layout::new::<AllocatorBlock<A>>()
    }

    pub fn new(backing: B) -> Self {
        LinkedChain {
            head: None,
            backing,
        }
    }

    fn allocate_new_block(
        &mut self,
    ) -> Result<NonNull<AllocatorBlock<A>>, AllocErr> {
        // Allocate a new block from the backing store
        let memory = self.backing.alloc(Self::block_layout())?.0;

        // Write the old head to the new block
        let memory = memory.cast::<AllocatorBlock<A>>();
        unsafe {
            memory.as_ptr().write(AllocatorBlock {
                allocator: A::default(),
                next: self.head.take(),
            });
        }

        // Make the new block into the head
        self.head = Some(memory);

        Ok(memory)
    }

    fn walk_chain_mut<CB, R>(&mut self, mut callback: CB) -> Option<R>
    where
        CB: FnMut(&mut AllocatorBlock<A>) -> Option<R>,
    {
        let mut chain = self.head;

        while let Some(mut it) = chain {
            unsafe {
                let it = it.as_mut();
                chain = it.next;

                if let Some(result) = callback(it) {
                    return Some(result);
                }
            }
        }

        None
    }

    fn walk_chain<CB, R>(&self, mut callback: CB) -> Option<R>
    where
        CB: FnMut(&AllocatorBlock<A>) -> Option<R>,
    {
        let mut chain = self.head;

        while let Some(it) = chain {
            unsafe {
                let it = it.as_ref();
                chain = it.next;

                if let Some(result) = callback(it) {
                    return Some(result);
                }
            }
        }

        None
    }
}

impl<A, B> Drop for LinkedChain<A, B>
where
    A: Allocator + OwnerCheck + Default,
    B: Allocator,
{
    fn drop(&mut self) {
        let mut chain = self.head.take();

        while let Some(mut it) = chain {
            // Save the next pointer
            unsafe {
                chain = it.as_mut().next.take();
            }

            // Drop the value at the location
            unsafe {
                drop_in_place(it.as_ptr());
            }

            // Deallocate the block
            unsafe {
                self.backing.dealloc(it.cast(), Self::block_layout());
            }
        }
    }
}

impl<A, B> Allocator for LinkedChain<A, B>
where
    A: Allocator + OwnerCheck + Default,
    B: Allocator,
{
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        // Walk the chain and try to find an allocator that can allocate something
        if let Some(allocation) =
            self.walk_chain_mut(|block| block.allocator.alloc(layout).ok())
        {
            return Ok(allocation);
        }

        // If no existing allocator could fulfill the request, allocate a new one
        let mut new_guy = self.allocate_new_block()?;

        // And let the new guy try it
        unsafe { new_guy.as_mut().allocator.alloc(layout) }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let result = self.walk_chain_mut(|block| {
            if block.allocator.is_owner(ptr, layout) {
                block.allocator.dealloc(ptr, layout);
                Some(())
            } else {
                None
            }
        });

        // If this is properly used, only an owned value should be passed to self
        // and one of our allocators should have recognized it
        debug_assert!(result.is_some());
    }
}

impl<A, B> OwnerCheck for LinkedChain<A, B>
where
    A: Allocator + OwnerCheck + Default,
    B: Allocator,
{
    fn is_owner(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        self.walk_chain(|block| {
            if block.allocator.is_owner(ptr, layout) {
                Some(())
            } else {
                None
            }
        })
        .is_some()
    }
}

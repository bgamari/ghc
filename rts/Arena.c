/* -----------------------------------------------------------------------------
   (c) The University of Glasgow 2001

   Arena allocation.  Arenas provide fast memory allocation at the
   expense of fine-grained recycling of storage: memory may be
   only be returned to the system by freeing the entire arena, it
   isn't possible to return individual objects within an arena.

   Do not assume that sequentially allocated objects will be adjacent
   in memory.

   Quirks: this allocator makes use of the RTS block allocator.  If
   the current block doesn't have enough room for the requested
   object, then a new block is allocated.  This means that allocating
   large objects will tend to result in wasted space at the end of
   each block.  In the worst case, half of the allocated space is
   wasted.  This allocator is therefore best suited to situations in
   which most allocations are small.
   -------------------------------------------------------------------------- */

#include "rts/PosixSource.h"
#include "Rts.h"

#include "RtsUtils.h"
#include "Arena.h"

typedef struct ArenaBlock {
    struct ArenaBlock *link;
    StgPtr start;
}ArenaBlock;

// Each arena struct is allocated using malloc().
struct _Arena {
    ArenaBlock *current;
    StgWord *free;              // ptr to next free byte in current block
    StgWord *lim;               // limit (== last free byte + 1)
};

// We like to keep track of how many blocks we've allocated for
// Storage.c:memInventory().
static long arena_blocks = 0;

// Begin a new arena
Arena *
newArena( void )
{
    Arena *arena;

    arena = stgMallocBytes(sizeof(Arena), "newArena");
    // arena->current = allocBlock_lock();
    // MMTK: use malloc instead of block alloc
    arena->current = stgMallocBytes(sizeof(ArenaBlock), "newArena");
    // arena->current->start = malloc(BLOCK_SIZE);
    // arena->current->start = mmtk_alloc(BLOCK_SIZE, )
    arena->current->link = NULL;
    arena->free = arena->current->start;
    arena->lim  = arena->current->start + BLOCK_SIZE_W;
    arena_blocks++;

    return arena;
}

// The minimum alignment of an allocated block.
#define MIN_ALIGN 8

/* 'n' is assumed to be a power of 2 */
#define ROUNDUP(x,n)  (((x)+((n)-1))&(~((n)-1)))
#define B_TO_W(x)     ((x) / sizeof(W_))

// Allocate some memory in an arena
void  *
arenaAlloc( Arena *arena, size_t size )
{
    void *p;
    uint32_t size_w;
    uint32_t req_blocks;
    bdescr *bd;

    // round up to nearest alignment chunk.
    size = ROUNDUP(size,MIN_ALIGN);

    // size of allocated block in words.
    size_w = B_TO_W(size);

    if ( arena->free + size_w < arena->lim ) {
        // enough room in the current block...
        p = arena->free;
        arena->free += size_w;
        return p;
    } else {
        // allocate a fresh block...

        uint32_t req_blocks;
        ArenaBlock *new_;

        req_blocks =  (W_)BLOCK_ROUND_UP(size) / BLOCK_SIZE;
        new_ = stgMallocBytes(sizeof(ArenaBlock), "newArena");
        new_->start = malloc(BLOCK_SIZE * req_blocks);
        new_->link = arena->current;

        arena_blocks += req_blocks;
        arena->current = new_;
        arena->free = new_->start + size_w;
        arena->lim = new_->start + req_blocks * BLOCK_SIZE_W;

        return new_->start;

    }
}

// Free an entire arena
void
arenaFree( Arena *arena )
{
    ArenaBlock *bd, *next;

    for (bd = arena->current; bd != NULL; bd = next) {
        next = bd->link;
        free(bd->start);
        stgFree(bd);
    }
    stgFree(arena);
}

unsigned long
arenaBlocks( void )
{
    return arena_blocks;
}

#if defined(DEBUG)
void checkPtrInArena( StgPtr p, Arena *arena )
{
    // We don't update free pointers of arena blocks, so we have to check cached
    // free pointer for the first block.
    if (p >= arena->current->start && p < arena->free) {
        return;
    }

    // Rest of the blocks should be full (except there may be a little bit of
    // slop at the end). Again, free pointers are not updated so we can't use
    // those.
    for (ArenaBlock *bd = arena->current->link; bd; bd = bd->link) {
        if (p >= bd->start && p < bd->start + BLOCK_SIZE_W) {
            return;
        }
    }

    barf("Location %p is not in arena %p", (void*)p, (void*)arena);
}
#endif

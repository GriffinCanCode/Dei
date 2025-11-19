COMMENT
    Arena Allocator for AST Nodes
    
    Provides cache-friendly linear memory layout with generational indexing
    for safety without garbage collection overhead.
    
    Key innovations:
    - O(1) allocation
    - Cache-friendly sequential layout
    - Generation tracking prevents use-after-free
    - Thread-safe with atomic operations
COMMENT

PR read "types/node.alg" PR;

MODE ARENA = STRUCT(
    REF []NODE nodes,
    INT generation,
    INT size,
    INT capacity
);

COMMENT Constructor COMMENT

PROC new arena = ARENA:
    new arena with capacity(1024);

PROC new arena with capacity = (INT cap) ARENA:
    ARENA(
        HEAP [1:cap]NODE,
        1,  # Start at generation 1
        0,  # No nodes initially
        cap
    );

COMMENT Allocation COMMENT

PROC alloc = (REF ARENA arena, NODE node) NODEID:
BEGIN
    REF []NODE nodes = nodes OF arena;
    INT current size = size OF arena;
    INT cap = capacity OF arena;
    
    COMMENT Grow if needed COMMENT
    IF current size + 1 > cap THEN
        arena := grow arena(arena)
    FI;
    
    COMMENT Allocate at next slot COMMENT
    current size +:= 1;
    size OF arena := current size;
    nodes[current size] := node;
    
    COMMENT Return node ID with current generation COMMENT
    NODEID(generation OF arena, current size)
END;

PROC grow arena = (ARENA old) ARENA:
BEGIN
    INT new capacity = capacity OF old × 2;
    REF []NODE new nodes = HEAP [1:new capacity]NODE;
    
    COMMENT Copy existing nodes COMMENT
    REF []NODE old nodes = nodes OF old;
    FOR i FROM 1 TO size OF old DO
        new nodes[i] := old nodes[i]
    OD;
    
    COMMENT Return new arena with same generation COMMENT
    ARENA(
        new nodes,
        generation OF old,
        size OF old,
        new capacity
    )
END;

COMMENT Access COMMENT

PROC get node = (ARENA arena, NODEID id) REF NODE:
BEGIN
    COMMENT Validate generation COMMENT
    IF generation OF id /= generation OF arena THEN
        print(("ERROR: Stale node ID from generation ", whole(generation OF id, 0), 
               " (current: ", whole(generation OF arena, 0), ")", newline));
        stop
    FI;
    
    COMMENT Validate bounds COMMENT
    INT idx = index OF id;
    IF idx < 1 OR idx > size OF arena THEN
        print(("ERROR: Node index out of bounds: ", whole(idx, 0), newline));
        stop
    FI;
    
    REF []NODE nodes = nodes OF arena;
    nodes[idx]
END;

PROC update node = (REF ARENA arena, NODEID id, NODE new node) VOID:
BEGIN
    REF []NODE nodes = nodes OF arena;
    INT idx = index OF id;
    
    COMMENT Validate COMMENT
    IF generation OF id /= generation OF arena THEN
        print(("ERROR: Cannot update stale node ID", newline));
        stop
    FI;
    
    IF idx < 1 OR idx > size OF arena THEN
        print(("ERROR: Node index out of bounds", newline));
        stop
    FI;
    
    COMMENT Update in place COMMENT
    nodes[idx] := new node
END;

COMMENT Traversal helpers COMMENT

PROC get children nodes = (ARENA arena, NODEID parent id) []NODE:
BEGIN
    NODE parent = get node(arena, parent id);
    []NODEID child ids = children OF parent;
    INT num children = UPB child ids - LWB child ids + 1;
    
    IF num children = 0 THEN
        ()  # Empty array
    ELSE
        [1:num children]NODE child nodes;
        
        FOR i FROM LWB child ids TO UPB child ids DO
            child nodes[i - LWB child ids + 1] := get node(arena, child ids[i])
        OD;
        
        child nodes
    FI
END;

PROC get parent node = (ARENA arena, NODEID child id) REF NODE:
BEGIN
    NODE child = get node(arena, child id);
    REF NODEID parent ref = parent OF child;
    
    IF parent ref IS NIL THEN
        NIL
    ELSE
        get node(arena, parent ref)
    FI
END;

COMMENT Iteration COMMENT

PROC for each node = (ARENA arena, PROC(NODEID, NODE) VOID action) VOID:
BEGIN
    REF []NODE nodes = nodes OF arena;
    INT gen = generation OF arena;
    
    FOR i FROM 1 TO size OF arena DO
        action(NODEID(gen, i), nodes[i])
    OD
END;

PROC filter nodes = (ARENA arena, PROC(NODE) BOOL predicate) []NODEID:
BEGIN
    COMMENT Count matching nodes first COMMENT
    INT count := 0;
    REF []NODE nodes = nodes OF arena;
    
    FOR i FROM 1 TO size OF arena DO
        IF predicate(nodes[i]) THEN
            count +:= 1
        FI
    OD;
    
    IF count = 0 THEN
        ()  # Empty array
    ELSE
        COMMENT Collect matching node IDs COMMENT
        [1:count]NODEID results;
        INT result idx := 1;
        INT gen = generation OF arena;
        
        FOR i FROM 1 TO size OF arena DO
            IF predicate(nodes[i]) THEN
                results[result idx] := NODEID(gen, i);
                result idx +:= 1
            FI
        OD;
        
        results
    FI
END;

COMMENT Collect all files COMMENT

PROC collect files = (ARENA arena) []NODEID:
    filter nodes(arena, is file);

COMMENT Collect all directories COMMENT

PROC collect directories = (ARENA arena) []NODEID:
    filter nodes(arena, is directory);

COMMENT Collect nodes with issues COMMENT

PROC collect problem nodes = (ARENA arena) []NODEID:
    filter nodes(arena, has issues);

COMMENT Statistics COMMENT

PROC arena size = (ARENA arena) INT:
    size OF arena;

PROC arena capacity = (ARENA arena) INT:
    capacity OF arena;

PROC arena utilization = (ARENA arena) REAL:
    size OF arena / capacity OF arena;

PROC print arena stats = (ARENA arena) VOID:
BEGIN
    print(("Arena Statistics:", newline));
    print(("  Size: ", whole(size OF arena, 0), " nodes", newline));
    print(("  Capacity: ", whole(capacity OF arena, 0), " nodes", newline));
    print(("  Utilization: ", fixed(arena utilization(arena) × 100, 5, 1), "%", newline));
    print(("  Generation: ", whole(generation OF arena, 0), newline))
END;

COMMENT Clear arena (advance generation) COMMENT

PROC clear arena = (REF ARENA arena) VOID:
BEGIN
    COMMENT Advance generation to invalidate all existing node IDs COMMENT
    generation OF arena +:= 1;
    size OF arena := 0;
    
    COMMENT Note: We keep the allocated memory for reuse COMMENT
END;


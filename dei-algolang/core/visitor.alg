COMMENT
    Visitor Pattern for AST Traversal
    
    Provides a clean abstraction for walking the AST and performing actions
    on nodes without modifying the core structures.
COMMENT

PR read "types/node.alg" PR;
PR read "core/arena.alg" PR;

COMMENT Visitor interface COMMENT

MODE VISITOR = STRUCT(
    PROC(REF ARENA, NODE) VOID visit directory,
    PROC(REF ARENA, NODE) VOID visit file,
    PROC(REF ARENA, NODE) VOID before children,
    PROC(REF ARENA, NODE) VOID after children
);

COMMENT Default no-op visitor COMMENT

PROC noop visit directory = (REF ARENA arena, NODE node) VOID: SKIP;
PROC noop visit file = (REF ARENA arena, NODE node) VOID: SKIP;
PROC noop before children = (REF ARENA arena, NODE node) VOID: SKIP;
PROC noop after children = (REF ARENA arena, NODE node) VOID: SKIP;

PROC default visitor = VISITOR:
    VISITOR(
        noop visit directory,
        noop visit file,
        noop before children,
        noop after children
    );

COMMENT Traversal strategies COMMENT

PROC walk depth first = (REF ARENA arena, NODEID root, VISITOR visitor) VOID:
BEGIN
    NODE node = get node(arena, root);
    
    COMMENT Visit node based on type COMMENT
    IF is directory(node) THEN
        visit directory OF visitor(arena, node)
    ELIF is file(node) THEN
        visit file OF visitor(arena, node)
    FI;
    
    COMMENT Before children hook COMMENT
    before children OF visitor(arena, node);
    
    COMMENT Recursively visit children COMMENT
    []NODEID child ids = children OF node;
    FOR i FROM LWB child ids TO UPB child ids DO
        walk depth first(arena, child ids[i], visitor)
    OD;
    
    COMMENT After children hook COMMENT
    after children OF visitor(arena, node)
END;

PROC walk breadth first = (REF ARENA arena, NODEID root, VISITOR visitor) VOID:
BEGIN
    COMMENT Queue of nodes to visit COMMENT
    [1:arena size(arena)]NODEID queue;
    INT queue front := 1;
    INT queue back := 1;
    
    COMMENT Enqueue root COMMENT
    queue[queue back] := root;
    queue back +:= 1;
    
    COMMENT Process queue COMMENT
    WHILE queue front < queue back DO
        NODEID current id = queue[queue front];
        queue front +:= 1;
        
        NODE current = get node(arena, current id);
        
        COMMENT Visit node COMMENT
        IF is directory(current) THEN
            visit directory OF visitor(arena, current)
        ELIF is file(current) THEN
            visit file OF visitor(arena, current)
        FI;
        
        COMMENT Enqueue children COMMENT
        []NODEID child ids = children OF current;
        FOR i FROM LWB child ids TO UPB child ids DO
            queue[queue back] := child ids[i];
            queue back +:= 1
        OD
    OD
END;

COMMENT Specialized visitors COMMENT

COMMENT Visitor that collects all file paths COMMENT

PROC make file collector = []STRING:
BEGIN
    [1:10000]STRING collected;
    INT count := 0;
    
    PROC collect file = (REF ARENA arena, NODE node) VOID:
    BEGIN
        count +:= 1;
        collected[count] := path OF node
    END;
    
    VISITOR(
        noop visit directory,
        collect file,
        noop before children,
        noop after children
    );
    
    COMMENT Return collected paths COMMENT
    [1:count]STRING results;
    FOR i FROM 1 TO count DO
        results[i] := collected[i]
    OD;
    results
END;

COMMENT Visitor that prints tree structure COMMENT

INT print depth := 0;  # Track current depth

PROC make tree printer = VISITOR:
BEGIN
    PROC print directory = (REF ARENA arena, NODE node) VOID:
    BEGIN
        FOR i FROM 1 TO print depth DO
            print(("  "))
        OD;
        print(("📁 ", name OF node, "/", newline))
    END;
    
    PROC print file = (REF ARENA arena, NODE node) VOID:
    BEGIN
        FOR i FROM 1 TO print depth DO
            print(("  "))
        OD;
        
        COMMENT Add symbol based on issues COMMENT
        STRING symbol = IF has issues(node) THEN "❌" ELSE "📄" FI;
        print((symbol, " ", name OF node, newline))
    END;
    
    PROC increment depth = (REF ARENA arena, NODE node) VOID:
        print depth +:= 1;
    
    PROC decrement depth = (REF ARENA arena, NODE node) VOID:
        print depth -:= 1;
    
    VISITOR(
        print directory,
        print file,
        increment depth,
        decrement depth
    )
END;

COMMENT Visitor that filters and collects nodes COMMENT

PROC make filter visitor = (PROC(NODE) BOOL predicate) VISITOR:
BEGIN
    [1:10000]NODE collected;
    INT count := 0;
    
    PROC check and collect = (REF ARENA arena, NODE node) VOID:
    BEGIN
        IF predicate(node) THEN
            count +:= 1;
            collected[count] := node
        FI
    END;
    
    VISITOR(
        check and collect,
        check and collect,
        noop before children,
        noop after children
    )
END;

COMMENT Visitor that counts nodes matching a condition COMMENT

PROC count matching = (REF ARENA arena, NODEID root, PROC(NODE) BOOL predicate) INT:
BEGIN
    INT count := 0;
    
    PROC increment if match = (REF ARENA arena, NODE node) VOID:
    BEGIN
        IF predicate(node) THEN
            count +:= 1
        FI
    END;
    
    VISITOR counter = VISITOR(
        increment if match,
        increment if match,
        noop before children,
        noop after children
    );
    
    walk depth first(arena, root, counter);
    count
END;

COMMENT Action on all nodes matching predicate COMMENT

PROC for each matching = (REF ARENA arena, NODEID root, PROC(NODE) BOOL predicate, 
                          PROC(REF ARENA, NODE) VOID action) VOID:
BEGIN
    PROC check and act = (REF ARENA arena, NODE node) VOID:
    BEGIN
        IF predicate(node) THEN
            action(arena, node)
        FI
    END;
    
    VISITOR actor = VISITOR(
        check and act,
        check and act,
        noop before children,
        noop after children
    );
    
    walk depth first(arena, root, actor)
END;


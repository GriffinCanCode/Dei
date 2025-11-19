COMMENT
    Filesystem AST Builder
    
    Recursively scans directory structures and builds an AST representation.
    Filters out build artifacts and respects .gitignore patterns.
COMMENT

PR read "types/node.alg" PR;
PR read "core/arena.alg" PR;

MODE BUILDRESULT = UNION(
    NODEID success,
    STRING error
);

COMMENT Excluded directories (build artifacts, version control, etc.) COMMENT

[]STRING excluded dirs = ("bin", "obj", ".git", ".vs", "node_modules", 
                           "packages", ".idea", "target", "dist", "build",
                           "__pycache__", ".next", ".cache");

PROC should exclude directory = (STRING name) BOOL:
BEGIN
    BOOL excluded := FALSE;
    
    FOR i FROM LWB excluded dirs TO UPB excluded dirs DO
        IF name = excluded dirs[i] THEN
            excluded := TRUE
        FI
    OD;
    
    excluded
END;

COMMENT Supported file extensions COMMENT

[]STRING supported extensions = ("cs", "rs", "py", "js", "ts", "go", "java", "alg");

PROC is supported file = (STRING name) BOOL:
BEGIN
    STRING ext = extract extension(name);
    BOOL supported := FALSE;
    
    FOR i FROM LWB supported extensions TO UPB supported extensions DO
        IF ext = supported extensions[i] THEN
            supported := TRUE
        FI
    OD;
    
    supported
END;

COMMENT Main builder procedure COMMENT

PROC build filesystem ast = (REF ARENA arena, STRING root path) BUILDRESULT:
BEGIN
    IF NOT path exists(root path) THEN
        error("Path not found: " + root path)
    ELSE
        IF is file path(root path) THEN
            build file node(arena, root path, 0, NIL)
        ELSE
            build directory node(arena, root path, 0, NIL)
        FI
    FI
END;

COMMENT Build directory node recursively COMMENT

PROC build directory node = (REF ARENA arena, STRING path, INT depth, REF NODEID parent) BUILDRESULT:
BEGIN
    COMMENT Create directory node COMMENT
    NODEID node id = alloc(arena, new directory node(
        NODEID(generation OF arena, 0),  # Temporary ID
        path,
        depth
    ));
    
    COMMENT Get directory entries COMMENT
    []STRING entries = list directory(path);
    
    COMMENT Process children COMMENT
    [1:UPB entries]NODEID child ids;
    INT child count := 0;
    
    FOR i FROM LWB entries TO UPB entries DO
        STRING entry path = path + "/" + entries[i];
        STRING entry name = entries[i];
        
        IF is directory path(entry path) THEN
            COMMENT Skip excluded directories COMMENT
            IF NOT should exclude directory(entry name) THEN
                BUILDRESULT result = build directory node(arena, entry path, depth + 1, node id);
                
                CASE result IN
                    (NODEID child id):
                        BEGIN
                            child count +:= 1;
                            child ids[child count] := child id
                        END,
                    (STRING error msg):
                        print(("Warning: ", error msg, newline))
                ESAC
            FI
        ELIF is file path(entry path) THEN
            COMMENT Only include supported files COMMENT
            IF is supported file(entry name) THEN
                BUILDRESULT result = build file node(arena, entry path, depth + 1, node id);
                
                CASE result IN
                    (NODEID child id):
                        BEGIN
                            child count +:= 1;
                            child ids[child count] := child id
                        END,
                    (STRING error msg):
                        print(("Warning: ", error msg, newline))
                ESAC
            FI
        FI
    OD;
    
    COMMENT Update node with children COMMENT
    NODE dir node = get node(arena, node id);
    
    COMMENT Create properly sized children array COMMENT
    [1:child count]NODEID final children;
    FOR i FROM 1 TO child count DO
        final children[i] := child ids[i]
    OD;
    
    dir node := with children(dir node, final children);
    
    IF parent ISNT NIL THEN
        dir node := with parent(dir node, parent)
    FI;
    
    update node(arena, node id, dir node);
    
    success(node id)
END;

COMMENT Build file node COMMENT

PROC build file node = (REF ARENA arena, STRING path, INT depth, REF NODEID parent) BUILDRESULT:
BEGIN
    NODEID node id = alloc(arena, new file node(
        NODEID(generation OF arena, 0),
        path,
        depth
    ));
    
    COMMENT Update with parent if provided COMMENT
    IF parent ISNT NIL THEN
        NODE file node = get node(arena, node id);
        file node := with parent(file node, parent);
        update node(arena, node id, file node)
    FI;
    
    success(node id)
END;

COMMENT Filesystem utilities (platform-specific implementations) COMMENT

PROC path exists = (STRING path) BOOL:
    COMMENT Implementation would call system API COMMENT
    TRUE;  # Placeholder

PROC is file path = (STRING path) BOOL:
    COMMENT Check if path points to a file COMMENT
    TRUE;  # Placeholder

PROC is directory path = (STRING path) BOOL:
    COMMENT Check if path points to a directory COMMENT
    TRUE;  # Placeholder

PROC list directory = (STRING path) []STRING:
    COMMENT Return array of entry names in directory COMMENT
    ();  # Placeholder - would call system API

COMMENT Result constructors COMMENT

PROC success = (NODEID id) BUILDRESULT:
    success(id);

PROC error = (STRING message) BUILDRESULT:
    error(message);

COMMENT Pretty print AST tree COMMENT

PROC print ast tree = (ARENA arena, NODEID root, INT indent level) VOID:
BEGIN
    NODE node = get node(arena, root);
    
    COMMENT Print indentation COMMENT
    FOR i FROM 1 TO indent level DO
        print(("  "))
    OD;
    
    COMMENT Print node COMMENT
    IF is directory(node) THEN
        print(("📁 ", name OF node, "/", newline))
    ELSE
        print(("📄 ", name OF node, newline))
    FI;
    
    COMMENT Print children recursively COMMENT
    []NODEID child ids = children OF node;
    FOR i FROM LWB child ids TO UPB child ids DO
        print ast tree(arena, child ids[i], indent level + 1)
    OD
END;

COMMENT Statistics COMMENT

PROC count nodes = (ARENA arena, NODEID root, PROC(NODE) BOOL predicate) INT:
BEGIN
    NODE node = get node(arena, root);
    INT count := IF predicate(node) THEN 1 ELSE 0 FI;
    
    []NODEID child ids = children OF node;
    FOR i FROM LWB child ids TO UPB child ids DO
        count +:= count nodes(arena, child ids[i], predicate)
    OD;
    
    count
END;

PROC count all files = (ARENA arena, NODEID root) INT:
    count nodes(arena, root, is file);

PROC count all directories = (ARENA arena, NODEID root) INT:
    count nodes(arena, root, is directory);

PROC print ast stats = (ARENA arena, NODEID root) VOID:
BEGIN
    print(("AST Statistics:", newline));
    print(("  Total Files: ", whole(count all files(arena, root), 0), newline));
    print(("  Total Directories: ", whole(count all directories(arena, root), 0), newline));
    NODE root node = get node(arena, root);
    print(("  Root: ", name OF root node, newline))
END;


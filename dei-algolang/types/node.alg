COMMENT
    AST Node Definitions
    
    Defines the core data structures for representing filesystem and code ASTs.
    Uses arena allocation with generational indexing for safe, efficient traversal.
COMMENT

COMMENT Type definitions COMMENT

MODE NODEID = STRUCT(
    INT generation,  # Generation for safety (detect use-after-free)
    INT index       # Index into arena
);

MODE NODEKIND = UNION(
    VOID directory,
    VOID file
);

MODE LANGUAGE = UNION(
    VOID rust,
    VOID csharp,
    VOID python,
    VOID javascript,
    VOID typescript,
    VOID go,
    VOID java,
    VOID unknown
);

COMMENT Forward declarations COMMENT
MODE FILEMETRICS = STRUCT(
    STRING path,
    INT lines,
    REF []CLASSMETRICS classes
);

MODE ANALYSISRESULT = STRUCT(
    REF CLASSMETRICS class metrics,
    BOOL is god class,
    REF []RESPONSIBILITYCLUSTER suggested extractions,
    REF []GODMETHODRESULT god methods,
    STRING summary,
    INT analyzed at  # Unix timestamp
);

MODE GODFILERESULT = STRUCT(
    STRING file path,
    INT class count,
    INT total lines,
    []STRING class names,
    []STRING violations
);

COMMENT Main AST Node COMMENT
MODE NODE = STRUCT(
    NODEID id,
    NODEKIND kind,
    STRING path,
    STRING name,
    INT depth,
    REF NODEID parent,  # REF NODEID allows NULL (NIL)
    []NODEID children,
    
    COMMENT Analysis results (populated during traversal) COMMENT
    REF FILEMETRICS file metrics,
    REF []ANALYSISRESULT analysis results,
    REF GODFILERESULT god file result
);

COMMENT Constructors COMMENT

PROC new directory node = (NODEID id, STRING path, INT depth) NODE:
(
    NODE(
        id,
        directory,
        path,
        extract filename(path),
        depth,
        NIL,  # No parent initially
        (),   # Empty children array
        NIL,  # No file metrics
        NIL,  # No analysis results
        NIL   # No god file result
    )
);

PROC new file node = (NODEID id, STRING path, INT depth) NODE:
(
    NODE(
        id,
        file,
        path,
        extract filename(path),
        depth,
        NIL,
        (),
        NIL,
        NIL,
        NIL
    )
);

COMMENT Accessors and predicates COMMENT

PROC is file = (NODE node) BOOL:
CASE kind OF node IN
    (VOID directory): FALSE,
    (VOID file): TRUE
ESAC;

PROC is directory = (NODE node) BOOL:
CASE kind OF node IN
    (VOID directory): TRUE,
    (VOID file): FALSE
ESAC;

PROC get language = (NODE node) LANGUAGE:
IF NOT is file(node) THEN
    unknown
ELSE
    STRING ext = extract extension(path OF node);
    IF ext = "rs" THEN rust
    ELIF ext = "cs" THEN csharp
    ELIF ext = "py" THEN python
    ELIF ext = "js" THEN javascript
    ELIF ext = "ts" THEN typescript
    ELIF ext = "go" THEN go
    ELIF ext = "java" THEN java
    ELSE unknown
    FI
FI;

PROC has issues = (NODE node) BOOL:
(
    REF []ANALYSISRESULT results = analysis results OF node;
    REF GODFILERESULT god file = god file result OF node;
    
    BOOL has analysis issues = FALSE;
    
    IF results ISNT NIL THEN
        FOR i FROM LWB results TO UPB results DO
            IF is god class OF results[i] OR UPB god methods OF results[i] >= 1 THEN
                has analysis issues := TRUE
            FI
        OD
    FI;
    
    has analysis issues OR god file ISNT NIL
);

COMMENT Builder methods (immutable updates) COMMENT

PROC with children = (NODE node, []NODEID new children) NODE:
(
    NODE result = node;
    children OF result := new children;
    result
);

PROC with parent = (NODE node, NODEID new parent) NODE:
(
    NODE result = node;
    parent OF result := new parent;
    result
);

PROC with file metrics = (NODE node, FILEMETRICS metrics) NODE:
(
    NODE result = node;
    file metrics OF result := HEAP FILEMETRICS := metrics;
    result
);

PROC with analysis results = (NODE node, []ANALYSISRESULT results) NODE:
(
    NODE result = node;
    analysis results OF result := HEAP []ANALYSISRESULT := results;
    result
);

PROC with god file result = (NODE node, GODFILERESULT result) NODE:
(
    NODE result copy = node;
    god file result OF result copy := HEAP GODFILERESULT := result;
    result copy
);

COMMENT Utility procedures COMMENT

PROC extract filename = (STRING path) STRING:
BEGIN
    INT last slash = 0;
    FOR i FROM LWB path TO UPB path DO
        IF path[i] = "/" OR path[i] = "\" THEN
            last slash := i
        FI
    OD;
    
    IF last slash = 0 THEN
        path
    ELSE
        path[last slash + 1:]
    FI
END;

PROC extract extension = (STRING path) STRING:
BEGIN
    INT last dot = 0;
    FOR i FROM LWB path TO UPB path DO
        IF path[i] = "." THEN
            last dot := i
        FI
    OD;
    
    IF last dot = 0 OR last dot = UPB path THEN
        ""
    ELSE
        path[last dot + 1:]
    FI
END;

COMMENT Equality and comparison COMMENT

OP = = (NODEID a, NODEID b) BOOL:
    generation OF a = generation OF b AND index OF a = index OF b;

OP /= = (NODEID a, NODEID b) BOOL:
    NOT (a = b);

OP < = (NODEID a, NODEID b) BOOL:
    IF generation OF a = generation OF b THEN
        index OF a < index OF b
    ELSE
        generation OF a < generation OF b
    FI;


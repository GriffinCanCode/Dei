COMMENT
    Report Formatter
    
    Generates beautiful, formatted reports with:
    - Box-drawing characters for visual structure
    - Color-coded output (when terminal supports it)
    - Tree visualization of project structure
    - Statistical summaries
COMMENT

PR read "types/node.alg" PR;
PR read "types/models.alg" PR;
PR read "core/arena.alg" PR;

MODE REPORTCONFIG = STRUCT(
    BOOL use colors,
    BOOL show tree,
    BOOL verbose,
    STRING format  # "text", "json", "markdown"
);

PROC default report config = REPORTCONFIG:
    REPORTCONFIG(
        TRUE,   # Use colors
        TRUE,   # Show tree
        FALSE,  # Not verbose
        "text"
    );

COMMENT Main report generation COMMENT

PROC generate report = (REF ARENA arena, NODEID root, REPORTCONFIG config) STRING:
BEGIN
    COMMENT Collect statistics COMMENT
    SUMMARY stats = collect statistics(arena, root);
    
    COMMENT Generate appropriate format COMMENT
    STRING report = IF format OF config = "json" THEN
        generate json report(arena, root, stats)
    ELIF format OF config = "markdown" THEN
        generate markdown report(arena, root, stats)
    ELSE
        generate text report(arena, root, stats, config)
    FI;
    
    report
END;

COMMENT Text report generation COMMENT

PROC generate text report = (REF ARENA arena, NODEID root, SUMMARY stats, REPORTCONFIG config) STRING:
BEGIN
    STRING report := "";
    
    COMMENT Header COMMENT
    report +:= "╔════════════════════════════════════════════════════════════╗" + newline;
    report +:= "║           DEI CHECK - PROJECT ANALYSIS REPORT              ║" + newline;
    report +:= "╚════════════════════════════════════════════════════════════╝" + newline;
    report +:= newline;
    
    NODE root node = get node(arena, root);
    report +:= "Root Path: " + path OF root node + newline;
    report +:= "Analyzed: " + current timestamp string + newline;
    report +:= newline;
    
    COMMENT Summary statistics COMMENT
    report +:= summary to string(stats);
    report +:= newline;
    
    COMMENT God files section COMMENT
    IF god files OF stats > 0 THEN
        report +:= "⚠️  GOD FILES DETECTED:" + newline + newline;
        report +:= generate god files section(arena, root)
    FI;
    
    COMMENT God classes section COMMENT
    IF god classes OF stats > 0 THEN
        report +:= "⚠️  GOD CLASSES DETECTED:" + newline + newline;
        report +:= generate god classes section(arena, root)
    FI;
    
    COMMENT God methods section COMMENT
    IF god methods OF stats > 0 THEN
        report +:= "⚠️  GOD METHODS DETECTED:" + newline + newline;
        report +:= generate god methods section(arena, root)
    FI;
    
    COMMENT Tree view of problems COMMENT
    IF show tree OF config AND has issues(root node) THEN
        report +:= newline;
        report +:= "Problem Files in Project Structure:" + newline + newline;
        report +:= "PROJECT STRUCTURE:" + newline + newline;
        report +:= generate problem tree(arena, root, 0)
    FI;
    
    report
END;

COMMENT Collect statistics from analyzed AST COMMENT

PROC collect statistics = (REF ARENA arena, NODEID root) SUMMARY:
BEGIN
    SUMMARY stats := empty summary;
    collect stats recursive(arena, root, stats);
    stats
END;

PROC collect stats recursive = (REF ARENA arena, NODEID node id, REF SUMMARY stats) VOID:
BEGIN
    NODE node = get node(arena, node id);
    
    IF is file(node) THEN
        total files OF stats +:= 1;
        
        COMMENT Count classes COMMENT
        REF FILEMETRICS file metrics = file metrics OF node;
        IF file metrics ISNT NIL THEN
            total classes OF stats +:= UPB classes OF file metrics
        FI;
        
        COMMENT Check god file COMMENT
        IF god file result OF node ISNT NIL THEN
            god files OF stats +:= 1
        FI;
        
        COMMENT Check analysis results COMMENT
        REF []ANALYSISRESULT results = analysis results OF node;
        IF results ISNT NIL THEN
            FOR i FROM LWB results TO UPB results DO
                IF is god class OF results[i] THEN
                    god classes OF stats +:= 1
                FI;
                
                INT method count = UPB god methods OF results[i];
                IF method count > 0 THEN
                    god methods OF stats +:= method count;
                    classes with god methods OF stats +:= 1
                ELIF NOT is god class OF results[i] THEN
                    healthy classes OF stats +:= 1
                FI
            OD
        FI
    ELIF is directory(node) THEN
        COMMENT Recurse into children COMMENT
        []NODEID children ids = children OF node;
        FOR i FROM LWB children ids TO UPB children ids DO
            collect stats recursive(arena, children ids[i], stats)
        OD
    FI
END;

COMMENT Generate god files section COMMENT

PROC generate god files section = (REF ARENA arena, NODEID root) STRING:
BEGIN
    STRING output := "";
    collect god files(arena, root, output);
    output
END;

PROC collect god files = (REF ARENA arena, NODEID node id, REF STRING output) VOID:
BEGIN
    NODE node = get node(arena, node id);
    
    IF is file(node) THEN
        REF GODFILERESULT god file = god file result OF node;
        IF god file ISNT NIL THEN
            output +:= "  📄 " + name OF node + newline;
            output +:= god file to string(god file);
            output +:= newline
        FI
    ELIF is directory(node) THEN
        []NODEID children ids = children OF node;
        FOR i FROM LWB children ids TO UPB children ids DO
            collect god files(arena, children ids[i], output)
        OD
    FI
END;

COMMENT Generate god classes section COMMENT

PROC generate god classes section = (REF ARENA arena, NODEID root) STRING:
BEGIN
    STRING output := "";
    collect god classes(arena, root, output);
    output
END;

PROC collect god classes = (REF ARENA arena, NODEID node id, REF STRING output) VOID:
BEGIN
    NODE node = get node(arena, node id);
    
    IF is file(node) THEN
        REF []ANALYSISRESULT results = analysis results OF node;
        IF results ISNT NIL THEN
            FOR i FROM LWB results TO UPB results DO
                IF is god class OF results[i] THEN
                    output +:= result to string(results[i]);
                    output +:= newline
                FI
            OD
        FI
    ELIF is directory(node) THEN
        []NODEID children ids = children OF node;
        FOR i FROM LWB children ids TO UPB children ids DO
            collect god classes(arena, children ids[i], output)
        OD
    FI
END;

COMMENT Generate god methods section COMMENT

PROC generate god methods section = (REF ARENA arena, NODEID root) STRING:
BEGIN
    STRING output := "";
    collect god methods(arena, root, output);
    output
END;

PROC collect god methods = (REF ARENA arena, NODEID node id, REF STRING output) VOID:
BEGIN
    NODE node = get node(arena, node id);
    
    IF is file(node) THEN
        REF []ANALYSISRESULT results = analysis results OF node;
        IF results ISNT NIL THEN
            FOR i FROM LWB results TO UPB results DO
                []GODMETHODRESULT god meths = god methods OF results[i];
                IF UPB god meths > 0 THEN
                    CLASSMETRICS class metrics = class metrics OF results[i];
                    output +:= "  📝 " + name OF class metrics + newline;
                    output +:= "     File: " + file path OF class metrics + newline;
                    output +:= "     God Methods: " + whole(UPB god meths, 0) + newline;
                    output +:= newline;
                    
                    FOR j FROM LWB god meths TO UPB god meths DO
                        output +:= god method to string(god meths[j])
                    OD;
                    output +:= newline
                FI
            OD
        FI
    ELIF is directory(node) THEN
        []NODEID children ids = children OF node;
        FOR i FROM LWB children ids TO UPB children ids DO
            collect god methods(arena, children ids[i], output)
        OD
    FI
END;

COMMENT Generate tree view showing only problem files COMMENT

PROC generate problem tree = (REF ARENA arena, NODEID node id, INT indent) STRING:
BEGIN
    NODE node = get node(arena, node id);
    STRING output := "";
    
    COMMENT Only show nodes that have issues or contain nodes with issues COMMENT
    IF has issues(node) OR has problem descendants(arena, node id) THEN
        COMMENT Indentation COMMENT
        FOR i FROM 1 TO indent DO
            output +:= "    "
        OD;
        
        COMMENT Tree connectors COMMENT
        IF indent > 0 THEN
            output +:= "├── "
        ELSE
            output +:= "└── "
        FI;
        
        COMMENT Node name with issue indicator COMMENT
        IF is directory(node) THEN
            output +:= name OF node + "/" + newline
        ELSE
            STRING indicator = "";
            REF GODFILERESULT god file = god file result OF node;
            REF []ANALYSISRESULT results = analysis results OF node;
            
            IF god file ISNT NIL THEN
                indicator := " 📄 [GOD FILE]"
            FI;
            
            IF results ISNT NIL THEN
                FOR i FROM LWB results TO UPB results DO
                    IF is god class OF results[i] THEN
                        indicator := " ❌ [GOD CLASS]"
                    ELIF UPB god methods OF results[i] > 0 THEN
                        indicator := " ⚠️ [" + whole(UPB god methods OF results[i], 0) + 
                                     " GOD METHOD(S)]"
                    FI
                OD
            FI;
            
            output +:= name OF node + indicator + newline
        FI;
        
        COMMENT Recurse into children COMMENT
        []NODEID children ids = children OF node;
        FOR i FROM LWB children ids TO UPB children ids DO
            output +:= generate problem tree(arena, children ids[i], indent + 1)
        OD
    FI;
    
    output
END;

PROC has problem descendants = (REF ARENA arena, NODEID node id) BOOL:
BEGIN
    NODE node = get node(arena, node id);
    []NODEID children ids = children OF node;
    BOOL has problems := FALSE;
    
    FOR i FROM LWB children ids TO UPB children ids DO
        NODE child = get node(arena, children ids[i]);
        IF has issues(child) OR has problem descendants(arena, children ids[i]) THEN
            has problems := TRUE
        FI
    OD;
    
    has problems
END;

COMMENT JSON output COMMENT

PROC generate json report = (REF ARENA arena, NODEID root, SUMMARY stats) STRING:
BEGIN
    STRING json := "{" + newline;
    json +:= "  " + quote + "summary" + quote + ": {" + newline;
    json +:= "    " + quote + "totalFiles" + quote + ": " + whole(total files OF stats, 0) + "," + newline;
    json +:= "    " + quote + "totalClasses" + quote + ": " + whole(total classes OF stats, 0) + "," + newline;
    json +:= "    " + quote + "godFiles" + quote + ": " + whole(god files OF stats, 0) + "," + newline;
    json +:= "    " + quote + "godClasses" + quote + ": " + whole(god classes OF stats, 0) + "," + newline;
    json +:= "    " + quote + "godMethods" + quote + ": " + whole(god methods OF stats, 0) + newline;
    json +:= "  }" + newline;
    json +:= "}" + newline;
    json
END;

COMMENT Markdown output COMMENT

PROC generate markdown report = (REF ARENA arena, NODEID root, SUMMARY stats) STRING:
BEGIN
    STRING md := "# DEI Analysis Report" + newline + newline;
    md +:= "## Summary" + newline + newline;
    md +:= "| Metric | Count |" + newline;
    md +:= "|--------|-------|" + newline;
    md +:= "| Total Files | " + whole(total files OF stats, 0) + " |" + newline;
    md +:= "| Total Classes | " + whole(total classes OF stats, 0) + " |" + newline;
    md +:= "| God Files | " + whole(god files OF stats, 0) + " |" + newline;
    md +:= "| God Classes | " + whole(god classes OF stats, 0) + " |" + newline;
    md +:= "| God Methods | " + whole(god methods OF stats, 0) + " |" + newline;
    md +:= newline;
    md
END;

COMMENT Utilities COMMENT

STRING newline = REPR 10;
STRING quote = REPR 34;

PROC current timestamp string = STRING:
    "2025-11-19 (ALGOL-68)";  # Placeholder

PROC fixed = (REAL x, INT width, INT precision) STRING:
    COMMENT Would format real number with precision COMMENT
    whole(entier x, width);


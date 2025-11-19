COMMENT
    Analysis Result Models
    
    Defines structures for storing and reporting analysis results.
COMMENT

COMMENT Violation tracking COMMENT

MODE VIOLATIONKIND = UNION(
    VOID lines violation,
    VOID complexity violation,
    VOID method count violation,
    VOID parameter count violation,
    VOID classes per file violation
);

MODE VIOLATION = STRUCT(
    VIOLATIONKIND kind,
    INT actual,
    INT threshold
);

PROC violation to string = (VIOLATION v) STRING:
BEGIN
    STRING kind name = CASE kind OF v IN
        (VOID lines violation): "Lines",
        (VOID complexity violation): "Complexity",
        (VOID method count violation): "Method count",
        (VOID parameter count violation): "Parameter count",
        (VOID classes per file violation): "Classes per file"
    ESAC;
    
    kind name + " (" + whole(actual OF v, 0) + ") exceeds threshold (" + 
    whole(threshold OF v, 0) + ")"
END;

COMMENT Responsibility clustering COMMENT

MODE RESPONSIBILITYCLUSTER = STRUCT(
    STRING suggested name,
    []STRING method names,
    REAL cohesion score,
    []STRING shared dependencies,
    STRING justification
);

PROC cluster to string = (RESPONSIBILITYCLUSTER cluster) STRING:
BEGIN
    STRING result := "Extract class: " + suggested name OF cluster + newline;
    result +:= "  Cohesion: " + fixed(cohesion score OF cluster, 3, 2) + newline;
    result +:= "  Methods: ";
    
    []STRING methods = method names OF cluster;
    FOR i FROM LWB methods TO UPB methods DO
        result +:= methods[i];
        IF i < UPB methods THEN result +:= ", " FI
    OD;
    
    result +:= newline;
    result +:= "  Reason: " + justification OF cluster;
    result
END;

COMMENT God method result COMMENT

MODE GODMETHODRESULT = STRUCT(
    STRING method name,
    STRING class name,
    STRING file path,
    METHODMETRICS metrics,
    []VIOLATION violations,
    REAL violation score
);

PROC god method to string = (GODMETHODRESULT result) STRING:
BEGIN
    STRING output := "  Method: " + method name OF result + newline;
    output +:= "    Lines: " + lines to string(lines OF metrics OF result);
    output +:= ", Complexity: " + complexity to string(complexity OF metrics OF result);
    output +:= ", Parameters: " + param count to string(parameters OF metrics OF result);
    output +:= newline;
    
    []VIOLATION viols = violations OF result;
    IF UPB viols > 0 THEN
        output +:= "    Violations:" + newline;
        FOR i FROM LWB viols TO UPB viols DO
            output +:= "      • " + violation to string(viols[i]) + newline
        OD
    FI;
    
    output
END;

COMMENT God file result COMMENT

MODE GODFILERESULT = STRUCT(
    STRING file path,
    INT class count,
    INT total lines,
    []STRING class names,
    []VIOLATION violations
);

PROC god file to string = (GODFILERESULT result) STRING:
BEGIN
    STRING output := "File: " + file path OF result + newline;
    output +:= "  Classes: " + whole(class count OF result, 0);
    output +:= ", Total Lines: " + whole(total lines OF result, 0) + newline;
    
    []STRING names = class names OF result;
    IF UPB names > 0 THEN
        output +:= "  Classes in file: ";
        FOR i FROM LWB names TO UPB names DO
            output +:= names[i];
            IF i < UPB names THEN output +:= ", " FI
        OD;
        output +:= newline
    FI;
    
    []VIOLATION viols = violations OF result;
    FOR i FROM LWB viols TO UPB viols DO
        output +:= "  • " + violation to string(viols[i]) + newline
    OD;
    
    output
END;

COMMENT Complete analysis result COMMENT

MODE ANALYSISRESULT = STRUCT(
    CLASSMETRICS class metrics,
    BOOL is god class,
    []RESPONSIBILITYCLUSTER suggested extractions,
    []GODMETHODRESULT god methods,
    STRING summary,
    INT analyzed at  # Unix timestamp
);

PROC healthy result = (CLASSMETRICS metrics) ANALYSISRESULT:
    ANALYSISRESULT(
        metrics,
        FALSE,  # Not a god class
        (),     # No suggested extractions
        (),     # No god methods
        "Class '" + name OF metrics + "' is within acceptable thresholds",
        seconds  # Current timestamp
    );

PROC has issues = (ANALYSISRESULT result) BOOL:
    is god class OF result OR UPB god methods OF result > 0;

PROC result to string = (ANALYSISRESULT result) STRING:
BEGIN
    CLASSMETRICS metrics = class metrics OF result;
    STRING output := "";
    
    IF is god class OF result THEN
        output +:= "❌ GOD CLASS: " + name OF metrics + newline;
        output +:= "  File: " + file path OF metrics + newline;
        output +:= "  Lines: " + lines to string(lines OF metrics);
        output +:= ", Methods: " + method count to string(method count OF metrics);
        output +:= ", Complexity: " + complexity to string(complexity OF metrics);
        output +:= newline;
        
        []RESPONSIBILITYCLUSTER clusters = suggested extractions OF result;
        IF UPB clusters > 0 THEN
            output +:= "  Suggested Extractions: " + whole(UPB clusters, 0) + newline;
            FOR i FROM LWB clusters TO UPB clusters DO
                output +:= "  " + cluster to string(clusters[i]) + newline
            OD
        FI
    FI;
    
    []GODMETHODRESULT god meths = god methods OF result;
    IF UPB god meths > 0 THEN
        IF is god class OF result THEN
            output +:= "  God Methods: " + whole(UPB god meths, 0) + newline
        ELSE
            output +:= "⚠️  CLASS WITH GOD METHODS: " + name OF metrics + newline;
            output +:= "  File: " + file path OF metrics + newline
        FI;
        
        FOR i FROM LWB god meths TO UPB god meths DO
            output +:= god method to string(god meths[i])
        OD
    FI;
    
    output
END;

COMMENT Summary statistics COMMENT

MODE SUMMARY = STRUCT(
    INT total files,
    INT total classes,
    INT god files,
    INT god classes,
    INT god methods,
    INT classes with god methods,
    INT healthy classes
);

PROC empty summary = SUMMARY:
    SUMMARY(0, 0, 0, 0, 0, 0, 0);

PROC summary to string = (SUMMARY stats) STRING:
BEGIN
    STRING output := "";
    output +:= "╭────────────────────────────┬───────╮" + newline;
    output +:= "│ Total Files Analyzed       │ " + whole(total files OF stats, 5) + " │" + newline;
    output +:= "│ Total Classes              │ " + whole(total classes OF stats, 5) + " │" + newline;
    output +:= "│ God Files Detected         │ " + whole(god files OF stats, 5) + " │" + newline;
    output +:= "│ God Classes Detected       │ " + whole(god classes OF stats, 5) + " │" + newline;
    output +:= "│ God Methods Detected       │ " + whole(god methods OF stats, 5) + " │" + newline;
    output +:= "│ Classes with God Methods   │ " + whole(classes with god methods OF stats, 5) + " │" + newline;
    output +:= "│ Healthy Classes            │ " + whole(healthy classes OF stats, 5) + " │" + newline;
    output +:= "╰────────────────────────────┴───────╯" + newline;
    output
END;


COMMENT
    Metrics Type Definitions
    
    Strong typing for code metrics prevents accidental comparisons
    and provides semantic meaning to measurements.
COMMENT

COMMENT Newtype wrappers for type safety COMMENT

MODE LINES = STRUCT(INT value);
MODE COMPLEXITY = STRUCT(INT value);
MODE METHODCOUNT = STRUCT(INT value);
MODE PARAMCOUNT = STRUCT(INT value);

COMMENT Operator overloading for natural comparisons COMMENT

OP > = (LINES a, LINES b) BOOL: value OF a > value OF b;
OP < = (LINES a, LINES b) BOOL: value OF a < value OF b;
OP >= = (LINES a, LINES b) BOOL: value OF a >= value OF b;
OP <= = (LINES a, LINES b) BOOL: value OF a <= value OF b;
OP = = (LINES a, LINES b) BOOL: value OF a = value OF b;

OP > = (COMPLEXITY a, COMPLEXITY b) BOOL: value OF a > value OF b;
OP < = (COMPLEXITY a, COMPLEXITY b) BOOL: value OF a < value OF b;
OP >= = (COMPLEXITY a, COMPLEXITY b) BOOL: value OF a >= value OF b;
OP <= = (COMPLEXITY a, COMPLEXITY b) BOOL: value OF a <= value OF b;
OP = = (COMPLEXITY a, COMPLEXITY b) BOOL: value OF a = value OF b;

OP > = (METHODCOUNT a, METHODCOUNT b) BOOL: value OF a > value OF b;
OP < = (METHODCOUNT a, METHODCOUNT b) BOOL: value OF a < value OF b;
OP >= = (METHODCOUNT a, METHODCOUNT b) BOOL: value OF a >= value OF b;
OP <= = (METHODCOUNT a, METHODCOUNT b) BOOL: value OF a <= value OF b;
OP = = (METHODCOUNT a, METHODCOUNT b) BOOL: value OF a = value OF b;

OP > = (PARAMCOUNT a, PARAMCOUNT b) BOOL: value OF a > value OF b;
OP < = (PARAMCOUNT a, PARAMCOUNT b) BOOL: value OF a < value OF b;
OP >= = (PARAMCOUNT a, PARAMCOUNT b) BOOL: value OF a >= value OF b;
OP <= = (PARAMCOUNT a, PARAMCOUNT b) BOOL: value OF a <= value OF b;
OP = = (PARAMCOUNT a, PARAMCOUNT b) BOOL: value OF a = value OF b;

COMMENT Arithmetic operators COMMENT

OP + = (LINES a, LINES b) LINES: LINES(value OF a + value OF b);
OP + = (COMPLEXITY a, COMPLEXITY b) COMPLEXITY: COMPLEXITY(value OF a + value OF b);
OP + = (METHODCOUNT a, METHODCOUNT b) METHODCOUNT: METHODCOUNT(value OF a + value OF b);

COMMENT Constructors COMMENT

PROC lines = (INT n) LINES: LINES(n);
PROC complexity = (INT n) COMPLEXITY: COMPLEXITY(n);
PROC method count = (INT n) METHODCOUNT: METHODCOUNT(n);
PROC param count = (INT n) PARAMCOUNT: PARAMCOUNT(n);

COMMENT Method Metrics COMMENT

MODE METHODMETRICS = STRUCT(
    STRING name,
    LINES lines,
    COMPLEXITY complexity,
    PARAMCOUNT parameters,
    []STRING called methods,
    []STRING accessed fields,
    STRING return type,
    BOOL is public,
    BOOL is static,
    BOOL is async,
    []STRING tokens  # For semantic analysis
);

MODE THRESHOLDS = STRUCT(
    COMMENT Class thresholds COMMENT
    LINES max class lines,
    METHODCOUNT max methods,
    COMPLEXITY max class complexity,
    
    COMMENT Method thresholds COMMENT
    LINES max method lines,
    COMPLEXITY max method complexity,
    PARAMCOUNT max parameters,
    
    COMMENT File thresholds COMMENT
    INT max classes per file,
    LINES max file lines,
    
    COMMENT Clustering parameters COMMENT
    INT min cluster size,
    REAL cluster threshold
);

PROC is god method = (METHODMETRICS method, THRESHOLDS thresholds) BOOL:
(
    lines OF method > max method lines OF thresholds OR
    complexity OF method > max method complexity OF thresholds OR
    parameters OF method > max parameters OF thresholds
);

PROC violation score = (METHODMETRICS method, THRESHOLDS thresholds) REAL:
BEGIN
    REAL line ratio = value OF lines OF method / value OF max method lines OF thresholds;
    REAL complexity ratio = value OF complexity OF method / value OF max method complexity OF thresholds;
    REAL param ratio = value OF parameters OF method / value OF max parameters OF thresholds;
    
    (line ratio + complexity ratio + param ratio) / 3.0
END;

COMMENT Class Metrics COMMENT

MODE CLASSMETRICS = STRUCT(
    STRING name,
    STRING fully qualified name,
    STRING file path,
    LINES lines,
    METHODCOUNT method count,
    INT property count,
    INT field count,
    COMPLEXITY complexity,
    []METHODMETRICS methods,
    []STRING dependencies
);

PROC is god class = (CLASSMETRICS class, THRESHOLDS thresholds) BOOL:
(
    lines OF class > max class lines OF thresholds OR
    method count OF class > max methods OF thresholds OR
    complexity OF class > max class complexity OF thresholds
);

PROC count god methods = (CLASSMETRICS class, THRESHOLDS thresholds) INT:
BEGIN
    INT count := 0;
    []METHODMETRICS methods = methods OF class;
    
    FOR i FROM LWB methods TO UPB methods DO
        IF is god method(methods[i], thresholds) THEN
            count +:= 1
        FI
    OD;
    
    count
END;

COMMENT File Metrics COMMENT

MODE FILEMETRICS = STRUCT(
    STRING path,
    LINES lines,
    []CLASSMETRICS classes
);

PROC is god file = (FILEMETRICS file, THRESHOLDS thresholds) BOOL:
(
    UPB classes OF file > max classes per file OF thresholds OR
    lines OF file > max file lines OF thresholds
);

COMMENT Default thresholds COMMENT

PROC default thresholds = THRESHOLDS:
    THRESHOLDS(
        COMMENT Class thresholds COMMENT
        lines(300),
        method count(20),
        complexity(50),
        
        COMMENT Method thresholds COMMENT
        lines(50),
        complexity(10),
        param count(5),
        
        COMMENT File thresholds COMMENT
        3,
        lines(500),
        
        COMMENT Clustering COMMENT
        3,
        0.7
    );

COMMENT Conversion to strings for display COMMENT

PROC lines to string = (LINES l) STRING:
    whole(value OF l, 0);

PROC complexity to string = (COMPLEXITY c) STRING:
    whole(value OF c, 0);

PROC method count to string = (METHODCOUNT m) STRING:
    whole(value OF m, 0);

PROC param count to string = (PARAMCOUNT p) STRING:
    whole(value OF p, 0);


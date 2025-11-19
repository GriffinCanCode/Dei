COMMENT
    Sophisticated Complexity Calculator
    
    Calculates cyclomatic complexity with weighted decision points.
    Goes beyond traditional McCabe complexity by considering cognitive load.
    
    Key innovations:
    - Weighted control structures (loops cost more than simple ifs)
    - Nesting depth penalties
    - Boolean expression complexity
    - Pattern-based analysis (language-agnostic)
COMMENT

PR read "types/metrics.alg" PR;

MODE COMPLEXITYCONFIG = STRUCT(
    INT if weight,
    INT loop weight,
    INT case weight,
    INT boolean weight,
    INT nesting penalty,
    BOOL count nesting
);

PROC default complexity config = COMPLEXITYCONFIG:
    COMPLEXITYCONFIG(
        1,  # IF statements
        2,  # Loops (higher cognitive load)
        2,  # CASE/switch statements
        1,  # Boolean operators (AND, OR)
        1,  # Nesting depth multiplier
        TRUE
    );

COMMENT Main complexity calculation COMMENT

PROC calculate cyclomatic complexity = (STRING source, COMPLEXITYCONFIG config) COMPLEXITY:
BEGIN
    INT base complexity := 1;
    
    COMMENT Count decision points COMMENT
    INT if count = count pattern(source, "IF") + count pattern(source, "if");
    INT while count = count pattern(source, "WHILE") + count pattern(source, "while");
    INT for count = count pattern(source, "FOR") + count pattern(source, "for");
    INT case count = count pattern(source, "CASE") + count pattern(source, "case") + 
                      count pattern(source, "switch");
    INT and count = count pattern(source, "AND") + count pattern(source, "and") +
                     count pattern(source, "&&");
    INT or count = count pattern(source, "OR") + count pattern(source, "or") +
                    count pattern(source, "||");
    
    COMMENT Apply weights COMMENT
    INT total complexity := base complexity;
    total complexity +:= if count × if weight OF config;
    total complexity +:= (while count + for count) × loop weight OF config;
    total complexity +:= case count × case weight OF config;
    total complexity +:= (and count + or count) × boolean weight OF config;
    
    COMMENT Add nesting penalty if configured COMMENT
    IF count nesting OF config THEN
        INT max depth = calculate max nesting depth(source);
        total complexity +:= max depth × nesting penalty OF config
    FI;
    
    complexity(total complexity)
END;

COMMENT Calculate maximum nesting depth COMMENT

PROC calculate max nesting depth = (STRING source) INT:
BEGIN
    INT current depth := 0;
    INT max depth := 0;
    
    COMMENT Simple brace/block counting COMMENT
    FOR i FROM LWB source TO UPB source DO
        CHAR c = source[i];
        
        IF c = "{" OR c = "(" OR c = "[" THEN
            current depth +:= 1;
            IF current depth > max depth THEN
                max depth := current depth
            FI
        ELIF c = "}" OR c = ")" OR c = "]" THEN
            current depth -:= 1
        FI
    OD;
    
    max depth
END;

COMMENT Count lines of code (non-comment, non-blank) COMMENT

PROC count lines of code = (STRING source) LINES:
BEGIN
    INT line count := 0;
    INT current pos := LWB source;
    BOOL in line := FALSE;
    BOOL in comment := FALSE;
    BOOL has content := FALSE;
    
    WHILE current pos <= UPB source DO
        CHAR c = source[current pos];
        CHAR next = IF current pos < UPB source THEN source[current pos + 1] ELSE " " FI;
        
        COMMENT Check for comments COMMENT
        IF c = "/" AND next = "/" THEN
            in comment := TRUE
        ELIF c = "#" THEN
            in comment := TRUE
        ELIF c = COMMENT Character COMMENT THEN
            in comment := NOT in comment
        FI;
        
        COMMENT Check for newline COMMENT
        IF c = newline character THEN
            IF has content AND NOT in comment THEN
                line count +:= 1
            FI;
            in comment := FALSE;
            has content := FALSE
        ELIF c /= " " AND c /= tab character AND NOT in comment THEN
            has content := TRUE
        FI;
        
        current pos +:= 1
    OD;
    
    COMMENT Count final line if it has content COMMENT
    IF has content AND NOT in comment THEN
        line count +:= 1
    FI;
    
    lines(line count)
END;

COMMENT Pattern counting utilities COMMENT

PROC count pattern = (STRING source, STRING pattern) INT:
BEGIN
    INT count := 0;
    INT source len = UPB source - LWB source + 1;
    INT pattern len = UPB pattern - LWB pattern + 1;
    
    IF pattern len > source len THEN
        0
    ELSE
        FOR i FROM LWB source TO UPB source - pattern len + 1 DO
            IF matches at(source, pattern, i) THEN
                count +:= 1
            FI
        OD;
        count
    FI
END;

PROC matches at = (STRING source, STRING pattern, INT pos) BOOL:
BEGIN
    BOOL matches := TRUE;
    INT pattern len = UPB pattern - LWB pattern + 1;
    
    FOR i FROM 0 TO pattern len - 1 DO
        IF source[pos + i] /= pattern[LWB pattern + i] THEN
            matches := FALSE
        FI
    OD;
    
    matches
END;

COMMENT Advanced metrics COMMENT

COMMENT Halstead complexity measures COMMENT

MODE HALSTEADMETRICS = STRUCT(
    INT n1,  # Unique operators
    INT n2,  # Unique operands
    INT cap n1,  # Total operators
    INT cap n2,  # Total operands
    REAL volume,
    REAL difficulty,
    REAL effort
);

PROC calculate halstead = (STRING source) HALSTEADMETRICS:
BEGIN
    COMMENT Simplified Halstead - count operators and operands COMMENT
    
    INT unique ops = count unique operators(source);
    INT unique operands = count unique operands(source);
    INT total ops = count total operators(source);
    INT total operands = count total operands(source);
    
    INT vocabulary = unique ops + unique operands;
    INT length = total ops + total operands;
    
    REAL volume = length × log2(vocabulary);
    REAL difficulty = (unique ops / 2.0) × (total operands / REAL(unique operands));
    REAL effort = volume × difficulty;
    
    HALSTEADMETRICS(
        unique ops,
        unique operands,
        total ops,
        total operands,
        volume,
        difficulty,
        effort
    )
END;

COMMENT Maintainability index COMMENT

PROC calculate maintainability index = (STRING source, COMPLEXITY complexity) REAL:
BEGIN
    LINES loc = count lines of code(source);
    HALSTEADMETRICS halstead = calculate halstead(source);
    
    COMMENT Microsoft's maintainability index formula COMMENT
    REAL mi = 171.0 - 5.2 × ln(volume OF halstead) - 
               0.23 × REAL(value OF complexity) -
               16.2 × ln(REAL(value OF loc));
    
    COMMENT Normalize to 0-100 scale COMMENT
    IF mi < 0 THEN 0
    ELIF mi > 100 THEN 100
    ELSE mi
    FI
END;

COMMENT Placeholder implementations for Halstead COMMENT

PROC count unique operators = (STRING source) INT:
    COMMENT Would tokenize and count unique operators COMMENT
    10;  # Placeholder

PROC count unique operands = (STRING source) INT:
    COMMENT Would tokenize and count unique identifiers/literals COMMENT
    20;  # Placeholder

PROC count total operators = (STRING source) INT:
    COMMENT Would tokenize and count all operators COMMENT
    50;  # Placeholder

PROC count total operands = (STRING source) INT:
    COMMENT Would tokenize and count all operands COMMENT
    100;  # Placeholder

COMMENT Math utilities COMMENT

PROC log2 = (INT n) REAL:
    ln(REAL(n)) / ln(2.0);

PROC ln = (REAL x) REAL:
    COMMENT Natural logarithm - would use math library COMMENT
    1.0;  # Placeholder

COMMENT Character constants COMMENT

CHAR newline character = REPR 10;
CHAR tab character = REPR 9;

COMMENT Cognitive complexity (alternative to cyclomatic) COMMENT

COMMENT
    Cognitive complexity better represents actual code understanding difficulty.
    It considers:
    - Nesting depth (heavily weighted)
    - Breaks in linear flow
    - Structural complexity
COMMENT

PROC calculate cognitive complexity = (STRING source) INT:
BEGIN
    INT cognitive := 0;
    INT nesting level := 0;
    
    COMMENT This would require proper parsing COMMENT
    COMMENT Simplified version counts control flow with nesting weight COMMENT
    
    INT control structures = 
        count pattern(source, "IF") + count pattern(source, "if") +
        count pattern(source, "WHILE") + count pattern(source, "while") +
        count pattern(source, "FOR") + count pattern(source, "for") +
        count pattern(source, "CASE") + count pattern(source, "case");
    
    INT max nesting = calculate max nesting depth(source);
    
    COMMENT Each control structure adds (1 + nesting level) COMMENT
    cognitive := control structures × (1 + max nesting ÷ 2);
    
    cognitive
END;


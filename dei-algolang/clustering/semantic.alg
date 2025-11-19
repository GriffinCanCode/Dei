COMMENT
    Semantic Clustering for Responsibility Detection
    
    Uses TF-IDF-like feature extraction and K-means to group methods
    by shared responsibility. Generates meaningful cluster names
    from method tokens.
COMMENT

PR read "types/metrics.alg" PR;
PR read "types/models.alg" PR;
PR read "clustering/kmeans.alg" PR;

MODE FEATUREVECTOR = STRUCT(
    []REAL features,
    METHODMETRICS method
);

COMMENT Main semantic analysis COMMENT

PROC perform semantic clustering = (CLASSMETRICS class, THRESHOLDS thresholds) []RESPONSIBILITYCLUSTER:
BEGIN
    []METHODMETRICS methods = methods OF class;
    INT num methods = UPB methods - LWB methods + 1;
    
    COMMENT Need at least min cluster size methods COMMENT
    IF num methods < min cluster size OF thresholds THEN
        ()
    ELSE
        COMMENT Build feature vectors COMMENT
        []FEATUREVECTOR features = build feature vectors(methods);
        
        COMMENT Determine optimal K COMMENT
        INT max k = entier sqrt(REAL(num methods));
        IF max k > 5 THEN max k := 5 FI;
        IF max k < 2 THEN max k := 2 FI;
        
        COMMENT Extract just the feature arrays for clustering COMMENT
        [1:num methods, 1:UPB features OF features[1]]REAL feature matrix;
        FOR i FROM 1 TO num methods DO
            []REAL method features = features OF features[i];
            FOR j FROM 1 TO UPB method features DO
                feature matrix[i, j] := method features[j]
            OD
        OD;
        
        COMMENT Perform K-means COMMENT
        INT optimal k = determine optimal k(feature matrix, max k, default cluster config);
        CLUSTERRESULT result = kmeans(feature matrix, optimal k, default cluster config);
        
        COMMENT Group methods by cluster COMMENT
        []INT labels = labels OF result;
        [][]METHODMETRICS method groups = group methods by cluster(methods, labels, optimal k);
        
        COMMENT Create responsibility clusters COMMENT
        [1:optimal k]RESPONSIBILITYCLUSTER clusters;
        INT cluster count := 0;
        
        FOR i FROM 1 TO optimal k DO
            []METHODMETRICS group = method groups[i];
            IF UPB group >= min cluster size OF thresholds THEN
                cluster count +:= 1;
                clusters[cluster count] := create responsibility cluster(
                    group,
                    name OF class
                )
            FI
        OD;
        
        COMMENT Return only non-empty clusters COMMENT
        [1:cluster count]RESPONSIBILITYCLUSTER final clusters;
        FOR i FROM 1 TO cluster count DO
            final clusters[i] := clusters[i]
        OD;
        
        final clusters
    FI
END;

COMMENT Build TF-IDF-like feature vectors COMMENT

PROC build feature vectors = ([]METHODMETRICS methods) []FEATUREVECTOR:
BEGIN
    INT num methods = UPB methods - LWB methods + 1;
    
    COMMENT Build vocabulary from all tokens COMMENT
    []STRING vocabulary = build vocabulary(methods);
    INT vocab size = UPB vocabulary - LWB vocabulary + 1;
    
    COMMENT Create feature vectors COMMENT
    [1:num methods]FEATUREVECTOR vectors;
    
    FOR i FROM 1 TO num methods DO
        METHODMETRICS method = methods[i];
        
        COMMENT Token features (TF-IDF) COMMENT
        [1:vocab size + 6]REAL features;  # vocab + 6 structural features
        
        COMMENT Calculate token frequencies COMMENT
        []STRING method tokens = tokens OF method;
        FOR j FROM 1 TO vocab size DO
            INT count = count token(method tokens, vocabulary[j]);
            REAL tf = REAL(count) / REAL(UPB method tokens);
            REAL idf = calculate idf(methods, vocabulary[j]);
            features[j] := tf × idf
        OD;
        
        COMMENT Add structural features (normalized) COMMENT
        features[vocab size + 1] := REAL(value OF lines OF method) / 100.0;
        features[vocab size + 2] := REAL(value OF complexity OF method) / 20.0;
        features[vocab size + 3] := REAL(UPB called methods OF method) / 10.0;
        features[vocab size + 4] := REAL(UPB accessed fields OF method) / 10.0;
        features[vocab size + 5] := IF is public OF method THEN 1.0 ELSE 0.0 FI;
        features[vocab size + 6] := IF is static OF method THEN 1.0 ELSE 0.0 FI;
        
        vectors[i] := FEATUREVECTOR(features, method)
    OD;
    
    vectors
END;

COMMENT Build vocabulary from all method tokens COMMENT

PROC build vocabulary = ([]METHODMETRICS methods) []STRING:
BEGIN
    COMMENT Collect all unique tokens COMMENT
    [1:1000]STRING vocab;
    INT vocab size := 0;
    
    FOR i FROM LWB methods TO UPB methods DO
        []STRING method tokens = tokens OF methods[i];
        FOR j FROM LWB method tokens TO UPB method tokens DO
            STRING token = method tokens[j];
            IF NOT contains(vocab, vocab size, token) THEN
                vocab size +:= 1;
                vocab[vocab size] := token
            FI
        OD
    OD;
    
    [1:vocab size]STRING final vocab;
    FOR i FROM 1 TO vocab size DO
        final vocab[i] := vocab[i]
    OD;
    
    final vocab
END;

COMMENT Calculate inverse document frequency COMMENT

PROC calculate idf = ([]METHODMETRICS methods, STRING token) REAL:
BEGIN
    INT num methods = UPB methods - LWB methods + 1;
    INT doc count := 0;
    
    FOR i FROM LWB methods TO UPB methods DO
        IF contains(tokens OF methods[i], UPB tokens OF methods[i], token) THEN
            doc count +:= 1
        FI
    OD;
    
    IF doc count = 0 THEN
        0.0
    ELSE
        ln(REAL(num methods) / REAL(doc count))
    FI
END;

COMMENT Group methods by cluster labels COMMENT

PROC group methods by cluster = ([]METHODMETRICS methods, []INT labels, INT k) [][]METHODMETRICS:
BEGIN
    COMMENT Count methods per cluster COMMENT
    [1:k]INT counts;
    FOR i FROM 1 TO k DO counts[i] := 0 OD;
    
    FOR i FROM LWB labels TO UPB labels DO
        counts[labels[i]] +:= 1
    OD;
    
    COMMENT Allocate groups COMMENT
    [1:k][]METHODMETRICS groups;
    FOR i FROM 1 TO k DO
        [1:counts[i]]METHODMETRICS group;
        groups[i] := group
    OD;
    
    COMMENT Reset counts for filling COMMENT
    FOR i FROM 1 TO k DO counts[i] := 0 OD;
    
    COMMENT Fill groups COMMENT
    FOR i FROM LWB methods TO UPB methods DO
        INT cluster = labels[i - LWB methods + 1];
        counts[cluster] +:= 1;
        groups[cluster][counts[cluster]] := methods[i]
    OD;
    
    groups
END;

COMMENT Create responsibility cluster with meaningful name COMMENT

PROC create responsibility cluster = ([]METHODMETRICS methods, STRING class name) RESPONSIBILITYCLUSTER:
BEGIN
    COMMENT Extract shared dependencies COMMENT
    []STRING shared deps = find shared dependencies(methods);
    
    COMMENT Calculate cohesion score COMMENT
    REAL cohesion = calculate cohesion(methods, shared deps);
    
    COMMENT Generate suggested class name COMMENT
    STRING suggested name = generate class name(methods, class name);
    
    COMMENT Generate justification COMMENT
    STRING justification = generate justification(methods, shared deps);
    
    COMMENT Extract method names COMMENT
    INT num methods = UPB methods - LWB methods + 1;
    [1:num methods]STRING method names;
    FOR i FROM 1 TO num methods DO
        method names[i] := name OF methods[i]
    OD;
    
    RESPONSIBILITYCLUSTER(
        suggested name,
        method names,
        cohesion,
        shared deps,
        justification
    )
END;

COMMENT Find dependencies shared by majority of methods COMMENT

PROC find shared dependencies = ([]METHODMETRICS methods) []STRING:
BEGIN
    INT num methods = UPB methods - LWB methods + 1;
    INT threshold = num methods ÷ 2;  # At least half
    
    COMMENT Collect all fields COMMENT
    [1:1000]STRING all fields;
    [1:1000]INT field counts;
    INT unique count := 0;
    
    FOR i FROM LWB methods TO UPB methods DO
        []STRING fields = accessed fields OF methods[i];
        FOR j FROM LWB fields TO UPB fields DO
            STRING field = fields[j];
            INT idx = find in array(all fields, unique count, field);
            
            IF idx = 0 THEN
                unique count +:= 1;
                all fields[unique count] := field;
                field counts[unique count] := 1
            ELSE
                field counts[idx] +:= 1
            FI
        OD
    OD;
    
    COMMENT Filter to shared ones COMMENT
    INT shared count := 0;
    FOR i FROM 1 TO unique count DO
        IF field counts[i] >= threshold THEN
            shared count +:= 1
        FI
    OD;
    
    [1:shared count]STRING shared;
    INT idx := 1;
    FOR i FROM 1 TO unique count DO
        IF field counts[i] >= threshold THEN
            shared[idx] := all fields[i];
            idx +:= 1
        FI
    OD;
    
    shared
END;

COMMENT Calculate cohesion based on shared dependencies COMMENT

PROC calculate cohesion = ([]METHODMETRICS methods, []STRING shared deps) REAL:
BEGIN
    INT num methods = UPB methods - LWB methods + 1;
    IF num methods <= 1 THEN
        0.5
    ELSE
        REAL avg deps = 0.0;
        FOR i FROM LWB methods TO UPB methods DO
            avg deps +:= REAL(UPB accessed fields OF methods[i])
        OD;
        avg deps /:= REAL(num methods);
        
        IF avg deps = 0.0 THEN
            0.5
        ELSE
            REAL shared count = REAL(UPB shared deps - LWB shared deps + 1);
            REAL cohesion = shared count / (avg deps + 1.0);
            IF cohesion > 1.0 THEN 1.0 ELSE cohesion FI
        FI
    FI
END;

COMMENT Generate meaningful class name from method tokens COMMENT

PROC generate class name = ([]METHODMETRICS methods, STRING original) STRING:
BEGIN
    COMMENT Find most common tokens in method names COMMENT
    [1:100]STRING tokens;
    [1:100]INT token counts;
    INT token count := 0;
    
    FOR i FROM LWB methods TO UPB methods DO
        []STRING parts = split camel case(name OF methods[i]);
        FOR j FROM LWB parts TO UPB parts DO
            IF NOT is common word(parts[j]) THEN
                INT idx = find in array(tokens, token count, parts[j]);
                IF idx = 0 THEN
                    token count +:= 1;
                    tokens[token count] := parts[j];
                    token counts[token count] := 1
                ELSE
                    token counts[idx] +:= 1
                FI
            FI
        OD
    OD;
    
    COMMENT Find top 2 tokens COMMENT
    STRING top1 := "";
    STRING top2 := "";
    INT max1 := 0;
    INT max2 := 0;
    
    FOR i FROM 1 TO token count DO
        IF token counts[i] > max1 THEN
            max2 := max1;
            top2 := top1;
            max1 := token counts[i];
            top1 := tokens[i]
        ELIF token counts[i] > max2 THEN
            max2 := token counts[i];
            top2 := tokens[i]
        FI
    OD;
    
    IF top1 = "" THEN
        original + "Component"
    ELSE
        capitalize(top1) + capitalize(top2) + "Service"
    FI
END;

PROC generate justification = ([]METHODMETRICS methods, []STRING shared deps) STRING:
BEGIN
    INT num methods = UPB methods - LWB methods + 1;
    STRING result := "Cohesive group of " + whole(num methods, 0) + " method(s)";
    
    IF UPB shared deps > 0 THEN
        result +:= " sharing dependencies on ";
        FOR i FROM LWB shared deps TO UPB shared deps DO
            result +:= shared deps[i];
            IF i < UPB shared deps THEN result +:= ", " FI
        OD
    FI;
    
    result
END;

COMMENT Utility procedures COMMENT

PROC contains = ([]STRING array, INT size, STRING item) BOOL:
BEGIN
    BOOL found := FALSE;
    FOR i FROM 1 TO size DO
        IF array[i] = item THEN
            found := TRUE
        FI
    OD;
    found
END;

PROC count token = ([]STRING tokens, STRING target) INT:
BEGIN
    INT count := 0;
    FOR i FROM LWB tokens TO UPB tokens DO
        IF tokens[i] = target THEN count +:= 1 FI
    OD;
    count
END;

PROC find in array = ([]STRING array, INT size, STRING item) INT:
BEGIN
    FOR i FROM 1 TO size DO
        IF array[i] = item THEN RETURN i FI
    OD;
    0
END;

PROC split camel case = (STRING s) []STRING:
    COMMENT Would split on capital letters COMMENT
    ();  # Placeholder

PROC is common word = (STRING word) BOOL:
    word = "Get" OR word = "Set" OR word = "Add" OR word = "Remove" OR
    word = "Delete" OR word = "Update" OR word = "Create";

PROC capitalize = (STRING s) STRING:
    COMMENT Would capitalize first letter COMMENT
    s;  # Placeholder

PROC ln = (REAL x) REAL:
    1.0;  # Placeholder - would use math library


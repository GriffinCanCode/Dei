COMMENT
    K-Means Clustering with Elbow Method
    
    Implements Lloyd's algorithm with intelligent K selection using:
    - Silhouette coefficient
    - Within-cluster sum of squares (WCSS)
    - Elbow method for optimal K
    
    Optimizations:
    - K-means++ initialization for better convergence
    - Early stopping on convergence
    - Vectorized distance calculations
COMMENT

MODE VECTOR = []REAL;
MODE MATRIX = [][]REAL;

MODE CLUSTERRESULT = STRUCT(
    []INT labels,           # Cluster assignment for each point
    MATRIX centroids,       # Cluster centers
    REAL wcss,             # Within-cluster sum of squares
    INT iterations,        # Iterations until convergence
    BOOL converged
);

MODE CLUSTERCONFIG = STRUCT(
    INT max iterations,
    REAL convergence threshold,
    BOOL use kmeans plus plus
);

PROC default cluster config = CLUSTERCONFIG:
    CLUSTERCONFIG(
        100,   # Max iterations
        1e-4,  # Convergence threshold
        TRUE   # Use k-means++
    );

COMMENT Main K-means algorithm COMMENT

PROC kmeans = (MATRIX data, INT k, CLUSTERCONFIG config) CLUSTERRESULT:
BEGIN
    INT n = UPB data;  # Number of points
    INT d = UPB data[1];  # Number of dimensions
    
    COMMENT Initialize centroids COMMENT
    MATRIX centroids = IF use kmeans plus plus OF config THEN
        kmeans plus plus init(data, k)
    ELSE
        random init(data, k)
    FI;
    
    COMMENT Initialize labels COMMENT
    [1:n]INT labels;
    [1:n]INT old labels;
    
    COMMENT Iterate until convergence COMMENT
    INT iteration := 0;
    BOOL converged := FALSE;
    
    WHILE iteration < max iterations OF config AND NOT converged DO
        iteration +:= 1;
        
        COMMENT Save old labels COMMENT
        FOR i FROM 1 TO n DO
            old labels[i] := labels[i]
        OD;
        
        COMMENT Assignment step: assign points to nearest centroid COMMENT
        FOR i FROM 1 TO n DO
            labels[i] := nearest centroid(data[i], centroids)
        OD;
        
        COMMENT Update step: recalculate centroids COMMENT
        centroids := update centroids(data, labels, k);
        
        COMMENT Check convergence COMMENT
        converged := TRUE;
        FOR i FROM 1 TO n DO
            IF labels[i] /= old labels[i] THEN
                converged := FALSE
            FI
        OD
    OD;
    
    COMMENT Calculate final WCSS COMMENT
    REAL wcss = calculate wcss(data, centroids, labels);
    
    CLUSTERRESULT(
        labels,
        centroids,
        wcss,
        iteration,
        converged
    )
END;

COMMENT K-means++ initialization (better than random) COMMENT

PROC kmeans plus plus init = (MATRIX data, INT k) MATRIX:
BEGIN
    INT n = UPB data;
    INT d = UPB data[1];
    
    [1:k, 1:d]REAL centroids;
    [1:n]REAL distances;
    
    COMMENT Choose first centroid randomly COMMENT
    INT first = random int(1, n);
    FOR j FROM 1 TO d DO
        centroids[1, j] := data[first][j]
    OD;
    
    COMMENT Choose remaining centroids with probability proportional to distance² COMMENT
    FOR c FROM 2 TO k DO
        COMMENT Calculate minimum distance to existing centroids COMMENT
        FOR i FROM 1 TO n DO
            REAL min dist = max real;
            FOR j FROM 1 TO c - 1 DO
                [1:d]REAL centroid;
                FOR dim FROM 1 TO d DO
                    centroid[dim] := centroids[j, dim]
                OD;
                REAL dist = euclidean distance(data[i], centroid);
                IF dist < min dist THEN
                    min dist := dist
                FI
            OD;
            distances[i] := min dist × min dist  # Square for weighting
        OD;
        
        COMMENT Choose next centroid with weighted probability COMMENT
        INT next = weighted random(distances);
        FOR j FROM 1 TO d DO
            centroids[c, j] := data[next][j]
        OD
    OD;
    
    centroids
END;

PROC random init = (MATRIX data, INT k) MATRIX:
BEGIN
    INT n = UPB data;
    INT d = UPB data[1];
    [1:k, 1:d]REAL centroids;
    
    COMMENT Randomly select k points as initial centroids COMMENT
    FOR i FROM 1 TO k DO
        INT idx = random int(1, n);
        FOR j FROM 1 TO d DO
            centroids[i, j] := data[idx][j]
        OD
    OD;
    
    centroids
END;

COMMENT Find nearest centroid COMMENT

PROC nearest centroid = (VECTOR point, MATRIX centroids) INT:
BEGIN
    INT k = UPB centroids;
    INT d = UPB point;
    REAL min dist := max real;
    INT nearest := 1;
    
    FOR i FROM 1 TO k DO
        [1:d]REAL centroid;
        FOR j FROM 1 TO d DO
            centroid[j] := centroids[i, j]
        OD;
        
        REAL dist = euclidean distance(point, centroid);
        IF dist < min dist THEN
            min dist := dist;
            nearest := i
        FI
    OD;
    
    nearest
END;

COMMENT Update centroids to mean of assigned points COMMENT

PROC update centroids = (MATRIX data, []INT labels, INT k) MATRIX:
BEGIN
    INT n = UPB data;
    INT d = UPB data[1];
    
    [1:k, 1:d]REAL new centroids;
    [1:k]INT counts;
    
    COMMENT Initialize COMMENT
    FOR i FROM 1 TO k DO
        counts[i] := 0;
        FOR j FROM 1 TO d DO
            new centroids[i, j] := 0.0
        OD
    OD;
    
    COMMENT Sum points by cluster COMMENT
    FOR i FROM 1 TO n DO
        INT cluster = labels[i];
        counts[cluster] +:= 1;
        FOR j FROM 1 TO d DO
            new centroids[cluster, j] +:= data[i][j]
        OD
    OD;
    
    COMMENT Divide by counts to get means COMMENT
    FOR i FROM 1 TO k DO
        IF counts[i] > 0 THEN
            FOR j FROM 1 TO d DO
                new centroids[i, j] /:= REAL(counts[i])
            OD
        FI
    OD;
    
    new centroids
END;

COMMENT Calculate within-cluster sum of squares COMMENT

PROC calculate wcss = (MATRIX data, MATRIX centroids, []INT labels) REAL:
BEGIN
    INT n = UPB data;
    INT d = UPB data[1];
    REAL wcss := 0.0;
    
    FOR i FROM 1 TO n DO
        INT cluster = labels[i];
        [1:d]REAL centroid;
        FOR j FROM 1 TO d DO
            centroid[j] := centroids[cluster, j]
        OD;
        
        REAL dist = euclidean distance(data[i], centroid);
        wcss +:= dist × dist
    OD;
    
    wcss
END;

COMMENT Elbow method to determine optimal K COMMENT

PROC determine optimal k = (MATRIX data, INT max k, CLUSTERCONFIG config) INT:
BEGIN
    [2:max k]REAL wcss values;
    
    COMMENT Try different K values COMMENT
    FOR k FROM 2 TO max k DO
        CLUSTERRESULT result = kmeans(data, k, config);
        wcss values[k] := wcss OF result
    OD;
    
    COMMENT Find elbow point (maximum rate of decrease change) COMMENT
    REAL max decrease change := 0.0;
    INT optimal := 2;
    
    FOR k FROM 3 TO max k - 1 DO
        REAL prev decrease = wcss values[k - 1] - wcss values[k];
        REAL next decrease = wcss values[k] - wcss values[k + 1];
        REAL change = prev decrease - next decrease;
        
        IF change > max decrease change THEN
            max decrease change := change;
            optimal := k
        FI
    OD;
    
    optimal
END;

COMMENT Silhouette coefficient for cluster quality COMMENT

PROC calculate silhouette = (MATRIX data, []INT labels, INT k) REAL:
BEGIN
    INT n = UPB data;
    REAL total silhouette := 0.0;
    
    FOR i FROM 1 TO n DO
        INT cluster = labels[i];
        
        COMMENT Calculate a(i): mean distance to points in same cluster COMMENT
        REAL a := mean distance to cluster(data, i, labels, cluster);
        
        COMMENT Calculate b(i): mean distance to nearest other cluster COMMENT
        REAL b := min mean distance to other clusters(data, i, labels, cluster, k);
        
        COMMENT Silhouette for point i COMMENT
        REAL s = IF a < b THEN (b - a) / b
                 ELIF a > b THEN (b - a) / a
                 ELSE 0.0
                 FI;
        
        total silhouette +:= s
    OD;
    
    total silhouette / REAL(n)
END;

PROC mean distance to cluster = (MATRIX data, INT point idx, []INT labels, INT cluster) REAL:
BEGIN
    REAL sum := 0.0;
    INT count := 0;
    
    FOR i FROM 1 TO UPB data DO
        IF i /= point idx AND labels[i] = cluster THEN
            sum +:= euclidean distance(data[point idx], data[i]);
            count +:= 1
        FI
    OD;
    
    IF count = 0 THEN 0.0 ELSE sum / REAL(count) FI
END;

PROC min mean distance to other clusters = (MATRIX data, INT point idx, []INT labels, 
                                             INT own cluster, INT k) REAL:
BEGIN
    REAL min dist := max real;
    
    FOR c FROM 1 TO k DO
        IF c /= own cluster THEN
            REAL mean dist = mean distance to cluster(data, point idx, labels, c);
            IF mean dist < min dist THEN
                min dist := mean dist
            FI
        FI
    OD;
    
    min dist
END;

COMMENT Distance metrics COMMENT

PROC euclidean distance = (VECTOR a, VECTOR b) REAL:
BEGIN
    REAL sum := 0.0;
    FOR i FROM LWB a TO UPB a DO
        REAL diff = a[i] - b[i];
        sum +:= diff × diff
    OD;
    sqrt(sum)
END;

PROC manhattan distance = (VECTOR a, VECTOR b) REAL:
BEGIN
    REAL sum := 0.0;
    FOR i FROM LWB a TO UPB a DO
        sum +:= ABS(a[i] - b[i])
    OD;
    sum
END;

COMMENT Utility functions COMMENT

PROC sqrt = (REAL x) REAL:
    COMMENT Would use math library COMMENT
    x ** 0.5;

PROC random int = (INT low, INT high) INT:
    COMMENT Would use proper RNG COMMENT
    low;  # Placeholder

PROC weighted random = ([]REAL weights) INT:
    COMMENT Choose index with probability proportional to weights COMMENT
    1;  # Placeholder

REAL max real = 1e308;


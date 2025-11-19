using Accord.MachineLearning;
using GodClassDetector.Core.Interfaces;
using GodClassDetector.Core.Models;

namespace GodClassDetector.Clustering.Analyzers;

/// <summary>
/// Performs semantic clustering of methods to identify distinct responsibilities
/// </summary>
public sealed class SemanticClusteringAnalyzer : ISemanticAnalyzer
{
    public Task<Result<IReadOnlyList<ResponsibilityCluster>>> AnalyzeAsync(
        ClassMetrics classMetrics,
        DetectionThresholds thresholds,
        CancellationToken cancellationToken = default)
    {
        try
        {
            if (classMetrics.Methods.Count < thresholds.MinClusterSize)
            {
                return Task.FromResult(
                    Result<IReadOnlyList<ResponsibilityCluster>>.Success(
                        Array.Empty<ResponsibilityCluster>()));
            }

            var clusters = PerformClustering(classMetrics, thresholds);
            return Task.FromResult(Result<IReadOnlyList<ResponsibilityCluster>>.Success(clusters));
        }
        catch (Exception ex)
        {
            return Task.FromResult(
                Result<IReadOnlyList<ResponsibilityCluster>>.Failure(
                    $"Clustering failed: {ex.Message}"));
        }
    }

    private IReadOnlyList<ResponsibilityCluster> PerformClustering(
        ClassMetrics classMetrics,
        DetectionThresholds thresholds)
    {
        var methods = classMetrics.Methods.ToList();
        
        // Build feature vectors for each method
        var (featureVectors, vocabulary) = BuildFeatureVectors(methods);

        // Determine optimal number of clusters (2 to sqrt(n))
        var maxClusters = Math.Min((int)Math.Sqrt(methods.Count), 5);
        var optimalK = DetermineOptimalClusters(featureVectors, maxClusters);

        // Perform K-means clustering
        var kmeans = new KMeans(optimalK);
        var clusters = kmeans.Learn(featureVectors);
        var labels = clusters.Decide(featureVectors);

        // Group methods by cluster
        var methodGroups = methods
            .Select((method, index) => new { Method = method, Cluster = labels[index] })
            .GroupBy(x => x.Cluster)
            .Where(g => g.Count() >= thresholds.MinClusterSize)
            .ToList();

        // Create responsibility clusters
        var responsibilityClusters = new List<ResponsibilityCluster>();

        foreach (var group in methodGroups)
        {
            var groupMethods = group.Select(x => x.Method).ToList();
            var cluster = CreateResponsibilityCluster(groupMethods, classMetrics.ClassName);
            responsibilityClusters.Add(cluster);
        }

        return responsibilityClusters;
    }

    private (double[][] FeatureVectors, HashSet<string> Vocabulary) BuildFeatureVectors(
        List<MethodMetrics> methods)
    {
        // Build vocabulary from all method tokens
        var vocabulary = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        foreach (var method in methods)
        {
            foreach (var token in method.Tokens)
                vocabulary.Add(token);
        }

        var vocabList = vocabulary.ToList();
        var featureVectors = new double[methods.Count][];

        // Create TF-IDF-like feature vectors
        for (int i = 0; i < methods.Count; i++)
        {
            var method = methods[i];
            var vector = new double[vocabList.Count];

            // Token frequency
            var tokenCounts = method.Tokens
                .GroupBy(t => t, StringComparer.OrdinalIgnoreCase)
                .ToDictionary(g => g.Key, g => g.Count(), StringComparer.OrdinalIgnoreCase);

            for (int j = 0; j < vocabList.Count; j++)
            {
                var token = vocabList[j];
                vector[j] = tokenCounts.TryGetValue(token, out var count)
                    ? count / (double)method.Tokens.Count
                    : 0;
            }

            // Add structural features (normalized)
            var structuralFeatures = new[]
            {
                method.LineCount / 100.0,
                method.CyclomaticComplexity / 20.0,
                method.CalledMethods.Count / 10.0,
                method.AccessedFields.Count / 10.0,
                method.IsPublic ? 1.0 : 0.0,
                method.IsStatic ? 1.0 : 0.0
            };

            featureVectors[i] = vector.Concat(structuralFeatures).ToArray();
        }

        return (featureVectors, vocabulary);
    }

    private int DetermineOptimalClusters(double[][] featureVectors, int maxK)
    {
        if (featureVectors.Length <= 3)
            return 2;

        var bestK = 2;
        var bestScore = double.MaxValue;

        // Use elbow method with silhouette coefficient
        for (int k = 2; k <= Math.Min(maxK, featureVectors.Length - 1); k++)
        {
            try
            {
                var kmeans = new KMeans(k);
                var clusters = kmeans.Learn(featureVectors);
                var labels = clusters.Decide(featureVectors);

                // Calculate within-cluster sum of squares
                var wcss = CalculateWCSS(featureVectors, clusters.Centroids, labels);

                if (wcss < bestScore)
                {
                    bestScore = wcss;
                    bestK = k;
                }
            }
            catch
            {
                // If clustering fails for this k, skip it
                continue;
            }
        }

        return bestK;
    }

    private double CalculateWCSS(double[][] data, double[][] centroids, int[] labels)
    {
        var wcss = 0.0;
        for (int i = 0; i < data.Length; i++)
        {
            var centroid = centroids[labels[i]];
            wcss += EuclideanDistance(data[i], centroid);
        }
        return wcss;
    }

    private double EuclideanDistance(double[] a, double[] b)
    {
        return Math.Sqrt(a.Zip(b, (x, y) => Math.Pow(x - y, 2)).Sum());
    }

    private ResponsibilityCluster CreateResponsibilityCluster(
        List<MethodMetrics> methods,
        string originalClassName)
    {
        // Extract shared dependencies and common tokens
        var sharedDependencies = methods
            .SelectMany(m => m.AccessedFields)
            .GroupBy(f => f)
            .Where(g => g.Count() >= methods.Count / 2)
            .Select(g => g.Key)
            .ToList();

        // Calculate cohesion score based on shared dependencies
        var cohesionScore = methods.Count > 1
            ? sharedDependencies.Count / (double)methods.Average(m => m.AccessedFields.Count + 1)
            : 0.5;

        // Generate suggested class name from common tokens
        var suggestedName = GenerateSuggestedClassName(methods, originalClassName);

        // Generate justification
        var justification = GenerateJustification(methods, sharedDependencies);

        return new ResponsibilityCluster
        {
            SuggestedClassName = suggestedName,
            Methods = methods,
            CohesionScore = Math.Min(cohesionScore, 1.0),
            SharedDependencies = sharedDependencies,
            Justification = justification
        };
    }

    private string GenerateSuggestedClassName(List<MethodMetrics> methods, string originalClassName)
    {
        // Find most common tokens in method names
        var tokenFrequency = methods
            .SelectMany(m => SplitMethodName(m.MethodName))
            .Where(t => !IsCommonWord(t))
            .GroupBy(t => t, StringComparer.OrdinalIgnoreCase)
            .OrderByDescending(g => g.Count())
            .Take(2)
            .Select(g => CapitalizeFirst(g.Key))
            .ToList();

        if (tokenFrequency.Any())
        {
            var baseName = string.Join("", tokenFrequency);
            return $"{baseName}Service";
        }

        return $"{originalClassName}Component";
    }

    private string GenerateJustification(List<MethodMetrics> methods, List<string> sharedDeps)
    {
        var methodNames = string.Join(", ", methods.Select(m => m.MethodName).Take(5));
        var depInfo = sharedDeps.Any()
            ? $" sharing dependencies on {string.Join(", ", sharedDeps.Take(3))}"
            : "";

        return $"Cohesive group of {methods.Count} method(s) ({methodNames}...){depInfo}";
    }

    private static IEnumerable<string> SplitMethodName(string methodName)
    {
        return System.Text.RegularExpressions.Regex
            .Split(methodName, @"(?<!^)(?=[A-Z])")
            .Where(s => !string.IsNullOrWhiteSpace(s) && s.Length > 2);
    }

    private static bool IsCommonWord(string word)
    {
        var commonWords = new HashSet<string>(StringComparer.OrdinalIgnoreCase)
        {
            "Get", "Set", "Add", "Remove", "Delete", "Update", "Create", "Save",
            "Load", "Handle", "Process", "Execute", "Run", "Do", "Is", "Has", "Can"
        };
        return commonWords.Contains(word);
    }

    private static string CapitalizeFirst(string word)
    {
        if (string.IsNullOrEmpty(word))
            return word;
        return char.ToUpper(word[0]) + word[1..].ToLower();
    }
}


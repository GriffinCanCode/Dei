namespace GodClassDetector.Core.Models;

/// <summary>
/// Represents the complete analysis result for a class
/// </summary>
public sealed record AnalysisResult
{
    public required ClassMetrics ClassMetrics { get; init; }
    public required bool IsGodClass { get; init; }
    public required IReadOnlyList<ResponsibilityCluster> SuggestedExtractions { get; init; }
    public required IReadOnlyList<GodMethodResult> GodMethods { get; init; }
    public required DateTime AnalyzedAt { get; init; }
    public required string Summary { get; init; }

    public bool HasIssues => IsGodClass || GodMethods.Any();

    public static AnalysisResult CreateHealthy(ClassMetrics metrics) =>
        new()
        {
            ClassMetrics = metrics,
            IsGodClass = false,
            SuggestedExtractions = Array.Empty<ResponsibilityCluster>(),
            GodMethods = Array.Empty<GodMethodResult>(),
            AnalyzedAt = DateTime.UtcNow,
            Summary = $"Class '{metrics.ClassName}' is within acceptable thresholds."
        };
}


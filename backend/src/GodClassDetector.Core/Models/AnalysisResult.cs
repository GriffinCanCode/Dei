namespace GodClassDetector.Core.Models;

/// <summary>
/// Represents the complete analysis result for a class
/// </summary>
public sealed record AnalysisResult
{
    public required ClassMetrics ClassMetrics { get; init; }
    public required bool IsGodClass { get; init; }
    public required IReadOnlyList<ResponsibilityCluster> SuggestedExtractions { get; init; }
    public required DateTime AnalyzedAt { get; init; }
    public required string Summary { get; init; }

    public static AnalysisResult CreateHealthy(ClassMetrics metrics) =>
        new()
        {
            ClassMetrics = metrics,
            IsGodClass = false,
            SuggestedExtractions = Array.Empty<ResponsibilityCluster>(),
            AnalyzedAt = DateTime.UtcNow,
            Summary = $"Class '{metrics.ClassName}' is within acceptable thresholds."
        };
}


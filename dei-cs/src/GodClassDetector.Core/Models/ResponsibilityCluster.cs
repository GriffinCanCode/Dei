namespace GodClassDetector.Core.Models;

/// <summary>
/// Represents a cluster of methods that share a common responsibility
/// </summary>
public sealed record ResponsibilityCluster
{
    public required string SuggestedClassName { get; init; }
    public required IReadOnlyList<MethodMetrics> Methods { get; init; }
    public required double CohesionScore { get; init; }
    public required IReadOnlyList<string> SharedDependencies { get; init; }
    public required string Justification { get; init; }
}


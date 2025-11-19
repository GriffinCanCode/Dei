namespace GodClassDetector.Core.Models;

/// <summary>
/// Configuration thresholds for detecting god classes
/// </summary>
public sealed record DetectionThresholds
{
    public int MaxLines { get; init; } = 300;
    public int MaxMethods { get; init; } = 20;
    public int MaxComplexity { get; init; } = 50;
    public int MinClusterSize { get; init; } = 3;
    public double ClusterThreshold { get; init; } = 0.7;
}


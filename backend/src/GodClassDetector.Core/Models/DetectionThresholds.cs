namespace GodClassDetector.Core.Models;

/// <summary>
/// Configuration thresholds for detecting god files, classes, and methods
/// </summary>
public sealed record DetectionThresholds
{
    // Class-level thresholds
    public int MaxLines { get; init; } = 300;
    public int MaxMethods { get; init; } = 20;
    public int MaxComplexity { get; init; } = 50;
    public int MinClusterSize { get; init; } = 3;
    public double ClusterThreshold { get; init; } = 0.7;

    // Method-level thresholds (god functions)
    public int MaxMethodLines { get; init; } = 50;
    public int MaxMethodComplexity { get; init; } = 10;
    public int MaxMethodParameters { get; init; } = 5;

    // File-level thresholds (god files)
    public int MaxClassesPerFile { get; init; } = 3;
    public int MaxFileLinesOfCode { get; init; } = 500;
}


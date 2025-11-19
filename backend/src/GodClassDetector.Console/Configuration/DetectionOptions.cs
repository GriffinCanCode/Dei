using GodClassDetector.Core.Models;

namespace GodClassDetector.Console.Configuration;

/// <summary>
/// Configuration options for the detector application
/// </summary>
public sealed class DetectionOptions
{
    public const string SectionName = "DetectionThresholds";

    public int MaxLines { get; set; } = 300;
    public int MaxMethods { get; set; } = 20;
    public int MaxComplexity { get; set; } = 50;
    public int MinClusterSize { get; set; } = 3;
    public double ClusterThreshold { get; set; } = 0.7;

    public DetectionThresholds ToThresholds() => new()
    {
        MaxLines = MaxLines,
        MaxMethods = MaxMethods,
        MaxComplexity = MaxComplexity,
        MinClusterSize = MinClusterSize,
        ClusterThreshold = ClusterThreshold
    };
}


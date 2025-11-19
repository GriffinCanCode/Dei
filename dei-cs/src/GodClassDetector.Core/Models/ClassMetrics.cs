namespace GodClassDetector.Core.Models;

/// <summary>
/// Represents comprehensive metrics for a C# class
/// </summary>
public sealed record ClassMetrics
{
    public required string ClassName { get; init; }
    public required string FullyQualifiedName { get; init; }
    public required string FilePath { get; init; }
    public required int LineCount { get; init; }
    public required int MethodCount { get; init; }
    public required int PropertyCount { get; init; }
    public required int FieldCount { get; init; }
    public required int CyclomaticComplexity { get; init; }
    public required IReadOnlyList<MethodMetrics> Methods { get; init; }
    public required IReadOnlyList<string> Dependencies { get; init; }

    public bool IsGodClass(DetectionThresholds thresholds) =>
        LineCount > thresholds.MaxLines ||
        MethodCount > thresholds.MaxMethods ||
        CyclomaticComplexity > thresholds.MaxComplexity;
}


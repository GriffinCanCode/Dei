namespace GodClassDetector.Core.Models;

/// <summary>
/// Represents metrics for a single method within a class
/// </summary>
public sealed record MethodMetrics
{
    public required string MethodName { get; init; }
    public required int LineCount { get; init; }
    public required int CyclomaticComplexity { get; init; }
    public required IReadOnlyList<string> CalledMethods { get; init; }
    public required IReadOnlyList<string> AccessedFields { get; init; }
    public required IReadOnlyList<string> Parameters { get; init; }
    public required string ReturnType { get; init; }
    public required bool IsPublic { get; init; }
    public required bool IsStatic { get; init; }
    public required IReadOnlyList<string> Tokens { get; init; } // For semantic analysis
}


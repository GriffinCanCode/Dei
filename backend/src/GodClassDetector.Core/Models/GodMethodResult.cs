namespace GodClassDetector.Core.Models;

/// <summary>
/// Represents a method that exceeds complexity thresholds (god function)
/// </summary>
public sealed record GodMethodResult
{
    public required MethodMetrics Method { get; init; }
    public required string ClassName { get; init; }
    public required string FilePath { get; init; }
    public required IReadOnlyList<string> Violations { get; init; }
    public required int ViolationScore { get; init; }

    public static GodMethodResult Create(
        MethodMetrics method,
        string className,
        string filePath,
        DetectionThresholds thresholds)
    {
        var violations = new List<string>();
        var score = 0;

        if (method.LineCount > thresholds.MaxMethodLines)
        {
            violations.Add($"Lines ({method.LineCount}) exceeds threshold ({thresholds.MaxMethodLines})");
            score += method.LineCount - thresholds.MaxMethodLines;
        }

        if (method.CyclomaticComplexity > thresholds.MaxMethodComplexity)
        {
            violations.Add($"Complexity ({method.CyclomaticComplexity}) exceeds threshold ({thresholds.MaxMethodComplexity})");
            score += (method.CyclomaticComplexity - thresholds.MaxMethodComplexity) * 2;
        }

        if (method.Parameters.Count > thresholds.MaxMethodParameters)
        {
            violations.Add($"Parameters ({method.Parameters.Count}) exceeds threshold ({thresholds.MaxMethodParameters})");
            score += method.Parameters.Count - thresholds.MaxMethodParameters;
        }

        return new GodMethodResult
        {
            Method = method,
            ClassName = className,
            FilePath = filePath,
            Violations = violations,
            ViolationScore = score
        };
    }

    public bool IsGodMethod => Violations.Any();
}


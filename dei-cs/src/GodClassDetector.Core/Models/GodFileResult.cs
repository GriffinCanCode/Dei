namespace GodClassDetector.Core.Models;

/// <summary>
/// Represents a file that exceeds organization thresholds (god file)
/// </summary>
public sealed record GodFileResult
{
    public required string FilePath { get; init; }
    public required int ClassCount { get; init; }
    public required int TotalLines { get; init; }
    public required IReadOnlyList<string> ClassNames { get; init; }
    public required IReadOnlyList<string> Violations { get; init; }
    public required int ViolationScore { get; init; }

    public static GodFileResult Create(
        string filePath,
        IReadOnlyList<ClassMetrics> classes,
        DetectionThresholds thresholds)
    {
        var violations = new List<string>();
        var score = 0;

        if (classes.Count > thresholds.MaxClassesPerFile)
        {
            violations.Add($"Classes ({classes.Count}) exceeds threshold ({thresholds.MaxClassesPerFile})");
            score += (classes.Count - thresholds.MaxClassesPerFile) * 5;
        }

        var totalLines = classes.Sum(c => c.LineCount);
        if (totalLines > thresholds.MaxFileLinesOfCode)
        {
            violations.Add($"Total lines ({totalLines}) exceeds threshold ({thresholds.MaxFileLinesOfCode})");
            score += totalLines - thresholds.MaxFileLinesOfCode;
        }

        return new GodFileResult
        {
            FilePath = filePath,
            ClassCount = classes.Count,
            TotalLines = totalLines,
            ClassNames = classes.Select(c => c.ClassName).ToList(),
            Violations = violations,
            ViolationScore = score
        };
    }

    public bool IsGodFile => Violations.Any();
}


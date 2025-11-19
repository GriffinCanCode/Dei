using GodClassDetector.Core.Models;

namespace GodClassDetector.Core.Interfaces;

/// <summary>
/// Calculates various complexity metrics for classes and methods
/// </summary>
public interface IMetricsCalculator
{
    int CalculateCyclomaticComplexity(string methodBody);
    int CalculateLineCount(string source);
    IReadOnlyList<string> ExtractDependencies(string source);
}


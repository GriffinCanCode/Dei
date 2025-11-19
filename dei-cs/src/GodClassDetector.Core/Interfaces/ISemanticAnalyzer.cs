using GodClassDetector.Core.Models;

namespace GodClassDetector.Core.Interfaces;

/// <summary>
/// Performs semantic analysis to cluster methods by responsibility
/// </summary>
public interface ISemanticAnalyzer
{
    Task<Result<IReadOnlyList<ResponsibilityCluster>>> AnalyzeAsync(
        ClassMetrics classMetrics,
        DetectionThresholds thresholds,
        CancellationToken cancellationToken = default);
}


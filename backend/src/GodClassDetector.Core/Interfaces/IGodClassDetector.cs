using GodClassDetector.Core.Models;

namespace GodClassDetector.Core.Interfaces;

/// <summary>
/// Main service for detecting god classes and suggesting refactorings
/// </summary>
public interface IGodClassDetector
{
    Task<Result<AnalysisResult>> AnalyzeClassAsync(
        string filePath,
        DetectionThresholds? thresholds = null,
        CancellationToken cancellationToken = default);

    Task<Result<IReadOnlyList<AnalysisResult>>> AnalyzeProjectAsync(
        string projectPath,
        DetectionThresholds? thresholds = null,
        CancellationToken cancellationToken = default);
}


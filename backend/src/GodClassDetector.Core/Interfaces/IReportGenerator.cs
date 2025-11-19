using GodClassDetector.Core.Models;

namespace GodClassDetector.Core.Interfaces;

/// <summary>
/// Generates reports from analysis results
/// </summary>
public interface IReportGenerator
{
    Task<string> GenerateConsoleReportAsync(IReadOnlyList<AnalysisResult> results);
    Task<string> GenerateJsonReportAsync(IReadOnlyList<AnalysisResult> results);
    Task<string> GenerateMarkdownReportAsync(IReadOnlyList<AnalysisResult> results);
}


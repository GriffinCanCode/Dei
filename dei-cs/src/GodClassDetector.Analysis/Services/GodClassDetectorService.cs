using GodClassDetector.Core.Interfaces;
using GodClassDetector.Core.Models;

namespace GodClassDetector.Analysis.Services;

/// <summary>
/// Main service orchestrating god class detection and analysis
/// </summary>
public sealed class GodClassDetectorService : IGodClassDetector
{
    private readonly IClassParser _classParser;
    private readonly ISemanticAnalyzer _semanticAnalyzer;
    private readonly FileSystemASTBuilder _astBuilder;
    private readonly ParallelASTTraverser _astTraverser;

    public GodClassDetectorService(
        IClassParser classParser,
        ISemanticAnalyzer semanticAnalyzer)
    {
        _classParser = classParser ?? throw new ArgumentNullException(nameof(classParser));
        _semanticAnalyzer = semanticAnalyzer ?? throw new ArgumentNullException(nameof(semanticAnalyzer));
        _astBuilder = new FileSystemASTBuilder();
        _astTraverser = new ParallelASTTraverser(classParser, this);
    }

    public async Task<Result<AnalysisResult>> AnalyzeClassAsync(
        string filePath,
        DetectionThresholds? thresholds = null,
        CancellationToken cancellationToken = default)
    {
        thresholds ??= new DetectionThresholds();

        var parseResult = await _classParser.ParseFileAsync(filePath, cancellationToken);
        if (!parseResult.IsSuccess)
            return Result<AnalysisResult>.Failure(parseResult.Error);

        var classes = parseResult.Value;
        if (!classes.Any())
            return Result<AnalysisResult>.Failure($"No classes found in {filePath}");

        // Analyze the first class in the file
        var classMetrics = classes.First();
        return await AnalyzeClassMetricsAsync(classMetrics, thresholds, cancellationToken);
    }

    public async Task<Result<IReadOnlyList<AnalysisResult>>> AnalyzeProjectAsync(
        string projectPath,
        DetectionThresholds? thresholds = null,
        CancellationToken cancellationToken = default)
    {
        thresholds ??= new DetectionThresholds();

        var parseResult = await _classParser.ParseDirectoryAsync(projectPath, cancellationToken);
        if (!parseResult.IsSuccess)
            return Result<IReadOnlyList<AnalysisResult>>.Failure(parseResult.Error);

        var results = new List<AnalysisResult>();

        foreach (var classMetrics in parseResult.Value)
        {
            var analysisResult = await AnalyzeClassMetricsAsync(classMetrics, thresholds, cancellationToken);
            if (analysisResult.IsSuccess)
                results.Add(analysisResult.Value);
        }

        return Result<IReadOnlyList<AnalysisResult>>.Success(results);
    }

    public async Task<Result<FileSystemNode>> AnalyzeProjectASTAsync(
        string projectPath,
        DetectionThresholds? thresholds = null,
        CancellationToken cancellationToken = default)
    {
        thresholds ??= new DetectionThresholds();

        // Build filesystem AST
        var astResult = await _astBuilder.BuildASTAsync(projectPath, cancellationToken);
        if (!astResult.IsSuccess)
            return Result<FileSystemNode>.Failure(astResult.Error);

        // Traverse AST in parallel, building sub-ASTs of classes/functions
        var traversalResult = await _astTraverser.TraverseAndAnalyzeAsync(
            astResult.Value,
            thresholds,
            cancellationToken);

        return traversalResult;
    }

    private async Task<Result<AnalysisResult>> AnalyzeClassMetricsAsync(
        ClassMetrics classMetrics,
        DetectionThresholds thresholds,
        CancellationToken cancellationToken)
    {
        var isGodClass = classMetrics.IsGodClass(thresholds);

        if (!isGodClass)
            return Result<AnalysisResult>.Success(AnalysisResult.CreateHealthy(classMetrics));

        // Perform semantic clustering for god classes
        var clusterResult = await _semanticAnalyzer.AnalyzeAsync(classMetrics, thresholds, cancellationToken);
        if (!clusterResult.IsSuccess)
            return Result<AnalysisResult>.Failure(clusterResult.Error);

        var summary = GenerateSummary(classMetrics, thresholds, clusterResult.Value);

        var result = new AnalysisResult
        {
            ClassMetrics = classMetrics,
            IsGodClass = true,
            SuggestedExtractions = clusterResult.Value,
            GodMethods = Array.Empty<GodMethodResult>(),
            AnalyzedAt = DateTime.UtcNow,
            Summary = summary
        };

        return Result<AnalysisResult>.Success(result);
    }

    private static string GenerateSummary(
        ClassMetrics metrics,
        DetectionThresholds thresholds,
        IReadOnlyList<ResponsibilityCluster> clusters)
    {
        var violations = new List<string>();

        if (metrics.LineCount > thresholds.MaxLines)
            violations.Add($"Line count ({metrics.LineCount}) exceeds threshold ({thresholds.MaxLines})");

        if (metrics.MethodCount > thresholds.MaxMethods)
            violations.Add($"Method count ({metrics.MethodCount}) exceeds threshold ({thresholds.MaxMethods})");

        if (metrics.CyclomaticComplexity > thresholds.MaxComplexity)
            violations.Add($"Complexity ({metrics.CyclomaticComplexity}) exceeds threshold ({thresholds.MaxComplexity})");

        var summary = $"God class detected in '{metrics.ClassName}':\n" +
                     string.Join("\n", violations.Select(v => $"  • {v}"));

        if (clusters.Any())
            summary += $"\n\nSuggested {clusters.Count} extraction(s) to improve maintainability.";

        return summary;
    }
}


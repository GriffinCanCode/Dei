using System.Collections.Concurrent;
using GodClassDetector.Core.Interfaces;
using GodClassDetector.Core.Models;

namespace GodClassDetector.Analysis.Services;

/// <summary>
/// Traverses filesystem AST in parallel, building sub-ASTs for classes and functions
/// </summary>
public sealed class ParallelASTTraverser : IParallelASTTraverser
{
    private readonly IClassParser _classParser;
    private readonly IGodClassDetector _godClassDetector;
    private readonly ParallelOptions _parallelOptions;

    public ParallelASTTraverser(
        IClassParser classParser,
        IGodClassDetector godClassDetector,
        int? maxDegreeOfParallelism = null)
    {
        _classParser = classParser ?? throw new ArgumentNullException(nameof(classParser));
        _godClassDetector = godClassDetector ?? throw new ArgumentNullException(nameof(godClassDetector));
        
        _parallelOptions = new ParallelOptions
        {
            MaxDegreeOfParallelism = maxDegreeOfParallelism ?? Environment.ProcessorCount
        };
    }

    public async Task<Result<FileSystemNode>> TraverseAndAnalyzeAsync(
        FileSystemNode root,
        DetectionThresholds thresholds,
        CancellationToken cancellationToken = default)
    {
        try
        {
            _parallelOptions.CancellationToken = cancellationToken;
            var updatedRoot = await TraverseNodeAsync(root, thresholds, cancellationToken);
            return Result<FileSystemNode>.Success(updatedRoot);
        }
        catch (Exception ex)
        {
            return Result<FileSystemNode>.Failure($"Error traversing AST: {ex.Message}");
        }
    }

    private async Task<FileSystemNode> TraverseNodeAsync(
        FileSystemNode node,
        DetectionThresholds thresholds,
        CancellationToken cancellationToken)
    {
        if (node.IsFile && node.IsCSharpFile)
        {
            // Parse and analyze the file to build class/function sub-AST
            return await AnalyzeFileNodeAsync(node, thresholds, cancellationToken);
        }

        if (node.IsDirectory)
        {
            // Traverse children in parallel
            var updatedChildren = await TraverseChildrenParallelAsync(node.Children, thresholds, cancellationToken);
            return node.WithChildren(updatedChildren);
        }

        return node;
    }

    private async Task<IReadOnlyList<FileSystemNode>> TraverseChildrenParallelAsync(
        IReadOnlyList<FileSystemNode> children,
        DetectionThresholds thresholds,
        CancellationToken cancellationToken)
    {
        if (!children.Any())
            return children;

        var results = new ConcurrentBag<(int index, FileSystemNode node)>();

        await Parallel.ForEachAsync(
            children.Select((child, index) => (child, index)),
            _parallelOptions,
            async (item, ct) =>
            {
                var updatedChild = await TraverseNodeAsync(item.child, thresholds, ct);
                results.Add((item.index, updatedChild));
            });

        return results
            .OrderBy(r => r.index)
            .Select(r => r.node)
            .ToList();
    }

    private async Task<FileSystemNode> AnalyzeFileNodeAsync(
        FileSystemNode node,
        DetectionThresholds thresholds,
        CancellationToken cancellationToken)
    {
        // Build class/function sub-AST by parsing the file
        var parseResult = await _classParser.ParseFileAsync(node.FullPath, cancellationToken);
        
        if (!parseResult.IsSuccess)
            return node;

        var classMetrics = parseResult.Value;
        if (!classMetrics.Any())
            return node;

        // Update node with class metrics (sub-AST of classes and their methods)
        var nodeWithMetrics = node.WithClassMetrics(classMetrics);

        // Check for god file (too many classes or too long)
        var godFile = DetectGodFile(node.FullPath, classMetrics, thresholds);
        if (godFile.IsGodFile)
            nodeWithMetrics = nodeWithMetrics.WithGodFileResult(godFile);

        // Analyze for god classes and god methods
        var analysisResults = new List<AnalysisResult>();
        
        foreach (var metrics in classMetrics)
        {
            var isGodClass = metrics.IsGodClass(thresholds);
            var godMethods = DetectGodMethods(metrics, thresholds);
            
            if (!isGodClass && !godMethods.Any())
            {
                analysisResults.Add(AnalysisResult.CreateHealthy(metrics));
                continue;
            }

            // For god classes, perform full analysis with clustering
            if (isGodClass)
            {
                var analysisResult = await AnalyzeClassMetricsAsync(metrics, thresholds, godMethods, cancellationToken);
                if (analysisResult.IsSuccess)
                    analysisResults.Add(analysisResult.Value);
            }
            else
            {
                // Class is healthy but has god methods
                analysisResults.Add(new AnalysisResult
                {
                    ClassMetrics = metrics,
                    IsGodClass = false,
                    SuggestedExtractions = Array.Empty<ResponsibilityCluster>(),
                    GodMethods = godMethods,
                    AnalyzedAt = DateTime.UtcNow,
                    Summary = $"Class '{metrics.ClassName}' has {godMethods.Count} god method(s)."
                });
            }
        }

        // Store the first analysis result (or aggregate multiple if needed)
        return analysisResults.Any()
            ? nodeWithMetrics.WithAnalysisResult(analysisResults.First())
            : nodeWithMetrics;
    }

    private GodFileResult DetectGodFile(
        string filePath,
        IReadOnlyList<ClassMetrics> classMetrics,
        DetectionThresholds thresholds)
    {
        return GodFileResult.Create(filePath, classMetrics, thresholds);
    }

    private IReadOnlyList<GodMethodResult> DetectGodMethods(
        ClassMetrics classMetrics,
        DetectionThresholds thresholds)
    {
        var godMethods = new List<GodMethodResult>();

        foreach (var method in classMetrics.Methods)
        {
            var godMethod = GodMethodResult.Create(
                method,
                classMetrics.ClassName,
                classMetrics.FilePath,
                thresholds);

            if (godMethod.IsGodMethod)
                godMethods.Add(godMethod);
        }

        return godMethods.OrderByDescending(m => m.ViolationScore).ToList();
    }

    private Task<Result<AnalysisResult>> AnalyzeClassMetricsAsync(
        ClassMetrics metrics,
        DetectionThresholds thresholds,
        IReadOnlyList<GodMethodResult> godMethods,
        CancellationToken cancellationToken)
    {
        try
        {
            // Create a temporary file analysis result
            var result = new AnalysisResult
            {
                ClassMetrics = metrics,
                IsGodClass = true,
                SuggestedExtractions = Array.Empty<ResponsibilityCluster>(),
                GodMethods = godMethods,
                AnalyzedAt = DateTime.UtcNow,
                Summary = $"God class detected: {metrics.ClassName}" +
                         (godMethods.Any() ? $" with {godMethods.Count} god method(s)." : ".")
            };

            return Task.FromResult(Result<AnalysisResult>.Success(result));
        }
        catch (Exception ex)
        {
            return Task.FromResult(Result<AnalysisResult>.Failure(ex.Message));
        }
    }
}


using GodClassDetector.Core.Models;

namespace GodClassDetector.Core.Interfaces;

/// <summary>
/// Traverses filesystem AST in parallel, building sub-ASTs for classes and functions
/// </summary>
public interface IParallelASTTraverser
{
    Task<Result<FileSystemNode>> TraverseAndAnalyzeAsync(
        FileSystemNode root,
        DetectionThresholds thresholds,
        CancellationToken cancellationToken = default);
}


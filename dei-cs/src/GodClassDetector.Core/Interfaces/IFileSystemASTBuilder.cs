using GodClassDetector.Core.Models;

namespace GodClassDetector.Core.Interfaces;

/// <summary>
/// Builds an AST representation of the filesystem structure
/// </summary>
public interface IFileSystemASTBuilder
{
    Task<Result<FileSystemNode>> BuildASTAsync(
        string rootPath,
        CancellationToken cancellationToken = default);
}


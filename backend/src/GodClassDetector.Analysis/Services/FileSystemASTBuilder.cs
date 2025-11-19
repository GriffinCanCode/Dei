using GodClassDetector.Core.Interfaces;
using GodClassDetector.Core.Models;

namespace GodClassDetector.Analysis.Services;

/// <summary>
/// Builds an AST representation of the filesystem structure
/// </summary>
public sealed class FileSystemASTBuilder : IFileSystemASTBuilder
{
    private static readonly HashSet<string> ExcludedDirectories = new(StringComparer.OrdinalIgnoreCase)
    {
        "bin", "obj", ".git", ".vs", "node_modules", "packages", ".idea"
    };

    public async Task<Result<FileSystemNode>> BuildASTAsync(
        string rootPath,
        CancellationToken cancellationToken = default)
    {
        try
        {
            if (!Path.Exists(rootPath))
                return Result<FileSystemNode>.Failure($"Path not found: {rootPath}");

            var root = File.Exists(rootPath)
                ? await BuildFileNodeAsync(rootPath, 0, null, cancellationToken)
                : await BuildDirectoryNodeAsync(rootPath, 0, null, cancellationToken);

            return Result<FileSystemNode>.Success(root);
        }
        catch (Exception ex)
        {
            return Result<FileSystemNode>.Failure($"Error building AST: {ex.Message}");
        }
    }

    private async Task<FileSystemNode> BuildDirectoryNodeAsync(
        string path,
        int depth,
        FileSystemNode? parent,
        CancellationToken cancellationToken)
    {
        var node = FileSystemNode.CreateDirectory(path, depth, parent);
        
        cancellationToken.ThrowIfCancellationRequested();

        var entries = Directory.GetFileSystemEntries(path);
        var children = new List<FileSystemNode>();

        foreach (var entry in entries)
        {
            var name = Path.GetFileName(entry);
            
            if (Directory.Exists(entry))
            {
                if (!ShouldExcludeDirectory(name))
                {
                    var childNode = await BuildDirectoryNodeAsync(entry, depth + 1, node, cancellationToken);
                    children.Add(childNode);
                }
            }
            else if (File.Exists(entry) && name.EndsWith(".cs", StringComparison.OrdinalIgnoreCase))
            {
                var childNode = await BuildFileNodeAsync(entry, depth + 1, node, cancellationToken);
                children.Add(childNode);
            }
        }

        return node.WithChildren(children);
    }

    private Task<FileSystemNode> BuildFileNodeAsync(
        string path,
        int depth,
        FileSystemNode? parent,
        CancellationToken cancellationToken)
    {
        cancellationToken.ThrowIfCancellationRequested();
        return Task.FromResult(FileSystemNode.CreateFile(path, depth, parent));
    }

    private static bool ShouldExcludeDirectory(string name) =>
        ExcludedDirectories.Contains(name);
}


namespace GodClassDetector.Core.Models;

/// <summary>
/// Represents a node in the filesystem AST
/// </summary>
public sealed record FileSystemNode
{
    public required string Name { get; init; }
    public required string FullPath { get; init; }
    public required FileSystemNodeType Type { get; init; }
    public required IReadOnlyList<FileSystemNode> Children { get; init; }
    public FileSystemNode? Parent { get; init; }
    public int Depth { get; init; }
    
    // Analysis results populated during traversal
    public IReadOnlyList<ClassMetrics>? ClassMetrics { get; init; }
    public AnalysisResult? AnalysisResult { get; init; }
    public GodFileResult? GodFileResult { get; init; }

    public bool IsFile => Type == FileSystemNodeType.File;
    public bool IsDirectory => Type == FileSystemNodeType.Directory;
    public bool IsCSharpFile => IsFile && Name.EndsWith(".cs", StringComparison.OrdinalIgnoreCase);
    public bool HasIssues => AnalysisResult?.HasIssues == true || GodFileResult?.IsGodFile == true;

    public static FileSystemNode CreateDirectory(string path, int depth = 0, FileSystemNode? parent = null) =>
        new()
        {
            Name = Path.GetFileName(path) ?? path,
            FullPath = Path.GetFullPath(path),
            Type = FileSystemNodeType.Directory,
            Children = Array.Empty<FileSystemNode>(),
            Depth = depth,
            Parent = parent
        };

    public static FileSystemNode CreateFile(string path, int depth = 0, FileSystemNode? parent = null) =>
        new()
        {
            Name = Path.GetFileName(path),
            FullPath = Path.GetFullPath(path),
            Type = FileSystemNodeType.File,
            Children = Array.Empty<FileSystemNode>(),
            Depth = depth,
            Parent = parent
        };

    public FileSystemNode WithChildren(IReadOnlyList<FileSystemNode> children) =>
        this with { Children = children };

    public FileSystemNode WithClassMetrics(IReadOnlyList<ClassMetrics> metrics) =>
        this with { ClassMetrics = metrics };

    public FileSystemNode WithAnalysisResult(AnalysisResult result) =>
        this with { AnalysisResult = result };

    public FileSystemNode WithGodFileResult(GodFileResult result) =>
        this with { GodFileResult = result };
}

public enum FileSystemNodeType
{
    File,
    Directory
}


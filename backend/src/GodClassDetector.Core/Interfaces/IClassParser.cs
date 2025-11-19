using GodClassDetector.Core.Models;

namespace GodClassDetector.Core.Interfaces;

/// <summary>
/// Parses C# source files to extract class information
/// </summary>
public interface IClassParser
{
    Task<Result<IReadOnlyList<ClassMetrics>>> ParseFileAsync(string filePath, CancellationToken cancellationToken = default);
    Task<Result<IReadOnlyList<ClassMetrics>>> ParseDirectoryAsync(string directoryPath, CancellationToken cancellationToken = default);
}


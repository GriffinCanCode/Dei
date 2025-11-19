using GodClassDetector.Core.Models;

namespace GodClassDetector.Analysis.Reporting;

/// <summary>
/// Generates reports from FileSystemNode AST analysis results
/// </summary>
public sealed class ASTReportGenerator
{
    public ProjectAnalysisReport GenerateReport(FileSystemNode root)
    {
        var allResults = new List<AnalysisResult>();
        var fileNodes = new List<FileSystemNode>();
        var godFiles = new List<GodFileResult>();
        
        CollectResults(root, allResults, fileNodes, godFiles);

        var godClasses = allResults.Where(r => r.IsGodClass).ToList();
        var classesWithGodMethods = allResults.Where(r => !r.IsGodClass && r.GodMethods.Any()).ToList();
        var healthyClasses = allResults.Where(r => !r.HasIssues).ToList();
        var allGodMethods = allResults.SelectMany(r => r.GodMethods).ToList();

        return new ProjectAnalysisReport
        {
            RootPath = root.FullPath,
            TotalFiles = fileNodes.Count,
            TotalClasses = allResults.Count,
            GodClasses = godClasses,
            ClassesWithGodMethods = classesWithGodMethods,
            HealthyClasses = healthyClasses,
            TotalGodMethods = allGodMethods.Count,
            GodFiles = godFiles.Where(f => f.IsGodFile).ToList(),
            FileSystemAST = root,
            AnalyzedAt = DateTime.UtcNow
        };
    }

    private void CollectResults(
        FileSystemNode node,
        List<AnalysisResult> results,
        List<FileSystemNode> files,
        List<GodFileResult> godFiles)
    {
        if (node.IsFile && node.IsCSharpFile)
        {
            files.Add(node);
            
            if (node.AnalysisResult != null)
                results.Add(node.AnalysisResult);

            if (node.GodFileResult != null)
                godFiles.Add(node.GodFileResult);
        }

        foreach (var child in node.Children)
            CollectResults(child, results, files, godFiles);
    }

    public string GenerateConsoleReport(ProjectAnalysisReport report)
    {
        var lines = new List<string>
        {
            "╔════════════════════════════════════════════════════════════╗",
            "║           DEI CHECK - PROJECT ANALYSIS REPORT              ║",
            "╚════════════════════════════════════════════════════════════╝",
            "",
            $"Root Path: {report.RootPath}",
            $"Analyzed: {report.AnalyzedAt:yyyy-MM-dd HH:mm:ss} UTC",
            "",
            "╭────────────────────────────┬───────╮",
            $"│ Total Files Analyzed       │ {report.TotalFiles,5} │",
            $"│ Total Classes              │ {report.TotalClasses,5} │",
            $"│ God Files Detected         │ {report.GodFiles.Count,5} │",
            $"│ God Classes Detected       │ {report.GodClasses.Count,5} │",
            $"│ God Methods Detected       │ {report.TotalGodMethods,5} │",
            $"│ Classes with God Methods   │ {report.ClassesWithGodMethods.Count,5} │",
            $"│ Healthy Classes            │ {report.HealthyClasses.Count,5} │",
            "╰────────────────────────────┴───────╯",
            ""
        };

        var hasIssues = report.GodFiles.Any() || report.GodClasses.Any() || report.TotalGodMethods > 0;

        if (report.GodFiles.Any())
        {
            lines.Add("⚠️  GOD FILES DETECTED:");
            lines.Add("");

            foreach (var godFile in report.GodFiles)
            {
                lines.Add($"  📄 {Path.GetFileName(godFile.FilePath)}");
                lines.Add($"     File: {godFile.FilePath}");
                lines.Add($"     Classes: {godFile.ClassCount}, Total Lines: {godFile.TotalLines}");
                lines.Add($"     Classes in file: {string.Join(", ", godFile.ClassNames)}");
                
                foreach (var violation in godFile.Violations)
                    lines.Add($"     • {violation}");
                
                lines.Add("");
            }
        }

        if (report.GodClasses.Any())
        {
            lines.Add("⚠️  GOD CLASSES DETECTED:");
            lines.Add("");

            foreach (var godClass in report.GodClasses)
            {
                lines.Add($"  ❌ {godClass.ClassMetrics.ClassName}");
                lines.Add($"     File: {godClass.ClassMetrics.FilePath}");
                lines.Add($"     Lines: {godClass.ClassMetrics.LineCount}, " +
                         $"Methods: {godClass.ClassMetrics.MethodCount}, " +
                         $"Complexity: {godClass.ClassMetrics.CyclomaticComplexity}");
                
                if (godClass.SuggestedExtractions.Any())
                    lines.Add($"     Suggested Extractions: {godClass.SuggestedExtractions.Count}");
                
                if (godClass.GodMethods.Any())
                    lines.Add($"     God Methods: {godClass.GodMethods.Count}");
                
                lines.Add("");
            }
        }

        if (report.ClassesWithGodMethods.Any())
        {
            lines.Add("⚠️  GOD METHODS DETECTED:");
            lines.Add("");

            foreach (var classResult in report.ClassesWithGodMethods)
            {
                lines.Add($"  📝 {classResult.ClassMetrics.ClassName}");
                lines.Add($"     File: {classResult.ClassMetrics.FilePath}");
                lines.Add($"     God Methods: {classResult.GodMethods.Count}");
                lines.Add("");

                foreach (var godMethod in classResult.GodMethods)
                {
                    lines.Add($"     ⚠️  {godMethod.Method.MethodName}()");
                    lines.Add($"        Lines: {godMethod.Method.LineCount}, " +
                             $"Complexity: {godMethod.Method.CyclomaticComplexity}, " +
                             $"Parameters: {godMethod.Method.Parameters.Count}");
                    
                    foreach (var violation in godMethod.Violations)
                        lines.Add($"        • {violation}");
                    
                    lines.Add("");
                }
            }
        }

        if (!hasIssues)
        {
            lines.Add("✅ No god files, classes, or methods detected! Your codebase is healthy.");
        }

        return string.Join(Environment.NewLine, lines);
    }

    public string GenerateTreeView(FileSystemNode root, bool showOnlyProblems = false)
    {
        var lines = new List<string>
        {
            "PROJECT STRUCTURE:",
            ""
        };

        BuildTreeView(root, "", true, lines, showOnlyProblems);
        return string.Join(Environment.NewLine, lines);
    }

    private void BuildTreeView(
        FileSystemNode node,
        string indent,
        bool isLast,
        List<string> lines,
        bool showOnlyProblems)
    {
        if (showOnlyProblems && node.IsFile && (node.AnalysisResult == null || !node.AnalysisResult.IsGodClass))
            return;

        var prefix = indent + (isLast ? "└── " : "├── ");
        var status = GetNodeStatus(node);
        lines.Add($"{prefix}{node.Name}{status}");

        if (node.IsDirectory)
        {
            var children = showOnlyProblems
                ? node.Children.Where(c => HasProblems(c)).ToList()
                : node.Children.ToList();

            for (var i = 0; i < children.Count; i++)
            {
                var child = children[i];
                var childIndent = indent + (isLast ? "    " : "│   ");
                BuildTreeView(child, childIndent, i == children.Count - 1, lines, showOnlyProblems);
            }
        }
    }

    private bool HasProblems(FileSystemNode node)
    {
        if (node.IsFile && node.HasIssues)
            return true;

        return node.Children.Any(HasProblems);
    }

    private string GetNodeStatus(FileSystemNode node)
    {
        if (!node.IsFile)
            return "";

        if (node.GodFileResult?.IsGodFile == true)
            return " 📄 [GOD FILE]";

        if (node.AnalysisResult == null)
            return "";

        if (node.AnalysisResult.IsGodClass)
            return " ❌ [GOD CLASS]";
        
        if (node.AnalysisResult.GodMethods.Any())
            return $" ⚠️ [{node.AnalysisResult.GodMethods.Count} GOD METHOD(S)]";

        return " ✅";
    }
}

public sealed record ProjectAnalysisReport
{
    public required string RootPath { get; init; }
    public required int TotalFiles { get; init; }
    public required int TotalClasses { get; init; }
    public required IReadOnlyList<GodFileResult> GodFiles { get; init; }
    public required IReadOnlyList<AnalysisResult> GodClasses { get; init; }
    public required IReadOnlyList<AnalysisResult> ClassesWithGodMethods { get; init; }
    public required IReadOnlyList<AnalysisResult> HealthyClasses { get; init; }
    public required int TotalGodMethods { get; init; }
    public required FileSystemNode FileSystemAST { get; init; }
    public required DateTime AnalyzedAt { get; init; }
}


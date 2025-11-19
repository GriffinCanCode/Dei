using System.Text;
using System.Text.Json;
using GodClassDetector.Core.Interfaces;
using GodClassDetector.Core.Models;

namespace GodClassDetector.Analysis.Reporting;

/// <summary>
/// Generates various report formats from analysis results
/// </summary>
public sealed class ReportGenerator : IReportGenerator
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    public Task<string> GenerateConsoleReportAsync(IReadOnlyList<AnalysisResult> results)
    {
        var sb = new StringBuilder();
        sb.AppendLine("╔════════════════════════════════════════════════════════════════╗");
        sb.AppendLine("║            GOD CLASS DETECTION REPORT                          ║");
        sb.AppendLine("╚════════════════════════════════════════════════════════════════╝");
        sb.AppendLine();

        var godClasses = results.Where(r => r.IsGodClass).ToList();
        
        sb.AppendLine($"📊 Summary:");
        sb.AppendLine($"   Total Classes Analyzed: {results.Count}");
        sb.AppendLine($"   God Classes Detected:   {godClasses.Count}");
        sb.AppendLine($"   Healthy Classes:        {results.Count - godClasses.Count}");
        sb.AppendLine();

        if (!godClasses.Any())
        {
            sb.AppendLine("✅ No god classes detected! Your code is well-structured.");
            return Task.FromResult(sb.ToString());
        }

        sb.AppendLine("⚠️  God Classes Found:");
        sb.AppendLine();

        foreach (var result in godClasses)
        {
            AppendClassDetails(sb, result);
        }

        return Task.FromResult(sb.ToString());
    }

    private static void AppendClassDetails(StringBuilder sb, AnalysisResult result)
    {
        var metrics = result.ClassMetrics;
        
        sb.AppendLine($"┌─ {metrics.ClassName}");
        sb.AppendLine($"│  File: {metrics.FilePath}");
        sb.AppendLine($"│  Metrics:");
        sb.AppendLine($"│    • Lines:       {metrics.LineCount}");
        sb.AppendLine($"│    • Methods:     {metrics.MethodCount}");
        sb.AppendLine($"│    • Complexity:  {metrics.CyclomaticComplexity}");
        sb.AppendLine($"│");

        if (result.SuggestedExtractions.Any())
        {
            sb.AppendLine($"│  💡 Suggested Extractions ({result.SuggestedExtractions.Count}):");
            
            foreach (var cluster in result.SuggestedExtractions)
            {
                sb.AppendLine($"│");
                sb.AppendLine($"│    → {cluster.SuggestedClassName}");
                sb.AppendLine($"│      Cohesion Score: {cluster.CohesionScore:F2}");
                sb.AppendLine($"│      Methods ({cluster.Methods.Count}):");
                
                foreach (var method in cluster.Methods.Take(5))
                {
                    sb.AppendLine($"│        • {method.MethodName}");
                }
                
                if (cluster.Methods.Count > 5)
                    sb.AppendLine($"│        • ... and {cluster.Methods.Count - 5} more");
                
                sb.AppendLine($"│      Reason: {cluster.Justification}");
            }
        }
        
        sb.AppendLine("└" + new string('─', 60));
        sb.AppendLine();
    }

    public Task<string> GenerateJsonReportAsync(IReadOnlyList<AnalysisResult> results)
    {
        var report = new
        {
            GeneratedAt = DateTime.UtcNow,
            Summary = new
            {
                TotalClasses = results.Count,
                GodClassesDetected = results.Count(r => r.IsGodClass),
                HealthyClasses = results.Count(r => !r.IsGodClass)
            },
            Results = results.Select(r => new
            {
                ClassName = r.ClassMetrics.ClassName,
                FilePath = r.ClassMetrics.FilePath,
                IsGodClass = r.IsGodClass,
                Metrics = new
                {
                    r.ClassMetrics.LineCount,
                    r.ClassMetrics.MethodCount,
                    r.ClassMetrics.CyclomaticComplexity
                },
                SuggestedExtractions = r.SuggestedExtractions.Select(c => new
                {
                    c.SuggestedClassName,
                    c.CohesionScore,
                    MethodCount = c.Methods.Count,
                    Methods = c.Methods.Select(m => m.MethodName),
                    c.Justification
                })
            })
        };

        var json = JsonSerializer.Serialize(report, JsonOptions);
        return Task.FromResult(json);
    }

    public Task<string> GenerateMarkdownReportAsync(IReadOnlyList<AnalysisResult> results)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# God Class Detection Report");
        sb.AppendLine();
        sb.AppendLine($"**Generated:** {DateTime.UtcNow:yyyy-MM-dd HH:mm:ss} UTC");
        sb.AppendLine();

        var godClasses = results.Where(r => r.IsGodClass).ToList();
        
        sb.AppendLine("## Summary");
        sb.AppendLine();
        sb.AppendLine($"- **Total Classes Analyzed:** {results.Count}");
        sb.AppendLine($"- **God Classes Detected:** {godClasses.Count}");
        sb.AppendLine($"- **Healthy Classes:** {results.Count - godClasses.Count}");
        sb.AppendLine();

        if (!godClasses.Any())
        {
            sb.AppendLine("✅ **No god classes detected!** Your code is well-structured.");
            return Task.FromResult(sb.ToString());
        }

        sb.AppendLine("## Detected God Classes");
        sb.AppendLine();

        foreach (var result in godClasses)
        {
            AppendMarkdownClassDetails(sb, result);
        }

        return Task.FromResult(sb.ToString());
    }

    private static void AppendMarkdownClassDetails(StringBuilder sb, AnalysisResult result)
    {
        var metrics = result.ClassMetrics;
        
        sb.AppendLine($"### {metrics.ClassName}");
        sb.AppendLine();
        sb.AppendLine($"**File:** `{metrics.FilePath}`");
        sb.AppendLine();
        sb.AppendLine("#### Metrics");
        sb.AppendLine();
        sb.AppendLine("| Metric | Value |");
        sb.AppendLine("|--------|-------|");
        sb.AppendLine($"| Lines | {metrics.LineCount} |");
        sb.AppendLine($"| Methods | {metrics.MethodCount} |");
        sb.AppendLine($"| Cyclomatic Complexity | {metrics.CyclomaticComplexity} |");
        sb.AppendLine();

        if (result.SuggestedExtractions.Any())
        {
            sb.AppendLine("#### Suggested Refactorings");
            sb.AppendLine();

            foreach (var cluster in result.SuggestedExtractions)
            {
                sb.AppendLine($"##### Extract → `{cluster.SuggestedClassName}`");
                sb.AppendLine();
                sb.AppendLine($"**Cohesion Score:** {cluster.CohesionScore:F2}");
                sb.AppendLine();
                sb.AppendLine("**Methods to Extract:**");
                sb.AppendLine();
                
                foreach (var method in cluster.Methods)
                {
                    sb.AppendLine($"- `{method.MethodName}`");
                }
                
                sb.AppendLine();
                sb.AppendLine($"**Justification:** {cluster.Justification}");
                sb.AppendLine();
            }
        }

        sb.AppendLine("---");
        sb.AppendLine();
    }
}


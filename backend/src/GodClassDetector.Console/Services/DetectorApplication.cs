using Microsoft.Extensions.Options;
using Spectre.Console;
using GodClassDetector.Core.Interfaces;
using GodClassDetector.Core.Models;
using GodClassDetector.Console.Configuration;

namespace GodClassDetector.Console.Services;

/// <summary>
/// Main application service that orchestrates the detection process
/// </summary>
public sealed class DetectorApplication
{
    private readonly IGodClassDetector _detector;
    private readonly IReportGenerator _reportGenerator;
    private readonly DetectionOptions _options;

    public DetectorApplication(
        IGodClassDetector detector,
        IReportGenerator reportGenerator,
        IOptions<DetectionOptions> options)
    {
        _detector = detector ?? throw new ArgumentNullException(nameof(detector));
        _reportGenerator = reportGenerator ?? throw new ArgumentNullException(nameof(reportGenerator));
        _options = options?.Value ?? throw new ArgumentNullException(nameof(options));
    }

    public async Task<int> RunAsync(string[] args)
    {
        DisplayBanner();

        if (args.Length == 0)
        {
            DisplayUsage();
            return 1;
        }

        var targetPath = args[0];
        var thresholds = _options.ToThresholds();

        return await AnalyzeTargetAsync(targetPath, thresholds);
    }

    private async Task<int> AnalyzeTargetAsync(string targetPath, DetectionThresholds thresholds)
    {
        if (!Path.Exists(targetPath))
        {
            AnsiConsole.MarkupLine($"[red]Error:[/] Path not found: {targetPath}");
            return 1;
        }

        var isDirectory = Directory.Exists(targetPath);
        
        return await AnsiConsole.Status()
            .StartAsync("Analyzing...", async ctx =>
            {
                ctx.Spinner(Spinner.Known.Dots);
                
                var result = isDirectory
                    ? await _detector.AnalyzeProjectAsync(targetPath, thresholds)
                    : await AnalyzeSingleFileAsync(targetPath, thresholds);

                return result.Match(
                    onSuccess: results => DisplayResults(results),
                    onFailure: error =>
                    {
                        AnsiConsole.MarkupLine($"[red]Error:[/] {error}");
                        return 1;
                    });
            });
    }

    private async Task<Result<IReadOnlyList<AnalysisResult>>> AnalyzeSingleFileAsync(
        string filePath,
        DetectionThresholds thresholds)
    {
        var result = await _detector.AnalyzeClassAsync(filePath, thresholds);
        return result.IsSuccess
            ? Result<IReadOnlyList<AnalysisResult>>.Success(new[] { result.Value })
            : Result<IReadOnlyList<AnalysisResult>>.Failure(result.Error);
    }

    private int DisplayResults(IReadOnlyList<AnalysisResult> results)
    {
        if (!results.Any())
        {
            AnsiConsole.MarkupLine("[yellow]No classes found to analyze.[/]");
            return 0;
        }

        var godClasses = results.Where(r => r.IsGodClass).ToList();

        // Display summary table
        DisplaySummaryTable(results, godClasses);

        // Display detailed results for god classes
        if (godClasses.Any())
        {
            AnsiConsole.WriteLine();
            DisplayDetailedResults(godClasses);
        }

        return godClasses.Any() ? 1 : 0;
    }

    private static void DisplaySummaryTable(
        IReadOnlyList<AnalysisResult> allResults,
        IReadOnlyList<AnalysisResult> godClasses)
    {
        var table = new Table()
            .Border(TableBorder.Rounded)
            .AddColumn(new TableColumn("[bold]Metric[/]").Centered())
            .AddColumn(new TableColumn("[bold]Value[/]").Centered());

        table.AddRow("Total Classes Analyzed", allResults.Count.ToString());
        table.AddRow(
            "[red]God Classes Detected[/]",
            $"[red bold]{godClasses.Count}[/]");
        table.AddRow(
            "[green]Healthy Classes[/]",
            $"[green bold]{allResults.Count - godClasses.Count}[/]");

        AnsiConsole.Write(table);
    }

    private static void DisplayDetailedResults(IReadOnlyList<AnalysisResult> godClasses)
    {
        AnsiConsole.MarkupLine("[bold red]⚠️  God Classes Detected:[/]");
        AnsiConsole.WriteLine();

        foreach (var result in godClasses)
        {
            DisplayClassPanel(result);
            AnsiConsole.WriteLine();
        }
    }

    private static void DisplayClassPanel(AnalysisResult result)
    {
        var metrics = result.ClassMetrics;
        
        var panel = new Panel(BuildClassContent(result))
            .Header($"[bold yellow]{metrics.ClassName}[/]")
            .Border(BoxBorder.Double)
            .BorderColor(Color.Red);

        AnsiConsole.Write(panel);
    }

    private static string BuildClassContent(AnalysisResult result)
    {
        var metrics = result.ClassMetrics;
        var content = new List<string>
        {
            $"[dim]File:[/] {metrics.FilePath}",
            "",
            "[bold]Metrics:[/]",
            $"  • Lines:      [red]{metrics.LineCount}[/]",
            $"  • Methods:    [red]{metrics.MethodCount}[/]",
            $"  • Complexity: [red]{metrics.CyclomaticComplexity}[/]",
            ""
        };

        if (result.SuggestedExtractions.Any())
        {
            content.Add($"[bold green]💡 Suggested Refactorings ({result.SuggestedExtractions.Count}):[/]");
            content.Add("");

            foreach (var cluster in result.SuggestedExtractions)
            {
                content.Add($"  [bold cyan]→ {cluster.SuggestedClassName}[/]");
                content.Add($"    Cohesion Score: {cluster.CohesionScore:F2}");
                content.Add($"    Methods ({cluster.Methods.Count}):");
                
                foreach (var method in cluster.Methods.Take(5))
                {
                    content.Add($"      • {method.MethodName}");
                }
                
                if (cluster.Methods.Count > 5)
                    content.Add($"      • [dim]... and {cluster.Methods.Count - 5} more[/]");
                
                content.Add($"    [dim]{cluster.Justification}[/]");
                content.Add("");
            }
        }

        return string.Join(Environment.NewLine, content);
    }

    private static void DisplayBanner()
    {
        AnsiConsole.Write(
            new FigletText("God Class Detector")
                .LeftJustified()
                .Color(Color.Cyan1));
        
        AnsiConsole.MarkupLine("[dim]A tool for identifying and refactoring god classes in C# projects[/]");
        AnsiConsole.WriteLine();
    }

    private static void DisplayUsage()
    {
        AnsiConsole.MarkupLine("[bold]Usage:[/]");
        AnsiConsole.MarkupLine("  dotnet run <path-to-file-or-directory>");
        AnsiConsole.WriteLine();
        AnsiConsole.MarkupLine("[bold]Examples:[/]");
        AnsiConsole.MarkupLine("  dotnet run ./MyClass.cs");
        AnsiConsole.MarkupLine("  dotnet run ./src/MyProject");
    }
}


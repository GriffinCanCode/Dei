using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using GodClassDetector.Analysis.Metrics;
using GodClassDetector.Analysis.Parsers;
using GodClassDetector.Analysis.Reporting;
using GodClassDetector.Analysis.Services;
using GodClassDetector.Clustering.Analyzers;
using GodClassDetector.Console.Configuration;
using GodClassDetector.Console.Services;
using GodClassDetector.Core.Interfaces;

var builder = Host.CreateDefaultBuilder(args)
    .ConfigureAppConfiguration((context, config) =>
    {
        config.SetBasePath(Directory.GetCurrentDirectory())
            .AddJsonFile("appsettings.json", optional: false, reloadOnChange: true)
            .AddEnvironmentVariables()
            .AddCommandLine(args);
    })
    .ConfigureServices((context, services) =>
    {
        // Configuration
        services.Configure<DetectionOptions>(
            context.Configuration.GetSection(DetectionOptions.SectionName));

        // Core Services
        services.AddSingleton<IMetricsCalculator, ComplexityCalculator>();
        services.AddSingleton<IClassParser, RoslynClassParser>();
        services.AddSingleton<ISemanticAnalyzer, SemanticClusteringAnalyzer>();
        services.AddSingleton<IGodClassDetector, GodClassDetectorService>();
        services.AddSingleton<IReportGenerator, ReportGenerator>();

        // Application
        services.AddSingleton<DetectorApplication>();
    });

var host = builder.Build();

// Run the application
var app = host.Services.GetRequiredService<DetectorApplication>();
var exitCode = await app.RunAsync(args);

Environment.Exit(exitCode);


using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using GodClassDetector.Core.Interfaces;
using GodClassDetector.Core.Models;

namespace GodClassDetector.Analysis.Parsers;

/// <summary>
/// Parses C# source files using Roslyn to extract class metrics
/// </summary>
public sealed class RoslynClassParser : IClassParser
{
    private readonly IMetricsCalculator _metricsCalculator;

    public RoslynClassParser(IMetricsCalculator metricsCalculator)
    {
        _metricsCalculator = metricsCalculator ?? throw new ArgumentNullException(nameof(metricsCalculator));
    }

    public async Task<Result<IReadOnlyList<ClassMetrics>>> ParseFileAsync(
        string filePath,
        CancellationToken cancellationToken = default)
    {
        try
        {
            if (!File.Exists(filePath))
                return Result<IReadOnlyList<ClassMetrics>>.Failure($"File not found: {filePath}");

            var sourceCode = await File.ReadAllTextAsync(filePath, cancellationToken);
            var tree = CSharpSyntaxTree.ParseText(sourceCode, cancellationToken: cancellationToken);
            var root = await tree.GetRootAsync(cancellationToken);

            var classes = root.DescendantNodes()
                .OfType<ClassDeclarationSyntax>()
                .Select(cls => ParseClass(cls, filePath))
                .ToList();

            return Result<IReadOnlyList<ClassMetrics>>.Success(classes);
        }
        catch (Exception ex)
        {
            return Result<IReadOnlyList<ClassMetrics>>.Failure($"Error parsing file {filePath}: {ex.Message}");
        }
    }

    public async Task<Result<IReadOnlyList<ClassMetrics>>> ParseDirectoryAsync(
        string directoryPath,
        CancellationToken cancellationToken = default)
    {
        try
        {
            if (!Directory.Exists(directoryPath))
                return Result<IReadOnlyList<ClassMetrics>>.Failure($"Directory not found: {directoryPath}");

            var csFiles = Directory.GetFiles(directoryPath, "*.cs", SearchOption.AllDirectories);
            var allClasses = new List<ClassMetrics>();

            foreach (var file in csFiles)
            {
                var result = await ParseFileAsync(file, cancellationToken);
                if (result.IsSuccess)
                    allClasses.AddRange(result.Value);
            }

            return Result<IReadOnlyList<ClassMetrics>>.Success(allClasses);
        }
        catch (Exception ex)
        {
            return Result<IReadOnlyList<ClassMetrics>>.Failure($"Error parsing directory {directoryPath}: {ex.Message}");
        }
    }

    private ClassMetrics ParseClass(ClassDeclarationSyntax classDecl, string filePath)
    {
        var methods = classDecl.Members
            .OfType<MethodDeclarationSyntax>()
            .Select(ParseMethod)
            .ToList();

        var properties = classDecl.Members.OfType<PropertyDeclarationSyntax>().Count();
        var fields = classDecl.Members.OfType<FieldDeclarationSyntax>().Count();

        var sourceText = classDecl.GetText().ToString();
        var lineCount = _metricsCalculator.CalculateLineCount(sourceText);
        var dependencies = _metricsCalculator.ExtractDependencies(sourceText);

        var totalComplexity = methods.Sum(m => m.CyclomaticComplexity);

        var namespaceName = classDecl.Ancestors()
            .OfType<NamespaceDeclarationSyntax>()
            .FirstOrDefault()?.Name.ToString() ?? string.Empty;

        var fullyQualifiedName = string.IsNullOrEmpty(namespaceName)
            ? classDecl.Identifier.Text
            : $"{namespaceName}.{classDecl.Identifier.Text}";

        return new ClassMetrics
        {
            ClassName = classDecl.Identifier.Text,
            FullyQualifiedName = fullyQualifiedName,
            FilePath = filePath,
            LineCount = lineCount,
            MethodCount = methods.Count,
            PropertyCount = properties,
            FieldCount = fields,
            CyclomaticComplexity = totalComplexity,
            Methods = methods,
            Dependencies = dependencies
        };
    }

    private MethodMetrics ParseMethod(MethodDeclarationSyntax methodDecl)
    {
        var body = methodDecl.Body?.ToString() ?? string.Empty;
        var lineCount = _metricsCalculator.CalculateLineCount(body);
        var complexity = _metricsCalculator.CalculateCyclomaticComplexity(body);

        var calledMethods = methodDecl.DescendantNodes()
            .OfType<InvocationExpressionSyntax>()
            .Select(inv => inv.Expression.ToString())
            .Distinct()
            .ToList();

        var accessedFields = methodDecl.DescendantNodes()
            .OfType<IdentifierNameSyntax>()
            .Select(id => id.Identifier.Text)
            .Distinct()
            .ToList();

        var parameters = methodDecl.ParameterList.Parameters
            .Select(p => $"{p.Type} {p.Identifier}")
            .ToList();

        var tokens = ExtractTokens(methodDecl);

        return new MethodMetrics
        {
            MethodName = methodDecl.Identifier.Text,
            LineCount = lineCount,
            CyclomaticComplexity = complexity,
            CalledMethods = calledMethods,
            AccessedFields = accessedFields,
            Parameters = parameters,
            ReturnType = methodDecl.ReturnType.ToString(),
            IsPublic = methodDecl.Modifiers.Any(m => m.IsKind(SyntaxKind.PublicKeyword)),
            IsStatic = methodDecl.Modifiers.Any(m => m.IsKind(SyntaxKind.StaticKeyword)),
            Tokens = tokens
        };
    }

    private static IReadOnlyList<string> ExtractTokens(MethodDeclarationSyntax methodDecl)
    {
        // Extract meaningful tokens for semantic analysis
        var tokens = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        // Add method name parts (split by camelCase/PascalCase)
        tokens.UnionWith(SplitIdentifier(methodDecl.Identifier.Text));

        // Add parameter type names
        foreach (var param in methodDecl.ParameterList.Parameters)
        {
            if (param.Type != null)
                tokens.UnionWith(SplitIdentifier(param.Type.ToString()));
        }

        // Add invoked method names
        var invocations = methodDecl.DescendantNodes()
            .OfType<InvocationExpressionSyntax>()
            .Select(inv => inv.Expression.ToString());

        foreach (var invocation in invocations)
            tokens.UnionWith(SplitIdentifier(invocation));

        return tokens.ToList();
    }

    private static IEnumerable<string> SplitIdentifier(string identifier)
    {
        // Split camelCase and PascalCase identifiers
        return System.Text.RegularExpressions.Regex
            .Split(identifier, @"(?<!^)(?=[A-Z])|[^\w]")
            .Where(s => !string.IsNullOrWhiteSpace(s) && s.Length > 2)
            .Select(s => s.ToLowerInvariant());
    }
}


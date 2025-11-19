using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using GodClassDetector.Core.Interfaces;

namespace GodClassDetector.Analysis.Metrics;

/// <summary>
/// Calculates various complexity and code metrics
/// </summary>
public sealed class ComplexityCalculator : IMetricsCalculator
{
    public int CalculateCyclomaticComplexity(string methodBody)
    {
        if (string.IsNullOrWhiteSpace(methodBody))
            return 1;

        var tree = CSharpSyntaxTree.ParseText(methodBody);
        var root = tree.GetRoot();

        // Start with base complexity of 1
        var complexity = 1;

        // Add complexity for each decision point
        var decisionNodes = root.DescendantNodes().Where(node =>
            node is IfStatementSyntax ||
            node is WhileStatementSyntax ||
            node is ForStatementSyntax ||
            node is ForEachStatementSyntax ||
            node is CaseSwitchLabelSyntax ||
            node is CatchClauseSyntax ||
            node is ConditionalExpressionSyntax ||
            node is BinaryExpressionSyntax binary && 
                (binary.IsKind(SyntaxKind.LogicalAndExpression) || 
                 binary.IsKind(SyntaxKind.LogicalOrExpression))
        );

        complexity += decisionNodes.Count();

        return complexity;
    }

    public int CalculateLineCount(string source)
    {
        if (string.IsNullOrWhiteSpace(source))
            return 0;

        return source
            .Split('\n')
            .Select(line => line.Trim())
            .Count(line => !string.IsNullOrWhiteSpace(line) && !line.StartsWith("//"));
    }

    public IReadOnlyList<string> ExtractDependencies(string source)
    {
        if (string.IsNullOrWhiteSpace(source))
            return Array.Empty<string>();

        var tree = CSharpSyntaxTree.ParseText(source);
        var root = tree.GetRoot();

        var dependencies = new HashSet<string>();

        // Extract using directives
        var usingDirectives = root.DescendantNodes()
            .OfType<UsingDirectiveSyntax>()
            .Select(u => u.Name?.ToString())
            .Where(n => n != null);

        foreach (var ns in usingDirectives)
            dependencies.Add(ns!);

        // Extract type references
        var typeReferences = root.DescendantNodes()
            .OfType<IdentifierNameSyntax>()
            .Select(id => id.Identifier.Text)
            .Where(name => char.IsUpper(name[0])); // Types typically start with uppercase

        foreach (var type in typeReferences)
            dependencies.Add(type);

        return dependencies.Take(50).ToList(); // Limit to avoid noise
    }
}


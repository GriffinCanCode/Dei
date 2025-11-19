using FluentAssertions;
using GodClassDetector.Analysis.Metrics;
using Xunit;

namespace GodClassDetector.Tests.Metrics;

public sealed class ComplexityCalculatorTests
{
    private readonly ComplexityCalculator _calculator = new();

    [Fact]
    public void CalculateCyclomaticComplexity_SimpleMethod_ReturnsOne()
    {
        // Arrange
        const string methodBody = @"
            public void SimpleMethod()
            {
                Console.WriteLine(""Hello"");
            }";

        // Act
        var complexity = _calculator.CalculateCyclomaticComplexity(methodBody);

        // Assert
        complexity.Should().Be(1);
    }

    [Fact]
    public void CalculateCyclomaticComplexity_WithIfStatement_ReturnsTwo()
    {
        // Arrange
        const string methodBody = @"
            public void MethodWithIf(bool condition)
            {
                if (condition)
                    Console.WriteLine(""True"");
            }";

        // Act
        var complexity = _calculator.CalculateCyclomaticComplexity(methodBody);

        // Assert
        complexity.Should().Be(2);
    }

    [Fact]
    public void CalculateCyclomaticComplexity_WithMultipleDecisionPoints_ReturnsCorrectValue()
    {
        // Arrange
        const string methodBody = @"
            public void ComplexMethod(bool a, bool b)
            {
                if (a && b)
                    DoSomething();
                
                for (int i = 0; i < 10; i++)
                    DoSomethingElse();
                
                while (condition)
                    DoMore();
            }";

        // Act
        var complexity = _calculator.CalculateCyclomaticComplexity(methodBody);

        // Assert
        complexity.Should().BeGreaterThan(3);
    }

    [Fact]
    public void CalculateLineCount_IgnoresEmptyLines()
    {
        // Arrange
        const string source = @"
            public void Method()
            {

                Console.WriteLine(""Line 1"");

                Console.WriteLine(""Line 2"");

            }";

        // Act
        var lineCount = _calculator.CalculateLineCount(source);

        // Assert
        lineCount.Should().Be(5); // public void, opening brace, two WriteLine, closing brace
    }

    [Fact]
    public void CalculateLineCount_IgnoresCommentLines()
    {
        // Arrange
        const string source = @"
            // This is a comment
            public void Method()
            {
                Console.WriteLine(""Hello"");
                // Another comment
            }";

        // Act
        var lineCount = _calculator.CalculateLineCount(source);

        // Assert
        lineCount.Should().Be(4); // public void, opening brace, WriteLine, closing brace (excludes comment lines)
    }

    [Fact]
    public void ExtractDependencies_FindsUsingDirectives()
    {
        // Arrange
        const string source = @"
            using System;
            using System.Collections.Generic;
            using MyNamespace;
            
            public class MyClass { }";

        // Act
        var dependencies = _calculator.ExtractDependencies(source);

        // Assert
        dependencies.Should().Contain("System");
        dependencies.Should().Contain("System.Collections.Generic");
        dependencies.Should().Contain("MyNamespace");
    }
}


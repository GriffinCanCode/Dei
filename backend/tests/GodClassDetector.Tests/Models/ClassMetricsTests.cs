using FluentAssertions;
using GodClassDetector.Core.Models;
using Xunit;

namespace GodClassDetector.Tests.Models;

public sealed class ClassMetricsTests
{
    [Fact]
    public void IsGodClass_WhenExceedsLineThreshold_ReturnsTrue()
    {
        // Arrange
        var metrics = CreateMetrics(lineCount: 500);
        var thresholds = new DetectionThresholds { MaxLines = 300 };

        // Act
        var result = metrics.IsGodClass(thresholds);

        // Assert
        result.Should().BeTrue();
    }

    [Fact]
    public void IsGodClass_WhenExceedsMethodThreshold_ReturnsTrue()
    {
        // Arrange
        var metrics = CreateMetrics(methodCount: 25);
        var thresholds = new DetectionThresholds { MaxMethods = 20 };

        // Act
        var result = metrics.IsGodClass(thresholds);

        // Assert
        result.Should().BeTrue();
    }

    [Fact]
    public void IsGodClass_WhenExceedsComplexityThreshold_ReturnsTrue()
    {
        // Arrange
        var metrics = CreateMetrics(complexity: 75);
        var thresholds = new DetectionThresholds { MaxComplexity = 50 };

        // Act
        var result = metrics.IsGodClass(thresholds);

        // Assert
        result.Should().BeTrue();
    }

    [Fact]
    public void IsGodClass_WhenWithinAllThresholds_ReturnsFalse()
    {
        // Arrange
        var metrics = CreateMetrics(lineCount: 200, methodCount: 15, complexity: 30);
        var thresholds = new DetectionThresholds
        {
            MaxLines = 300,
            MaxMethods = 20,
            MaxComplexity = 50
        };

        // Act
        var result = metrics.IsGodClass(thresholds);

        // Assert
        result.Should().BeFalse();
    }

    private static ClassMetrics CreateMetrics(
        int lineCount = 100,
        int methodCount = 10,
        int complexity = 20)
    {
        return new ClassMetrics
        {
            ClassName = "TestClass",
            FullyQualifiedName = "Test.TestClass",
            FilePath = "/test/TestClass.cs",
            LineCount = lineCount,
            MethodCount = methodCount,
            PropertyCount = 5,
            FieldCount = 3,
            CyclomaticComplexity = complexity,
            Methods = Array.Empty<MethodMetrics>(),
            Dependencies = Array.Empty<string>()
        };
    }
}


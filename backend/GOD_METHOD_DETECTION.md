# God Method Detection

## Overview

In addition to detecting god classes, the tool now detects **god methods** (also called god functions) - methods that are overly complex and violate good design principles.

## What is a God Method?

A god method is a function/method that:
- Is too long (hard to understand)
- Has too many branches/logic paths (high complexity)
- Takes too many parameters (hard to use)

These methods are difficult to:
- **Understand**: Too much to keep in your head
- **Test**: Many edge cases and paths
- **Maintain**: Changes affect many things
- **Reuse**: Tightly coupled to specific use cases

## Detection Thresholds

A method is flagged as a god method if it exceeds ANY of these thresholds:

| Metric | Threshold | Why |
|--------|-----------|-----|
| **Lines** | > 50 | Methods longer than 50 lines become hard to understand |
| **Cyclomatic Complexity** | > 10 | More than 10 decision points indicates too much branching |
| **Parameters** | > 5 | More than 5 parameters are hard to remember and use |

## Configuration

Edit `appsettings.json`:

```json
{
  "DetectionThresholds": {
    "MaxMethodLines": 50,
    "MaxMethodComplexity": 10,
    "MaxMethodParameters": 5
  }
}
```

## Example Output

When god methods are detected, you'll see:

```
⚠️  GOD METHODS DETECTED:

  📝 OrderProcessor
     File: /src/Business/OrderProcessor.cs
     God Methods: 2

     ⚠️  CalculateOrderTotal()
        Lines: 82, Complexity: 16, Parameters: 1
        • Lines (82) exceeds threshold (50)
        • Complexity (16) exceeds threshold (10)

     ⚠️  ProcessOrder()
        Lines: 12, Complexity: 1, Parameters: 6
        • Parameters (6) exceeds threshold (5)
```

Each god method shows:
- **Method name** with `()` to indicate it's a function
- **Current metrics**: Lines, Complexity, Parameters
- **Specific violations**: Which thresholds were exceeded and by how much

## Tree View

Files with god methods are shown with a warning icon:

```
└── src/
    └── Business/
        └── PaymentProcessor.cs ⚠️ [3 GOD METHOD(S)]
```

## Violation Score

God methods are ranked by a **violation score**:
- Lines over threshold: +1 point per line
- Complexity over threshold: +2 points per complexity point
- Parameters over threshold: +1 point per parameter

This helps prioritize which methods to refactor first.

## How It Works

1. **AST Traversal**: Analyzes each method in every class
2. **Metric Calculation**: 
   - Lines: Non-empty lines in method body
   - Complexity: Number of decision points (if, while, for, etc.)
   - Parameters: Count of method parameters
3. **Threshold Comparison**: Checks each metric against thresholds
4. **Violation Tracking**: Records which thresholds are exceeded
5. **Reporting**: Displays methods ordered by violation severity

## Common God Method Patterns

### 1. The "Do Everything" Method

```csharp
// BAD: 100+ lines doing everything
public Order ProcessOrder(string customerId, List<Product> products, ...)
{
    // Validate input
    // Calculate prices
    // Apply discounts
    // Calculate tax
    // Process payment
    // Update inventory
    // Send notifications
    // Generate invoice
    // Update analytics
    // ...
}
```

**Fix**: Extract smaller, focused methods
- `ValidateOrder()`
- `CalculateTotalPrice()`
- `ProcessPayment()`
- `SendOrderConfirmation()`

### 2. The "Too Many Parameters" Method

```csharp
// BAD: 8 parameters
public void CreateUser(
    string firstName,
    string lastName, 
    string email,
    string password,
    string phone,
    DateTime birthDate,
    string address,
    string city)
{
    // ...
}
```

**Fix**: Use a parameter object
```csharp
// GOOD: 1 parameter object
public void CreateUser(UserRegistration registration)
{
    // ...
}
```

### 3. The "Deeply Nested" Method

```csharp
// BAD: High complexity from nested conditionals
public decimal CalculateDiscount(Order order)
{
    if (order.Total > 100)
    {
        if (order.Customer.IsVIP)
        {
            if (order.Items.Count > 10)
            {
                if (DateTime.Now.DayOfWeek == DayOfWeek.Friday)
                {
                    // ... 6 levels deep!
                }
            }
        }
    }
}
```

**Fix**: Use early returns and extraction
```csharp
// GOOD: Flat structure
public decimal CalculateDiscount(Order order)
{
    if (order.Total <= 100) return 0;
    if (!order.Customer.IsVIP) return GetStandardDiscount(order);
    if (order.Items.Count <= 10) return GetVIPDiscount(order);
    if (DateTime.Now.DayOfWeek != DayOfWeek.Friday) return GetBulkDiscount(order);
    
    return GetFridaySpecialDiscount(order);
}
```

## Best Practices

### Refactoring God Methods

1. **Extract Methods**: Break into smaller, focused methods
2. **Extract Classes**: Move related logic to new classes
3. **Use Parameter Objects**: Replace long parameter lists
4. **Simplify Logic**: Reduce nesting with early returns
5. **Single Responsibility**: Each method should do one thing

### Prevention

- **Code Reviews**: Catch complex methods before merging
- **CI/CD Integration**: Fail builds with god methods
- **Regular Scans**: Run `dei check` frequently
- **Team Standards**: Agree on complexity limits

## Integration Examples

### Pre-commit Hook

```bash
#!/bin/bash
cd backend
dotnet run --project src/GodClassDetector.Console check ./src
if [ $? -ne 0 ]; then
  echo "❌ God methods detected! Refactor before committing."
  exit 1
fi
```

### GitHub Actions

```yaml
- name: Check for God Methods
  run: |
    cd backend
    dotnet run --project src/GodClassDetector.Console check ./src
```

### Local Development

```bash
# Add to your .bashrc or .zshrc
alias check-code='cd /path/to/Dei/backend && dotnet run --project src/GodClassDetector.Console check'

# Then just run:
check-code ./src/MyProject
```

## Benefits

✅ **Early Detection**: Catch complexity before it becomes a problem
✅ **Objective Metrics**: Data-driven refactoring priorities
✅ **Team Alignment**: Shared understanding of "too complex"
✅ **Code Quality**: Enforces good design principles
✅ **Maintainability**: Easier to understand and modify code

## Metrics Explained

### Lines of Code
- **What**: Non-empty, non-comment lines in method body
- **Why**: Long methods are hard to understand
- **Threshold**: 50 lines (one screen worth)

### Cyclomatic Complexity
- **What**: Number of linearly independent paths through code
- **Why**: High complexity = many test cases needed
- **Threshold**: 10 (research-backed maintainability limit)
- **How**: +1 for each: if, while, for, case, &&, ||, ?:

### Parameter Count
- **What**: Number of parameters method accepts
- **Why**: Hard to remember and easy to misuse
- **Threshold**: 5 parameters (cognitive load limit)

## Real-World Example

From the examples directory:

```csharp
// Detected as god method:
public decimal CalculateOrderTotal(Order order)
{
    // 82 lines
    // 16 cyclomatic complexity
    // Multiple nested if/else statements
    // Complex business logic
}
```

**Violations:**
- Lines (82) exceeds threshold (50) → +32 points
- Complexity (16) exceeds threshold (10) → +12 points
- **Total violation score: 44** (high priority for refactoring)

**Refactoring suggestion:**
```csharp
public decimal CalculateOrderTotal(Order order)
{
    var subtotal = CalculateSubtotal(order);
    var discount = CalculateDiscount(order, subtotal);
    var tax = CalculateTax(order, subtotal - discount);
    var shipping = CalculateShipping(order, subtotal);
    
    return subtotal - discount + tax + shipping;
}
```

## Summary

God method detection helps you:
- Identify overly complex functions
- Prioritize refactoring efforts
- Maintain code quality
- Enforce team standards
- Prevent technical debt

Run `dei check` regularly to keep your codebase clean and maintainable!


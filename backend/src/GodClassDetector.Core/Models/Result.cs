namespace GodClassDetector.Core.Models;

/// <summary>
/// Represents the result of an operation that can succeed or fail
/// </summary>
public readonly record struct Result<T>
{
    private readonly T? _value;
    private readonly string? _error;

    public bool IsSuccess { get; }
    public T Value => IsSuccess ? _value! : throw new InvalidOperationException($"Cannot access Value when result is failed: {_error}");
    public string Error => !IsSuccess ? _error! : throw new InvalidOperationException("Cannot access Error when result is successful");

    private Result(T value)
    {
        IsSuccess = true;
        _value = value;
        _error = null;
    }

    private Result(string error)
    {
        IsSuccess = false;
        _value = default;
        _error = error;
    }

    public static Result<T> Success(T value) => new(value);
    public static Result<T> Failure(string error) => new(error);

    public TResult Match<TResult>(Func<T, TResult> onSuccess, Func<string, TResult> onFailure) =>
        IsSuccess ? onSuccess(Value) : onFailure(Error);

    public async Task<TResult> MatchAsync<TResult>(
        Func<T, Task<TResult>> onSuccess,
        Func<string, Task<TResult>> onFailure) =>
        IsSuccess ? await onSuccess(Value) : await onFailure(Error);
}


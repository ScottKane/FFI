using System.Runtime.InteropServices;

namespace Native.Interop;

[StructLayout(LayoutKind.Sequential)]
internal struct DbResult
{
    private enum Kind : uint
    {
        Ok,

        Done,
        BufferTooSmall,

        ArgumentNull,
        InternalError
    }

    private readonly Kind _result;
    private readonly uint _id;

    public static (DbResult, string) GetLastResult() => LastResult.GetLastResult();

    public DbResult Check()
    {
        if (IsSuccess() || IsBufferTooSmall()) return this;

        var (lastResult, msg) = GetLastResult();
            
        if (lastResult._result == _result && lastResult._id == _id)
            throw new Exception($"Native storage failed ({_result}), {msg?.TrimEnd()}");

        throw new Exception($"Native storage failed with {_result}");
    }

    public bool IsSuccess() => _result is Kind.Ok or Kind.Done;

    public bool IsDone() => _result == Kind.Done;

    public bool IsBufferTooSmall() => _result == Kind.BufferTooSmall;
}
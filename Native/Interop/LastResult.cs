using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;

namespace Native.Interop;

internal static partial class LastResult
{
    public static (DbResult, string) GetLastResult() => FillLastResult(new Span<byte>(new byte[1024]));

    private static unsafe (DbResult, string) FillLastResult(Span<byte> buffer)
    {
        fixed (byte* messageBufPtr = buffer)
        {
            var result = db_last_result(
                    (nint)messageBufPtr,
                    (nuint) buffer.Length,
                    out var actualMessageLen,
                    out var lastResult)
                .Check();

            return result.IsBufferTooSmall()
                ? FillLastResult(new Span<byte>(new byte[(int) actualMessageLen]))
                : (lastResult, Encoding.UTF8.GetString(messageBufPtr, (int) actualMessageLen));
        }
    }
    
    [LibraryImport(Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_last_result(nint messageBuf, nuint messageBufLen, out nuint actualMessageLen, out DbResult lastResult);
}
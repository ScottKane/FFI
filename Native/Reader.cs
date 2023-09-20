using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Native.Interop;

namespace Native;

public sealed partial class Reader : IDisposable
{
    private readonly ReaderHandle _handle;

    internal Reader(ReaderHandle handle) => _handle = handle ?? throw new ArgumentNullException(nameof(handle));

    public ReadResult TryReadNext(Span<byte> buffer)
    {
        unsafe
        {
            fixed (byte* bufferPtr = buffer)
            {
                var result = db_read_next(
                        _handle,
                        out var key,
                        (nint)bufferPtr,
                        (nuint)buffer.Length,
                        out var actualValueLength)
                    .Check();

                if (result.IsBufferTooSmall()) return ReadResult.BufferTooSmall((int) actualValueLength);

                return result.IsDone() ? ReadResult.Done() : ReadResult.Data(new Key(key), buffer, ..(int) actualValueLength);
            }
        }
    }
    
    public void Dispose() => _handle.Dispose();
    
    [LibraryImport(global::Native.Interop.Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_read_next(ReaderHandle reader, out DbKey key, nint valueBuf, nuint valueBufLen, out nuint actualValueLen);
}
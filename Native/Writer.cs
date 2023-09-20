using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Native.Interop;

namespace Native;

public sealed partial class Writer : IDisposable
{
    private readonly WriterHandle _handle;

    internal Writer(WriterHandle handle) => _handle = handle ?? throw new ArgumentNullException(nameof(handle));

    public void Dispose() => _handle.Dispose();

    public void Set(Key key, ReadOnlySpan<byte> value)
    {
        unsafe
        {
            var rawKey = key.Value;
            var keyPtr = Unsafe.AsPointer(ref rawKey);

            fixed (byte* valuePtr = value)
                db_write_set(_handle, (nint)keyPtr, (nint)valuePtr, (nuint)value.Length).Check();
        }
    }
    
    [LibraryImport(global::Native.Interop.Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_write_set(WriterHandle writer, nint key, nint value, nuint valueLen);
}
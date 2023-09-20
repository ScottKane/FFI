using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Native.Interop;

namespace Native;

public sealed partial class Deleter : IDisposable
{
    private readonly DeleterHandle _handle;

    internal Deleter(DeleterHandle handle) => _handle = handle ?? throw new ArgumentNullException(nameof(handle));

    public void Dispose() => _handle.Dispose();

    public void Remove(Key key)
    {
        unsafe
        {
            var rawKey = key.Value;
            var keyPtr = Unsafe.AsPointer(ref rawKey);

            db_delete_remove(_handle, (nint)keyPtr).Check();
        }
    }
    
    [LibraryImport(global::Native.Interop.Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_delete_remove(DeleterHandle deleter, nint key);
}
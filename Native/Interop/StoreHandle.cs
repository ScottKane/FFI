using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Native.Interop;

internal partial class StoreHandle : SafeHandle
{
    public StoreHandle() : base(nint.Zero, true) { }

    public override bool IsInvalid => handle == nint.Zero;

    protected override bool ReleaseHandle()
    {
        db_store_close(handle);
        return true;
    }
    
    [LibraryImport(Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_store_close(nint store);
}
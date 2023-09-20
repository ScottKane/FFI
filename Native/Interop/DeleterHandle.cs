using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Native.Interop;

internal partial class DeleterHandle : SafeHandle
{
    public DeleterHandle() : base(nint.Zero, true) { }

    public override bool IsInvalid => handle == nint.Zero;

    protected override bool ReleaseHandle()
    {
        if (handle == nint.Zero) return true;

        var h = handle;
        handle = nint.Zero;

        db_delete_end(h);
        return true;
    }
    
    [LibraryImport(Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_delete_end(nint deleter);
}
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Native.Interop;

internal partial class ReaderHandle : SafeHandle
{
    public ReaderHandle() : base(nint.Zero, true) { }

    public override bool IsInvalid => handle == nint.Zero;

    protected override bool ReleaseHandle()
    {
        if (handle == nint.Zero) return true;

        var h = handle;
        handle = nint.Zero;

        db_read_end(h);
        return true;
    }
    
    [LibraryImport(Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_read_end(nint reader);
}
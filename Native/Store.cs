using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using Native.Interop;

namespace Native;

public sealed partial class Store : IDisposable
{
    private readonly StoreHandle? _handle;

    public void Dispose() => _handle?.Dispose();

    public Store(string path)
    {
        ArgumentNullException.ThrowIfNull(path);
        var pathUtf8 = Encoding.UTF8.GetBytes(path);

        unsafe
        {
            fixed (byte* pathUtf8Ptr = pathUtf8)
            {
                db_store_open((nint)pathUtf8Ptr, (nuint) pathUtf8.Length, out var handle).Check();
                _handle = handle;
            }
        }
    }

    public Reader BeginRead()
    {
        db_read_begin(_handle, out var readerHandle).Check();
        return new Reader(readerHandle);
    }

    public Writer BeginWrite()
    {
        db_write_begin(_handle, out var writerHandle).Check();
        return new Writer(writerHandle);
    }

    public Deleter BeginDelete()
    {
        db_delete_begin(_handle, out var deleterHandle).Check();
        return new Deleter(deleterHandle);
    }
    
    [LibraryImport(global::Native.Interop.Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_store_open(nint path, nuint pathLen, out StoreHandle store);
    
    [LibraryImport(global::Native.Interop.Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_read_begin(StoreHandle? store, out ReaderHandle reader);
    
    [LibraryImport(global::Native.Interop.Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_write_begin(StoreHandle? store, out WriterHandle writer);
    
    [LibraryImport(global::Native.Interop.Native.Unnamed), UnmanagedCallConv(CallConvs = new []{ typeof(CallConvCdecl) })]
    private static partial DbResult db_delete_begin(StoreHandle? store, out DeleterHandle deleter);
}
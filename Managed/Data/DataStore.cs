using System.Buffers;
using Store = Native.Store;

namespace Managed;

public sealed class DataStore : IDisposable
{
    private readonly MemoryPool<byte> _pool;
    private readonly Store _store;

    public DataStore(MemoryPool<byte> pool, Store store)
    {
        _pool = pool ?? throw new ArgumentNullException(nameof(pool));
        _store = store ?? throw new ArgumentNullException(nameof(store));
    }

    public void Dispose()
    {
        _store.Dispose();
        _pool.Dispose();
    }

    public DataReader BeginRead() => new(_pool, _store.BeginRead());

    public DataWriter BeginWrite() => new(_store.BeginWrite());

    public DataDeleter BeginDelete() => new(_store.BeginDelete());
}
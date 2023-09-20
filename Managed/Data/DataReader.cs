using System.Buffers;
using Native;
using Reader = Native.Reader;

namespace Managed;

public sealed class DataReader : IDisposable
{
    private readonly MemoryPool<byte> _pool;
    private readonly Reader _reader;

    internal DataReader(MemoryPool<byte> pool, Reader reader)
    {
        _pool = pool ?? throw new ArgumentNullException(nameof(pool));
        _reader = reader ?? throw new ArgumentNullException(nameof(reader));
    }

    public void Dispose() => _reader.Dispose();

    public IEnumerable<Data> Data()
    {
        var requiredSize = 1024;
        while (true)
        {
            var readInto = _pool.Rent(requiredSize);
            ReadResult read;

            try
            {
                read = _reader.TryReadNext(readInto.Memory.Span);
            }
            catch
            {
                readInto.Dispose();
                throw;
            }
                
            if (read.IsDone)
            {
                readInto.Dispose();
                yield break;
            }
                
            if (read.IsBufferTooSmall(out var required))
            {
                requiredSize = required;
                readInto.Dispose();
                continue;
            }
                
            read.GetData(out var key, out var payload);
            yield return new Data(key, readInto, payload.Range);
        }
    }
}
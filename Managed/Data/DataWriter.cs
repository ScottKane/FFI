using Writer = Native.Writer;

namespace Managed;

public sealed class DataWriter : IDisposable
{
    private readonly Writer _writer;

    internal DataWriter(Writer writer)
    {
        _writer = writer ?? throw new ArgumentNullException(nameof(writer));
    }

    public void Dispose() => _writer.Dispose();

    public void Set(Data data) => _writer.Set(data.Key, data.RawValue.Span);
}
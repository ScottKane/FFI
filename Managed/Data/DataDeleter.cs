using Native;
using Deleter = Native.Deleter;

namespace Managed;

public sealed class DataDeleter : IDisposable
{
    private readonly Deleter _deleter;

    internal DataDeleter(Deleter deleter) => _deleter = deleter ?? throw new ArgumentNullException(nameof(deleter));

    public void Dispose() => _deleter.Dispose();

    public void Remove(Key key) => _deleter.Remove(key);
}
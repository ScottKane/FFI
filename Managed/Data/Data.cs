using System.Buffers;
using System.Text.Json;
using System.Text.Json.Serialization;
using Native;

namespace Managed;

public sealed class Data : IDisposable
{
    private readonly IMemoryOwner<byte>? _memory;

    private readonly JsonDocument _value;
        
    public Data(Key key, MemoryStream memory)
    {
        Key = key;
        RawValue = memory.ToArray().AsMemory();
            
        _value = JsonDocument.Parse(RawValue);
    }

    public Data(Key key, IMemoryOwner<byte> value, Range range)
    {
        Key = key;
        
        var (start, length) = range.GetOffsetAndLength(value.Memory.Length);
        RawValue = value.Memory.Slice(start, length);

        _memory = value;
        _value = JsonDocument.Parse(RawValue);
    }

    [JsonConverter(typeof(Key.Converter))]
    public Key Key { get; }
    public JsonElement Value => _value.RootElement;
    internal ReadOnlyMemory<byte> RawValue { get; }
    
    public void Dispose()
    {
        _value.Dispose();
        _memory?.Dispose();
    }
}
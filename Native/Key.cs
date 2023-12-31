using System.Runtime.CompilerServices;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using Native.Interop;

namespace Native;

public struct Key
{
    private static readonly Encoder Encoder = Encoding.ASCII.GetEncoder();

    private DbKey _key;

    internal Key(DbKey key) => _key = key;

    public Key(string key)
    {
        if (key.Length < 10) throw new ArgumentException("The key is too short", nameof(key));
        if (key[8] != '-') throw new Exception("The key is in an invalid format");

        var hi = key.Substring(0, 8);
        var lo = Convert.ToUInt64(key.Substring(9));

        _key = BuildDbKey(hi, lo);
    }

    public Key(string hi, ulong lo) => _key = BuildDbKey(hi, lo);

    public override string ToString()
    {
        var (hi, lo) = this;

        return $"{hi}-{lo}";
    }

    internal DbKey Value => _key;

    private static DbKey BuildDbKey(string hi, ulong lo)
    {
        unsafe
        {
            var key = default(DbKey);
            var keyPtr = Unsafe.AsPointer(ref key);

            var written = Encoder.GetBytes(hi.AsSpan(), new Span<byte>(keyPtr, 8), true);
            if (written != 8)
                throw new ArgumentException("The hi string must contain exactly 8 ASCII chars", nameof(hi));

            Unsafe.WriteUnaligned(Unsafe.Add<ulong>(keyPtr, 1), lo);

            return key;
        }
    }

    public void Deconstruct(out string hi, out ulong lo)
    {
        var local = _key;

        unsafe
        {
            var localPtr = Unsafe.AsPointer(ref local);

            hi = Encoding.ASCII.GetString((byte*) localPtr, 8);
            lo = Unsafe.ReadUnaligned<ulong>(Unsafe.Add<ulong>(localPtr, 1));
        }
    }

    public static bool operator ==(Key lhs, Key rhs) => lhs.Equals(rhs);

    public static bool operator !=(Key lhs, Key rhs) => !(lhs == rhs);

    private bool Equals(Key other)
    {
        var local = _key;

        unsafe
        {
            var localPtr = Unsafe.AsPointer(ref local);
            var otherPtr = Unsafe.AsPointer(ref other._key);

            var thisSpan = new Span<byte>(localPtr, 16);
            var otherSpan = new Span<byte>(otherPtr, 16);

            return thisSpan.SequenceEqual(otherSpan);
        }
    }

    public override bool Equals(object? obj)
    {
        if (ReferenceEquals(null, obj)) return false;
        return obj is Key other && Equals(other);
    }

    public override int GetHashCode()
    {
        var local = this;

        unsafe
        {
            var localPtr = Unsafe.AsPointer(ref local);

            var hi = Unsafe.ReadUnaligned<ulong>(localPtr);
            var lo = Unsafe.ReadUnaligned<ulong>(Unsafe.Add<ulong>(localPtr, 1));

            return (hi.GetHashCode() * 397) ^ lo.GetHashCode();
        }
    }
    
    public class Converter : JsonConverter<Key>
    {
        public override Key Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options) => throw new NotImplementedException();

        public override void Write(Utf8JsonWriter writer, Key value, JsonSerializerOptions options)
        {
            writer.WriteStringValue(value.ToString());
        }
    }
}
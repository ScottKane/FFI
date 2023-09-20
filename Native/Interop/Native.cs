namespace Native.Interop;

internal static class Native
{
#if AOT
    public const string Unnamed = "*";
#elif WINDOWS
    public const string Unnamed = "Native/ffi.dll";
#elif LINUX
    public const string Unnamed = "Native/libffi.so";
#elif MACOS
    public const string Unnamed = "Native/libffi.dylib";
#endif
}
using System.Buffers;
using Managed;
using Managed.Options;
using Microsoft.AspNetCore.Http.HttpResults;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Options;
using Native;

var builder = WebApplication.CreateSlimBuilder(args);

builder.Services.Configure<Storage>(builder.Configuration.GetSection(nameof(Storage)));

builder.Services.AddSingleton(MemoryPool<byte>.Shared);
builder.Services.AddSingleton(provider => new Store(provider.GetRequiredService<IOptions<Storage>>().Value.Path));
builder.Services.AddSingleton<DataStore>();

var app = builder.Build();

app.MapGet("/", Task<Ok<Data[]>> ([FromServices] IServiceProvider provider) =>
{
    var store = provider.GetRequiredService<DataStore>();
    using var reader = store.BeginRead();
    var data = reader.Data().ToArray();

    return Task.FromResult(TypedResults.Ok(data));
});

app.MapPost("/{key}", async Task<Ok> (string key, HttpRequest request, [FromServices] IServiceProvider provider) =>
{
    var store = provider.GetRequiredService<DataStore>();

    using var memory = new MemoryStream();
    await request.Body.CopyToAsync(memory);
            
    using var doc = new Data(new Key(key), memory);
    using var write = store.BeginWrite();
    write.Set(doc);

    return TypedResults.Ok();
});

app.MapDelete("/{key}", Task<Ok> (string key, [FromServices] IServiceProvider provider) =>
{
    var store = provider.GetRequiredService<DataStore>();
    using var remove = store.BeginDelete();
    remove.Remove(new Key(key));

    return Task.FromResult(TypedResults.Ok());
});

await app.RunAsync();
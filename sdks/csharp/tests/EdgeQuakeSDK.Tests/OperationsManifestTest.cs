using System.Text.Json;

namespace EdgeQuakeSDK.Tests;

public class OperationsManifestTest
{
    [Fact]
    public void EveryHttpOperationPathAndGeneratedVerb()
    {
        var root = FindRepo();
        var ops = JsonDocument.Parse(File.ReadAllText(Path.Combine(root, "sdks", "operation-manifest.json")));
        var generated = File.ReadAllText(Path.Combine(root, "sdks", "csharp", "src", "EdgeQuakeSDK", "OperationsGenerated.cs"));
        var blob = string.Concat(Directory.GetFiles(Path.Combine(root, "sdks", "csharp", "src"), "*.cs", SearchOption.AllDirectories).Select(File.ReadAllText));
        foreach (var op in ops.RootElement.EnumerateArray())
        {
            if (op.GetProperty("kind").GetString() == "websocket")
            {
                Assert.Contains(op.GetProperty("path").GetString()!, generated);
                continue;
            }
            var path = op.GetProperty("path").GetString()!;
            Assert.Contains(path, blob);
            if (op.GetProperty("missing").GetProperty("csharp").GetBoolean())
                Assert.Contains(op.GetProperty("method").GetString() + " " + path, generated);
        }
    }

    static string FindRepo()
    {
        var dir = new DirectoryInfo(Directory.GetCurrentDirectory());
        for (var i = 0; i < 8 && dir != null; i++)
        {
            if (File.Exists(Path.Combine(dir.FullName, "sdks", "operation-manifest.json")))
                return dir.FullName;
            dir = dir.Parent;
        }
        throw new InvalidOperationException("operation-manifest.json not found");
    }
}

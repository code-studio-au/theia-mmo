// Read-only offline importer/effective-texture evidence in an ignored project.
using System;
using System.IO;
using System.Security.Cryptography;
using UnityEditor;
using UnityEngine;

public static class TheiaTextureReference
{
    [Serializable] public class Input { public string path, guid, sha256, meta_sha256; }
    [Serializable] public class Request
    {
        public int schema;
        public string archive_sha256, collector_sha256, unity_version;
        public Input[] textures;
    }
    [Serializable] public class Texture
    {
        public string path, guid, sha256, meta_sha256, file_id, format;
        public int width, height, mip_count, filter, wrap_u, wrap_v, aniso;
        public float mip_bias;
        public int importer_filter, importer_wrap_u, importer_wrap_v, importer_type;
        public bool srgb, mipmaps, convert_normal;
    }
    [Serializable] public class Report
    {
        public int schema = 1;
        public string unity_version, platform, request_sha256, collector_sha256;
        public Texture[] textures;
    }
    static string Hash(string path)
    {
        using (var hash = SHA256.Create())
        using (var stream = File.OpenRead(path))
            return BitConverter.ToString(hash.ComputeHash(stream)).Replace("-", "").ToLowerInvariant();
    }
    static string Argument(string key)
    {
        var args = Environment.GetCommandLineArgs();
        for (int i = 0; i < args.Length - 1; ++i) if (args[i] == key) return args[i + 1];
        throw new InvalidOperationException("Missing " + key);
    }
    public static void Run()
    {
        try
        {
            var requestPath = Argument("-theiaTextureRequest");
            var output = Argument("-theiaTextureOutput");
            if (File.Exists(output)) throw new InvalidOperationException("Output exists");
            var request = JsonUtility.FromJson<Request>(File.ReadAllText(requestPath));
            var collector = Hash(Path.Combine(Application.dataPath, "Editor/TheiaTextureReference.cs"));
            if (request.schema != 1 || request.unity_version != Application.unityVersion || request.collector_sha256 != collector || request.textures.Length < 1 || request.textures.Length > 32)
                throw new InvalidOperationException("Request/collector/version differs");
            var result = new Texture[request.textures.Length];
            for (int i = 0; i < result.Length; ++i)
            {
                var input = request.textures[i];
                if (!input.path.StartsWith("Assets/", StringComparison.Ordinal) || input.path.Contains("..") || !input.path.EndsWith(".png", StringComparison.OrdinalIgnoreCase) || Hash(input.path) != input.sha256 || Hash(input.path + ".meta") != input.meta_sha256 || AssetDatabase.AssetPathToGUID(input.path) != input.guid)
                    throw new InvalidOperationException("Texture input binding differs");
                AssetDatabase.ImportAsset(input.path, ImportAssetOptions.ForceSynchronousImport);
                var importer = AssetImporter.GetAtPath(input.path) as TextureImporter;
                var texture = AssetDatabase.LoadAssetAtPath<Texture2D>(input.path);
                string guid; long id;
                if (importer == null || texture == null || !AssetDatabase.TryGetGUIDAndLocalFileIdentifier(texture, out guid, out id) || guid != input.guid)
                    throw new InvalidOperationException("Texture persistent identity differs");
                if (texture.width < 1 || texture.height < 1 || texture.width > 4096 || texture.height > 4096)
                    throw new InvalidOperationException("Texture dimensions exceed profile");
                result[i] = new Texture {
                    path = input.path, guid = guid, file_id = id.ToString(System.Globalization.CultureInfo.InvariantCulture),
                    sha256 = Hash(input.path), meta_sha256 = Hash(input.path + ".meta"),
                    width = texture.width, height = texture.height, mip_count = texture.mipmapCount, format = texture.format.ToString(),
                    filter = (int)texture.filterMode, wrap_u = (int)texture.wrapModeU, wrap_v = (int)texture.wrapModeV,
                    aniso = texture.anisoLevel, mip_bias = texture.mipMapBias,
                    importer_filter = (int)importer.filterMode, importer_wrap_u = (int)importer.wrapModeU,
                    importer_wrap_v = (int)importer.wrapModeV, importer_type = (int)importer.textureType,
                    srgb = importer.sRGBTexture, mipmaps = importer.mipmapEnabled, convert_normal = importer.convertToNormalmap
                };
                if (result[i].sha256 != input.sha256 || result[i].meta_sha256 != input.meta_sha256)
                    throw new InvalidOperationException("Editor changed original texture or metadata");
            }
            var report = new Report { unity_version = Application.unityVersion, platform = Application.platform.ToString(),
                request_sha256 = Hash(requestPath), collector_sha256 = collector, textures = result };
            Directory.CreateDirectory(Path.GetDirectoryName(output));
            File.WriteAllText(output, JsonUtility.ToJson(report, true));
            EditorApplication.Exit(0);
        }
        catch (Exception error) { Debug.LogException(error); EditorApplication.Exit(1); }
    }
}

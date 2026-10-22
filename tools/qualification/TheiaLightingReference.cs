// Offline lighting prerequisites only. Empty project; no source scripts/scenes.
using System;
using System.Collections.Generic;
using System.IO;
using System.Security.Cryptography;
using UnityEditor;
using UnityEngine;

public static class TheiaLightingReference
{
    [Serializable] public class Input { public string path, guid, sha256, meta_sha256; }
    [Serializable] public class Request {
        public int schema; public string unity_version, collector_sha256, archive_sha256;
        public Input[] models, prefabs, textures;
    }
    [Serializable] public class Identity { public string guid, file_id; }
    [Serializable] public class RendererLayer {
        public string name; public int layer;
        public Identity renderer, game_object, original_renderer, original_game_object;
    }
    [Serializable] public class PrefabLayer {
        public string path, guid; public RendererLayer[] renderers;
    }
    [Serializable] public class Cookie {
        public string path, guid, file_id, format;
        public int importer_type, alpha_source, width, height, filter, wrap_u, wrap_v, aniso, mip_count;
        public float mip_bias; public bool srgb, mipmaps;
    }
    [Serializable] public class Report {
        public int schema = 1;
        public string unity_version, platform, request_sha256, collector_sha256;
        public PrefabLayer[] prefabs; public Cookie[] textures;
    }
    static string Hash(string path) {
        using (var hash = SHA256.Create())
        using (var stream = File.OpenRead(path))
            return BitConverter.ToString(hash.ComputeHash(stream)).Replace("-", "").ToLowerInvariant();
    }
    static string Argument(string name) {
        var args=Environment.GetCommandLineArgs();
        for (int i=0;i<args.Length-1;++i) if(args[i]==name)return Path.GetFullPath(args[i+1]);
        throw new InvalidOperationException("Missing argument "+name);
    }
    static Identity Id(UnityEngine.Object value) {
        string guid;long id;
        if (!value || !AssetDatabase.TryGetGUIDAndLocalFileIdentifier(value,out guid,out id))
            throw new InvalidOperationException("Missing persistent layer identity");
        return new Identity {guid=guid,file_id=id.ToString(System.Globalization.CultureInfo.InvariantCulture)};
    }
    static void Check(Input input) {
        if (!input.path.StartsWith("Assets/",StringComparison.Ordinal) || input.path.Contains("..") || input.path.Contains("\\"))
            throw new InvalidOperationException("Invalid staged input path");
        var path=Path.Combine(Path.GetDirectoryName(Application.dataPath),input.path);
        if (Hash(path)!=input.sha256 || Hash(path+".meta")!=input.meta_sha256 || AssetDatabase.AssetPathToGUID(input.path)!=input.guid)
            throw new InvalidOperationException("Staged input/metadata/GUID changed");
    }
    public static void Export() {
        try {
            var requestPath=Argument("-theiaRequest");var output=Argument("-theiaOutput");
            var request=JsonUtility.FromJson<Request>(File.ReadAllText(requestPath));
            var collector=Hash(Path.Combine(Application.dataPath,"Editor/TheiaLightingReference.cs"));
            if(request.schema!=1 || request.unity_version!=Application.unityVersion || request.collector_sha256!=collector || request.prefabs.Length!=4 || request.textures.Length!=2 || request.models.Length!=4)
                throw new InvalidOperationException("Unqualified lighting reference request/version/collector");
            foreach(var input in request.models)Check(input);
            var prefabs=new List<PrefabLayer>();
            foreach(var input in request.prefabs) {
                Check(input);var root=AssetDatabase.LoadAssetAtPath<GameObject>(input.path);
                if(!root)throw new InvalidOperationException("Missing staged variant");
                var renderers=new List<RendererLayer>();
                foreach(var renderer in root.GetComponentsInChildren<MeshRenderer>(true)) {
                    var original=PrefabUtility.GetCorrespondingObjectFromOriginalSource(renderer);
                    if(!original || renderer.gameObject.layer<0 || renderer.gameObject.layer>31)
                        throw new InvalidOperationException("Unbound variant renderer/original layer");
                    renderers.Add(new RendererLayer { name=renderer.name,layer=renderer.gameObject.layer,
                        renderer=Id(renderer),game_object=Id(renderer.gameObject),original_renderer=Id(original),original_game_object=Id(original.gameObject) });
                }
                if(renderers.Count<1 || renderers.Count>256)throw new InvalidOperationException("Variant renderer budget exceeded");
                prefabs.Add(new PrefabLayer {path=input.path,guid=input.guid,renderers=renderers.ToArray()});
                Check(input);
            }
            var textures=new List<Cookie>();
            foreach(var input in request.textures) {
                Check(input);var importer=AssetImporter.GetAtPath(input.path) as TextureImporter;
                var texture=AssetDatabase.LoadAssetAtPath<Texture2D>(input.path);
                if(!importer || !texture || texture.width>4096 || texture.height>4096)throw new InvalidOperationException("Invalid source cookie reference");
                var id=Id(texture);
                if(id.guid!=input.guid || id.file_id!="2800000")throw new InvalidOperationException("Cookie identity differs");
                textures.Add(new Cookie {path=input.path,guid=id.guid,file_id=id.file_id,format=texture.format.ToString(),
                    importer_type=(int)importer.textureType,alpha_source=(int)importer.alphaSource,
                    width=texture.width,height=texture.height,filter=(int)texture.filterMode,wrap_u=(int)texture.wrapModeU,wrap_v=(int)texture.wrapModeV,
                    aniso=texture.anisoLevel,mip_count=texture.mipmapCount,mip_bias=texture.mipMapBias,srgb=importer.sRGBTexture,mipmaps=importer.mipmapEnabled});
                Check(input);
            }
            var report=new Report {unity_version=Application.unityVersion,platform=Application.platform.ToString(),
                request_sha256=Hash(requestPath),collector_sha256=collector,prefabs=prefabs.ToArray(),textures=textures.ToArray()};
            using(var stream=new FileStream(output,FileMode.CreateNew,FileAccess.Write))
            using(var writer=new StreamWriter(stream))writer.Write(JsonUtility.ToJson(report));
            Debug.Log("THEIA_LIGHTING_REFERENCE_COMPLETE");EditorApplication.Exit(0);
        } catch(Exception error) {Debug.LogException(error);EditorApplication.Exit(1);}
    }
}

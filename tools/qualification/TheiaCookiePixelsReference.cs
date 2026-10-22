// Offline cookie alpha reference. Only temporary CPU readability changes in a
// fresh ignored project; original package files and saved original metadata stay intact.
using System;
using System.Collections.Generic;
using System.IO;
using System.Security.Cryptography;
using UnityEditor;
using UnityEngine;

public static class TheiaCookiePixelsReference
{
    [Serializable] public class Input { public string path,guid,sha256,meta_sha256; }
    [Serializable] public class Request { public int schema;public string unity_version,collector_sha256,archive_sha256;public Input[] textures; }
    [Serializable] public class Cookie {
        public string path,guid,file_id,format,alpha_file,alpha_sha256,settings_before,settings_after,platform_before,platform_after,restored_meta_sha256;
        public int width,height,alpha_source,mip_count;
        public bool original_readable,restored_readable;
    }
    [Serializable] public class Report {
        public int schema=1;public string unity_version,platform,request_sha256,collector_sha256,orientation="top-left rows";
        public Cookie[] textures;
    }
    static string Hash(string path) {
        using(var hash=SHA256.Create())using(var stream=File.OpenRead(path))return BitConverter.ToString(hash.ComputeHash(stream)).Replace("-","").ToLowerInvariant();
    }
    static string Argument(string name) {
        var a=Environment.GetCommandLineArgs();for(int i=0;i<a.Length-1;++i)if(a[i]==name)return Path.GetFullPath(a[i+1]);throw new InvalidOperationException("Missing "+name);
    }
    static string Settings(TextureImporter importer) {
        var s=new TextureImporterSettings();importer.ReadTextureSettings(s);var json=JsonUtility.ToJson(s);
        if(json.Length<16)throw new InvalidOperationException("Missing importer settings snapshot");return json;
    }
    static string Platforms(TextureImporter importer) {
        return JsonUtility.ToJson(importer.GetDefaultPlatformTextureSettings())+"\n"+JsonUtility.ToJson(importer.GetPlatformTextureSettings("Standalone"));
    }
    public static void Export() {
        try {
            var requestPath=Argument("-theiaRequest");var output=Argument("-theiaOutput");var directory=Path.GetDirectoryName(output);
            var request=JsonUtility.FromJson<Request>(File.ReadAllText(requestPath));var collector=Hash(Path.Combine(Application.dataPath,"Editor/TheiaCookiePixelsReference.cs"));
            if(request.schema!=1 || request.unity_version!=Application.unityVersion || request.collector_sha256!=collector || request.textures.Length!=2)throw new InvalidOperationException("Unqualified cookie reference request");
            var values=new List<Cookie>();
            foreach(var input in request.textures) {
                if(!input.path.StartsWith("Assets/",StringComparison.Ordinal) || input.path.Contains("..") || input.path.Contains("\\"))throw new InvalidOperationException("Invalid source texture path");
                var physical=Path.Combine(Path.GetDirectoryName(Application.dataPath),input.path);
                if(Hash(physical)!=input.sha256 || Hash(physical+".meta")!=input.meta_sha256)throw new InvalidOperationException("Initial cookie bytes/metadata differ");
                var importer=AssetImporter.GetAtPath(input.path) as TextureImporter;
                var texture=AssetDatabase.LoadAssetAtPath<Texture2D>(input.path);string guid;long id;
                if(!importer || !texture || !AssetDatabase.TryGetGUIDAndLocalFileIdentifier(texture,out guid,out id) || guid!=input.guid || id!=2800000 || texture.format!=TextureFormat.Alpha8 || importer.alphaSource!=TextureImporterAlphaSource.FromGrayScale || texture.width!=2048 || texture.height!=2048)
                    throw new InvalidOperationException("Source imported cookie format/identity differs");
                var before=Settings(importer);var platforms=Platforms(importer);bool originalReadable=importer.isReadable;
                var result=new Cookie {path=input.path,guid=guid,file_id=id.ToString(),format=texture.format.ToString(),width=texture.width,height=texture.height,alpha_source=(int)importer.alphaSource,mip_count=texture.mipmapCount,original_readable=originalReadable,settings_before=before,platform_before=platforms};
                try {
                    importer.isReadable=true;importer.SaveAndReimport();texture=AssetDatabase.LoadAssetAtPath<Texture2D>(input.path);
                    if(!texture.isReadable || texture.format!=TextureFormat.Alpha8 || texture.width!=result.width || texture.height!=result.height || texture.mipmapCount!=result.mip_count)throw new InvalidOperationException("Readability changed cookie pixel representation");
                    var pixels=texture.GetPixels32(0);var alpha=new byte[result.width*result.height];
                    if(pixels.Length!=alpha.Length)throw new InvalidOperationException("Cookie pixel budget differs");
                    // GetPixels32 is bottom-left rows; publish explicit top-left rows.
                    for(int y=0;y<result.height;++y)for(int x=0;x<result.width;++x)alpha[y*result.width+x]=pixels[(result.height-1-y)*result.width+x].a;
                    result.alpha_file="cookie-"+input.guid+".alpha8";var path=Path.Combine(directory,result.alpha_file);
                    using(var stream=new FileStream(path,FileMode.CreateNew,FileAccess.Write))stream.Write(alpha,0,alpha.Length);
                    result.alpha_sha256=Hash(path);
                } finally {
                    importer=AssetImporter.GetAtPath(input.path) as TextureImporter;importer.isReadable=originalReadable;importer.SaveAndReimport();
                }
                importer=AssetImporter.GetAtPath(input.path) as TextureImporter;result.settings_after=Settings(importer);result.platform_after=Platforms(importer);result.restored_readable=importer.isReadable;result.restored_meta_sha256=Hash(physical+".meta");
                if(result.settings_before!=result.settings_after || result.platform_before!=result.platform_after || originalReadable!=result.restored_readable || Hash(physical)!=input.sha256)throw new InvalidOperationException("Cookie settings/pixels not restored after CPU access");
                values.Add(result);
            }
            var report=new Report {unity_version=Application.unityVersion,platform=Application.platform.ToString(),request_sha256=Hash(requestPath),collector_sha256=collector,textures=values.ToArray()};
            using(var stream=new FileStream(output,FileMode.CreateNew,FileAccess.Write))using(var writer=new StreamWriter(stream))writer.Write(JsonUtility.ToJson(report));
            Debug.Log("THEIA_COOKIE_PIXELS_REFERENCE_COMPLETE");EditorApplication.Exit(0);
        }catch(Exception error){Debug.LogException(error);EditorApplication.Exit(1);}
    }
}

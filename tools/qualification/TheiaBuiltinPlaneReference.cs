// Offline reference only: empty scratch project, no source scripts or scenes.
using System;
using System.IO;
using System.Security.Cryptography;
using Unity.Collections;
using UnityEditor;
using UnityEngine;
using UnityEngine.Rendering;

public static class TheiaBuiltinPlaneReference
{
    [Serializable] public class Request
    {
        public int schema;
        public string unity_version, collector_sha256, primitive, guid, file_id;
    }
    [Serializable] public class Report
    {
        public int schema = 1;
        public string unity_version, platform, request_sha256, collector_sha256, primitive, guid, file_id;
        public Vector3 bounds_center, bounds_size;
        public float[] positions, normals, uv0;
        public int[] indices;
    }
    static string Hash(string path)
    {
        using (var hash = SHA256.Create())
        using (var stream = File.OpenRead(path))
            return BitConverter.ToString(hash.ComputeHash(stream)).Replace("-", "").ToLowerInvariant();
    }
    static string Argument(string name)
    {
        var args = Environment.GetCommandLineArgs();
        for (int i = 0; i < args.Length - 1; ++i)
            if (args[i] == name) return Path.GetFullPath(args[i + 1]);
        throw new InvalidOperationException("Missing argument " + name);
    }
    public static void Export()
    {
        GameObject temporary = null;
        try
        {
            var requestPath = Argument("-theiaRequest");
            var output = Argument("-theiaOutput");
            var request = JsonUtility.FromJson<Request>(File.ReadAllText(requestPath));
            var collector = Hash(Path.Combine(Application.dataPath, "Editor/TheiaBuiltinPlaneReference.cs"));
            if (request.schema != 1 || request.unity_version != Application.unityVersion ||
                request.collector_sha256 != collector || request.primitive != "Plane" ||
                request.guid != "0000000000000000e000000000000000" || request.file_id != "10209")
                throw new InvalidOperationException("Unqualified built-in mesh request/version/collector");
            temporary = GameObject.CreatePrimitive(PrimitiveType.Plane);
            var mesh = temporary.GetComponent<MeshFilter>().sharedMesh;
            string guid; long id;
            if (!AssetDatabase.TryGetGUIDAndLocalFileIdentifier(mesh, out guid, out id) ||
                guid != request.guid || id.ToString(System.Globalization.CultureInfo.InvariantCulture) != request.file_id ||
                mesh.subMeshCount != 1 || mesh.vertexCount < 3 || mesh.vertexCount > 65536 || mesh.blendShapeCount != 0)
                throw new InvalidOperationException("Built-in Plane identity/geometry differs");
            var report = new Report {
                unity_version = Application.unityVersion, platform = Application.platform.ToString(),
                request_sha256 = Hash(requestPath), collector_sha256 = collector,
                primitive = request.primitive, guid = guid, file_id = request.file_id,
                bounds_center = mesh.bounds.center, bounds_size = mesh.bounds.size
            };
            using (var dataArray = MeshUtility.AcquireReadOnlyMeshData(mesh))
            {
                var data = dataArray[0];
                var descriptor = data.GetSubMesh(0);
                if (descriptor.topology != MeshTopology.Triangles || descriptor.indexCount < 3 ||
                    descriptor.indexCount % 3 != 0 || descriptor.indexCount > 393216)
                    throw new InvalidOperationException("Unsupported Plane topology/budget");
                using (var positions = new NativeArray<Vector3>(data.vertexCount, Allocator.Temp))
                using (var normals = new NativeArray<Vector3>(data.vertexCount, Allocator.Temp))
                using (var uv = new NativeArray<Vector2>(data.vertexCount, Allocator.Temp))
                using (var indices = new NativeArray<int>(descriptor.indexCount, Allocator.Temp))
                {
                    data.GetVertices(positions); data.GetNormals(normals); data.GetUVs(0, uv);
                    data.GetIndices(indices, 0, true);
                    report.positions = new float[3 * data.vertexCount];
                    report.normals = new float[3 * data.vertexCount];
                    report.uv0 = new float[2 * data.vertexCount];
                    for (int i = 0; i < data.vertexCount; ++i)
                    {
                        report.positions[3*i] = positions[i].x;
                        report.positions[3*i+1] = positions[i].y;
                        report.positions[3*i+2] = positions[i].z;
                        report.normals[3*i] = normals[i].x;
                        report.normals[3*i+1] = normals[i].y;
                        report.normals[3*i+2] = normals[i].z;
                        report.uv0[2*i] = uv[i].x;
                        report.uv0[2*i+1] = uv[i].y;
                    }
                    report.indices = indices.ToArray();
                }
            }
            using (var stream = new FileStream(output, FileMode.CreateNew, FileAccess.Write))
            using (var writer = new StreamWriter(stream)) writer.Write(JsonUtility.ToJson(report));
            Debug.Log("THEIA_BUILTIN_PLANE_REFERENCE_COMPLETE");
            UnityEngine.Object.DestroyImmediate(temporary); temporary = null;
            EditorApplication.Exit(0);
        }
        catch (Exception error)
        {
            if (temporary) UnityEngine.Object.DestroyImmediate(temporary);
            Debug.LogException(error); EditorApplication.Exit(1);
        }
    }
}

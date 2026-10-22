// Adapted from TheiaMeshReference.cs, SHA-256
// 0ccce450f462f00817365722086a4172d149c6df9bb9291766f510e1f106c67e.
// Separate collector preserves the original qualified FBX reference contract.
// Offline Editor-only reference collector. Stage in an ignored scratch project;
// never add this file or imported source scripts to the runtime asset pipeline.
using System;
using System.Collections.Generic;
using System.IO;
using System.Security.Cryptography;
using Unity.Collections;
using UnityEditor;
using UnityEngine;
using UnityEngine.Rendering;

public static class TheiaSerializedMeshReference
{
    [Serializable] public class Input
    {
        public string path, guid, sha256, meta_sha256;
    }
    [Serializable] public class Request
    {
        public int schema;
        public string archive_sha256, collector_sha256, unity_version;
        public Input[] models;
    }
    [Serializable] public class Submesh
    {
        public int topology;
        public int[] indices;
    }
    [Serializable] public class Geometry
    {
        public string file_id, name;
        public bool is_readable;
        public Vector3 bounds_center, bounds_size;
        public float[] positions;
        public Submesh[] submeshes;
    }
    [Serializable] public class Node
    {
        public string file_id, parent_id, name, mesh_file_id;
        public float[] local_matrix, world_matrix;
    }
    [Serializable] public class Model
    {
        public string path, guid, sha256, meta_sha256;
        public Geometry[] meshes;
        public Node[] nodes;
    }
    [Serializable] public class Report
    {
        public int schema = 1;
        public string unity_version, platform, request_sha256, collector_sha256;
        public Model[] models;
    }

    static string Hash(string path)
    {
        using (var hash = SHA256.Create())
        using (var stream = File.OpenRead(path))
            return BitConverter.ToString(hash.ComputeHash(stream)).Replace("-", "").ToLowerInvariant();
    }

    static string Identifier(UnityEngine.Object value, string expectedGuid)
    {
        string guid;
        long id;
        if (!AssetDatabase.TryGetGUIDAndLocalFileIdentifier(value, out guid, out id) || guid != expectedGuid)
            throw new InvalidOperationException("Missing or unexpected persistent asset identity");
        return id.ToString(System.Globalization.CultureInfo.InvariantCulture);
    }

    static float[] Matrix(Matrix4x4 matrix)
    {
        var values = new float[16];
        // Explicit row-major encoding, independent of Unity's storage order.
        for (int r = 0; r < 4; ++r)
            for (int c = 0; c < 4; ++c)
                values[4 * r + c] = matrix[r, c];
        return values;
    }

    static Geometry ReadMesh(Mesh mesh, string guid)
    {
        if (mesh.vertexCount < 1 || mesh.vertexCount > 2000000 || mesh.subMeshCount < 1 || mesh.subMeshCount > 256 || mesh.blendShapeCount != 0)
            throw new InvalidOperationException("Mesh exceeds static reference budgets/profile");
        var result = new Geometry {
            file_id = Identifier(mesh, guid), name = mesh.name, is_readable = mesh.isReadable,
            bounds_center = mesh.bounds.center, bounds_size = mesh.bounds.size,
            positions = new float[3 * mesh.vertexCount], submeshes = new Submesh[mesh.subMeshCount]
        };
        // Editor API bypasses isReadable; neither importer nor mesh is changed.
        using (var snapshot = MeshUtility.AcquireReadOnlyMeshData(mesh))
        {
            var data = snapshot[0];
            using (var positions = new NativeArray<Vector3>(data.vertexCount, Allocator.Temp))
            {
                data.GetVertices(positions);
                for (int i = 0; i < positions.Length; ++i)
                {
                    result.positions[3 * i] = positions[i].x;
                    result.positions[3 * i + 1] = positions[i].y;
                    result.positions[3 * i + 2] = positions[i].z;
                }
            }
            long total = 0;
            for (int i = 0; i < data.subMeshCount; ++i)
            {
                var descriptor = data.GetSubMesh(i);
                total += descriptor.indexCount;
                if (descriptor.topology != MeshTopology.Triangles || descriptor.indexCount < 3 || descriptor.indexCount % 3 != 0 || total > 8000000)
                    throw new InvalidOperationException("Unsupported static submesh topology/budget");
                using (var indices = new NativeArray<int>(descriptor.indexCount, Allocator.Temp))
                {
                    data.GetIndices(indices, i, true);
                    result.submeshes[i] = new Submesh { topology = (int)descriptor.topology, indices = indices.ToArray() };
                }
            }
        }
        return result;
    }

    static Model ReadModel(Input input)
    {
        if (!input.path.StartsWith("Assets/", StringComparison.Ordinal) || !input.path.EndsWith(".asset", StringComparison.OrdinalIgnoreCase) || input.path.Contains("..") || input.path.Contains("\\"))
            throw new InvalidOperationException("Invalid staged serialized mesh path");
        var physicalPath = Path.Combine(Path.GetDirectoryName(Application.dataPath), input.path);
        if (Hash(physicalPath) != input.sha256 || Hash(physicalPath + ".meta") != input.meta_sha256 || AssetDatabase.AssetPathToGUID(input.path) != input.guid)
            throw new InvalidOperationException("Staged source hash/metadata/GUID differs");
        var meshes = new List<Geometry>();
        foreach (var asset in AssetDatabase.LoadAllAssetsAtPath(input.path))
            if (asset is Mesh) meshes.Add(ReadMesh((Mesh)asset, input.guid));
        meshes.Sort((a, b) => string.CompareOrdinal(a.file_id, b.file_id));
        if (meshes.Count < 1 || meshes.Count > 256)
            throw new InvalidOperationException("Serialized mesh asset exceeds reference budgets");
        if (Hash(physicalPath) != input.sha256 || Hash(physicalPath + ".meta") != input.meta_sha256)
            throw new InvalidOperationException("Read-only collection changed source or metadata");
        return new Model { path = input.path, guid = input.guid, sha256 = input.sha256,
            meta_sha256 = input.meta_sha256, meshes = meshes.ToArray(), nodes = new Node[0] };
    }

    static string Argument(string name)
    {
        var args = Environment.GetCommandLineArgs();
        for (int i = 0; i < args.Length - 1; ++i)
            if (args[i] == name) return Path.GetFullPath(args[i + 1]);
        throw new InvalidOperationException("Missing argument " + name);
    }

    // Unity -batchmode -nographics -projectPath ... -executeMethod TheiaSerializedMeshReference.Run
    // -theiaReferenceRequest ... -theiaReferenceOutput ... -logFile ...
    public static void Run()
    {
        try
        {
            var requestPath = Argument("-theiaReferenceRequest");
            var output = Argument("-theiaReferenceOutput");
            if (File.Exists(output) || new FileInfo(requestPath).Length > 65536)
                throw new InvalidOperationException("Use a fresh output and bounded request");
            var request = JsonUtility.FromJson<Request>(File.ReadAllText(requestPath));
            if (request.schema != 1 || request.models == null || request.models.Length < 1 || request.models.Length > 32)
                throw new InvalidOperationException("Unsupported reference request");
            if (request.unity_version != Application.unityVersion)
                throw new InvalidOperationException("Editor version differs from requested version");
            var collector = Hash(Path.Combine(Application.dataPath, "Editor/TheiaSerializedMeshReference.cs"));
            if (collector != request.collector_sha256)
                throw new InvalidOperationException("Reference collector differs from request");
            var models = new List<Model>();
            var seen = new HashSet<string>();
            foreach (var input in request.models)
            {
                if (!seen.Add(input.guid)) throw new InvalidOperationException("Duplicate requested GUID");
                models.Add(ReadModel(input));
            }
            var report = new Report {
                unity_version = Application.unityVersion, platform = Application.platform.ToString(),
                request_sha256 = Hash(requestPath), collector_sha256 = collector, models = models.ToArray()
            };
            using (var stream = new FileStream(output, FileMode.CreateNew, FileAccess.Write))
            using (var writer = new StreamWriter(stream)) writer.Write(JsonUtility.ToJson(report));
            Debug.Log("THEIA_SERIALIZED_MESH_REFERENCE_COMPLETE models=" + models.Count);
            EditorApplication.Exit(0);
        }
        catch (Exception error)
        {
            Debug.LogException(error);
            EditorApplication.Exit(1);
        }
    }
}

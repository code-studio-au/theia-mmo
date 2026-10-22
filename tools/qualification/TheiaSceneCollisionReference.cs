// Offline read-only Editor physics reference. No source scripts are staged.
using System;
using System.Collections.Generic;
using System.IO;
using System.Security.Cryptography;
using UnityEditor;
using UnityEditor.SceneManagement;
using UnityEngine;

public static class TheiaSceneCollisionReference
{
    [Serializable] public class Input { public string path, guid, sha256, meta_sha256; }
    [Serializable] public class Request {
        public int schema;
        public string archive_sha256, collector_sha256, unity_version, scene;
        public Input[] inputs;
    }
    [Serializable] public class Identity { public string guid, file_id; }
    [Serializable] public class Probe {
        public Vector3 origin, direction, point, normal;
        public float maximum, distance;
        public bool hit;
    }
    [Serializable] public class Shape {
        public string name, kind, global_id, object_id, prefab_id;
        public Identity source, mesh, terrain;
        public bool enabled, active, trigger, convex, attached_rigidbody;
        public int layer, cooking_options, capsule_direction, terrain_resolution;
        public float radius, height;
        public Vector3 center, size, lossy_scale, bounds_center, bounds_size;
        public float[] world_matrix;
        public Probe[] probes;
    }
    [Serializable] public class Report {
        public int schema = 1;
        public string unity_version, platform, request_sha256, collector_sha256, scene_guid;
        public int missing_behaviours, resolved_behaviours;
        public bool playing;
        public Shape[] colliders;
    }
    static string Hash(string path) {
        using (var hash = SHA256.Create())
        using (var stream = File.OpenRead(path))
            return BitConverter.ToString(hash.ComputeHash(stream)).Replace("-", "").ToLowerInvariant();
    }
    static string Argument(string name) {
        var args = Environment.GetCommandLineArgs();
        for (int i = 0; i < args.Length - 1; ++i)
            if (args[i] == name) return Path.GetFullPath(args[i + 1]);
        throw new InvalidOperationException("Missing argument " + name);
    }
    static Identity Id(UnityEngine.Object value) {
        if (!value) return null;
        string guid; long id;
        if (!AssetDatabase.TryGetGUIDAndLocalFileIdentifier(value, out guid, out id))
            throw new InvalidOperationException("Missing persistent asset identity");
        return new Identity { guid = guid, file_id = id.ToString(System.Globalization.CultureInfo.InvariantCulture) };
    }
    static float[] Matrix(Matrix4x4 matrix) {
        var values = new float[16];
        for (int r = 0; r < 4; ++r)
            for (int c = 0; c < 4; ++c) values[4 * r + c] = matrix[r, c];
        return values;
    }
    static void Check(Input input) {
        if (!input.path.StartsWith("Assets/", StringComparison.Ordinal) || input.path.Contains("..") || input.path.Contains("\\"))
            throw new InvalidOperationException("Invalid staged input path");
        var path = Path.Combine(Path.GetDirectoryName(Application.dataPath), input.path);
        if (Hash(path) != input.sha256 || Hash(path + ".meta") != input.meta_sha256 || AssetDatabase.AssetPathToGUID(input.path) != input.guid)
            throw new InvalidOperationException("Read-only scene input/metadata/GUID differs: " + input.path);
    }
    static Shape Read(Collider collider) {
        var global = GlobalObjectId.GetGlobalObjectIdSlow(collider);
        var bounds = collider.bounds;
        var result = new Shape {
            name = collider.name, kind = collider.GetType().Name, global_id = global.ToString(),
            object_id = global.targetObjectId.ToString(), prefab_id = global.targetPrefabId.ToString(),
            source = Id(PrefabUtility.GetCorrespondingObjectFromOriginalSource(collider)),
            enabled = collider.enabled, active = collider.gameObject.activeInHierarchy,
            trigger = collider.isTrigger, attached_rigidbody = collider.attachedRigidbody != null,
            layer = collider.gameObject.layer, lossy_scale = collider.transform.lossyScale,
            world_matrix = Matrix(collider.transform.localToWorldMatrix),
            bounds_center = bounds.center, bounds_size = bounds.size
        };
        if (collider is MeshCollider mesh) {
            result.mesh = Id(mesh.sharedMesh); result.convex = mesh.convex;
            result.cooking_options = (int)mesh.cookingOptions;
        } else if (collider is BoxCollider box) {
            result.center = box.center; result.size = box.size;
        } else if (collider is CapsuleCollider capsule) {
            result.center = capsule.center; result.radius = capsule.radius;
            result.height = capsule.height; result.capsule_direction = capsule.direction;
        } else if (collider is TerrainCollider terrain) {
            result.terrain = Id(terrain.terrainData);
            result.terrain_resolution = terrain.terrainData.heightmapResolution;
            result.size = terrain.terrainData.size;
        } else throw new InvalidOperationException("Unqualified source collider kind " + result.kind);
        var probes = new List<Probe>();
        if (result.enabled && result.active) {
            var axes = new [] { Vector3.right, Vector3.up, Vector3.forward };
            for (int axis = 0; axis < 3; ++axis)
                foreach (var sign in new [] { -1f, 1f }) {
                    float extent = bounds.extents[axis] + 1f;
                    var origin = bounds.center + axes[axis] * (sign * extent);
                    var direction = axes[axis] * -sign;
                    RaycastHit hit;
                    bool found = collider.Raycast(new Ray(origin, direction), out hit, extent * 2f);
                    probes.Add(new Probe { origin = origin, direction = direction, maximum = extent * 2f,
                        hit = found, distance = found ? hit.distance : 0f,
                        point = found ? hit.point : Vector3.zero, normal = found ? hit.normal : Vector3.zero });
                }
        }
        result.probes = probes.ToArray();
        return result;
    }
    public static void Run() {
        try {
            var requestPath = Argument("-theiaReferenceRequest");
            var output = Argument("-theiaReferenceOutput");
            if (File.Exists(output) || new FileInfo(requestPath).Length > 512 * 1024)
                throw new InvalidOperationException("Use a fresh output and bounded request");
            var request = JsonUtility.FromJson<Request>(File.ReadAllText(requestPath));
            var collector = Hash(Path.Combine(Application.dataPath, "Editor/TheiaSceneCollisionReference.cs"));
            if (request.schema != 1 || request.unity_version != Application.unityVersion || request.collector_sha256 != collector || request.inputs.Length < 1 || request.inputs.Length > 768 || Application.isPlaying)
                throw new InvalidOperationException("Unqualified scene reference request/version/collector");
            foreach (var input in request.inputs) Check(input);
            var scene = EditorSceneManager.OpenScene(request.scene, OpenSceneMode.Single);
            Physics.SyncTransforms();
            var shapes = new List<Shape>();
            int missing = 0, resolved = 0;
            foreach (var root in scene.GetRootGameObjects()) {
                foreach (var behaviour in root.GetComponentsInChildren<MonoBehaviour>(true)) {
                    if (behaviour) ++resolved; else ++missing;
                }
                foreach (var collider in root.GetComponentsInChildren<Collider>(true)) shapes.Add(Read(collider));
            }
            if (shapes.Count < 1 || shapes.Count > 8192 || resolved != 0)
                throw new InvalidOperationException("Scene collider budget or source-script exclusion differs");
            shapes.Sort((a, b) => string.CompareOrdinal(a.global_id, b.global_id));
            foreach (var input in request.inputs) Check(input);
            var report = new Report { unity_version = Application.unityVersion, platform = Application.platform.ToString(),
                request_sha256 = Hash(requestPath), collector_sha256 = collector,
                scene_guid = AssetDatabase.AssetPathToGUID(request.scene), playing = Application.isPlaying,
                missing_behaviours = missing, resolved_behaviours = resolved, colliders = shapes.ToArray() };
            using (var stream = new FileStream(output, FileMode.CreateNew, FileAccess.Write))
            using (var writer = new StreamWriter(stream)) writer.Write(JsonUtility.ToJson(report));
            Debug.Log("THEIA_SCENE_COLLISION_REFERENCE_COMPLETE colliders=" + shapes.Count);
            EditorApplication.Exit(0);
        } catch (Exception error) { Debug.LogException(error); EditorApplication.Exit(1); }
    }
}

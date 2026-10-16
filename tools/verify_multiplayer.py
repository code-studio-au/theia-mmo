#!/usr/bin/env python3
"""Run an actual server and two rendered clients on the caller's display.

Use a private native compositor for headless qualification. Requires Pillow.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time

from PIL import Image


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(output, binaries, references=None, terrain=None, player_assets=None, collision=False):
    assert not (references and terrain), "Use separate reference or terrain scenes"
    assert not player_assets or (terrain and not references), "Player assets require terrain mode"
    assert not collision or terrain, "Static collision smoke requires terrain mode"
    suffix = ".exe" if os.name == "nt" else ""
    server_binary = binaries / f"theia-server{suffix}"
    client_binary = binaries / f"theia-client{suffix}"
    output.mkdir(parents=True, exist_ok=True)
    proof = output / "verification.json"
    proof.unlink(missing_ok=True)
    certificate = output / "server-cert.der"
    certificate.unlink(missing_ok=True)
    client_scene = (["--reference-scene", str(references)] if references else
                    ["--terrain-world", str(terrain)] if terrain else [])
    if player_assets:
        client_scene.extend(["--player-assets", str(player_assets)])
    server_scene = ["--terrain-world", str(terrain)] if terrain else []
    processes, logs = [], []
    try:
        log = (output / "server.log").open("w")
        logs.append(log)
        server = subprocess.Popen(
            [str(server_binary), "127.0.0.1:0", str(certificate), *server_scene],
            stdout=log, stderr=subprocess.STDOUT,
        )
        processes.append(server)
        deadline = time.monotonic() + 10
        address = None
        while time.monotonic() < deadline:
            assert server.poll() is None, "Server exited before readiness"
            match = re.search(r"listening=(\S+) public_certificate=", (output / "server.log").read_text())
            if match and certificate.exists():
                address = match.group(1)
                break
            time.sleep(0.05)
        assert address, "Server did not become ready"
        clients = []
        for name, direction in [("a", "house" if collision else "z-gaits" if player_assets else "z"), ("b", "x-gaits" if player_assets else "x")]:
            directory = output / name
            directory.mkdir(exist_ok=True)
            (directory / "network.json").unlink(missing_ok=True)
            log = (directory / "run.log").open("w")
            logs.append(log)
            process = subprocess.Popen(
                [str(client_binary), *client_scene, "--network-smoke", address,
                 str(certificate), str(directory), direction],
                stdout=log, stderr=subprocess.STDOUT,
            )
            clients.append(process)
            processes.append(process)
        for client in clients:
            assert client.wait(timeout=90) == 0, "Client failed; inspect run.log"
        assert server.poll() is None, "Server exited during verification"
        reports = [json.loads((output / name / "network.json").read_text()) for name in ("a", "b")]
        ids = {report["id"] for report in reports}
        assert len(ids) == 2, "Expected distinct server-assigned identities"
        images = []
        for name, report in zip(("a", "b"), reports):
            assert report["completed"] and not report["offline"]
            players = {player["id"]: player for player in report["final"]["players"]}
            assert set(players) == ids and report["max_players"] == 2
            assert report["final"]["tick"] > report["initial"]["tick"]
            local = report["local_render_position"]
            authoritative = players[report["id"]]["xz"]
            assert abs(local[1]-players[report["id"]]["y"]) < .01
            assert max(abs(local[0] - authoritative[0]), abs(local[2] - authoritative[1])) < .01
            remote, = report["remote_render_positions"]
            assert remote["id"] in ids - {report["id"]}
            authoritative = players[remote["id"]]["xz"]
            assert max(abs(remote["position"][0] - authoritative[0]),
                       abs(remote["position"][2] - authoritative[1])) < .05
            assert abs(remote['position'][1]-players[remote['id']]['y']) < .05
            pipelines=report['pipelines']
            assert pipelines['pending']==0 and pipelines['failed']==0 and pipelines['compiled']>0 and pipelines['clear_frames']>=3
            if terrain:
                profile_bytes=(terrain/'terrain.json').read_bytes()
                assert bytes(report['terrain_world']).hex()==hashlib.sha256(profile_bytes).hexdigest()
                profile=json.loads(profile_bytes)
                def height(x,z):
                    heights=[]
                    for a,b,c in profile['triangles']:
                        def cross(p,q,r): return (q[0]-p[0])*(r[2]-p[2])-(q[2]-p[2])*(r[0]-p[0])
                        area=cross(a,b,c); p=[x,0,z]
                        u=cross(p,b,c)/area; v=cross(a,p,c)/area; w=1-u-v
                        if min(u,v,w)>=-1e-5: heights.append(u*a[1]+v*b[1]+w*c[1])
                    assert heights,'Missing terrain under player'
                    return max(heights)
                for player in players.values():
                    assert abs(player['y']-.9-height(*player['xz'])) < .001
                own=players[report['id']]
                initial=next(p for p in report['initial']['players'] if p['id']==report['id'])
                if not collision:
                    assert abs(own['y']-initial['y']) > .02, 'Server height must change on this terrain walk'
            png = output / name / "scene.png"
            image = Image.open(png).convert("RGB")
            assert image.size == (1280, 720)
            blue = sum(b > r * 2.5 and b > g * 1.8 for r, g, b in image.getdata())
            red = sum(r > g * 2 and r > b * 2 for r, g, b in image.getdata())
            if references or player_assets:
                expected_environment = int(not player_assets)
                assert report['reference_visuals'] == {'humans': 2, 'wolves': expected_environment, 'houses': expected_environment}
                assert len(image.getcolors(image.width * image.height)) > 256
            else:
                assert blue > 100 and red > 100, f"Missing pawn pixels: {blue=}, {red=}"
            log = (output / name / "run.log").read_text()
            for marker in ("VUID-", "Validation Error", " ERROR ", "panicked at"):
                assert marker not in log, f"Diagnostic {marker}: inspect {name}/run.log"
            if references or player_assets:
                walking = set(re.findall(r'Reference actor (\S+): Walk', log))
                assert len(walking) == 2, 'Both local and remote humans must enter walking playback'
            if player_assets:
                evidence = report['human_animation']
                assert len(evidence) == 2 and len({e['owner'] for e in evidence}) == 2
                count = 2 if collision else 3
                assert all(all(n > 0 for n in e['pose_changes'][:count]) for e in evidence), 'Humans need sampled locomotion motion'
                for gait in ['Stand', 'Walk', 'Run']:
                    expected = 1 if collision and gait == 'Run' else 2
                    assert len(set(re.findall(r'Reference actor (\S+): '+gait, log))) == expected
            if collision:
                assert profile['solids'], 'Expected authored house collision'
                for player in players.values():
                    for solid in profile['solids']:
                        dx=abs(player['xz'][0]-solid['center'][0]); dz=abs(player['xz'][1]-solid['center'][2])
                        assert dx >= solid['size'][0]/2+.35-.001 or dz >= solid['size'][2]/2+.35-.001
                if name == 'a':
                    own=players[report['id']]
                    initial_x=next(p['xz'][0] for p in report['initial']['players'] if p['id']==report['id'])
                    assert abs(own['xz'][0]-initial_x+8) < .8 and abs(own['xz'][1]+3.35) < .001
                    assert report['static_collision'] == {'camera_blocked':True,'camera_restored':True}
            validation_active = 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in log
            if "VK_LAYER_KHRONOS_validation" in os.environ.get("VK_INSTANCE_LAYERS", ""):
                assert validation_active, "Enable VK_LOADER_DEBUG=layer to prove layer activation"
            images.append({"client": name, "blue_pixels": blue, "red_pixels": red,
                           "image_sha256": sha256(png), "gpu": report["gpu"],
                           "validation_active": validation_active})
        for player_id in ids:
            poses = [next([*p["xz"],p["y"]] for p in r["final"]["players"] if p["id"] == player_id) for r in reports]
            assert max(abs(a-b) for a, b in zip(*poses)) < .05
        result = {"schema": 1, "passed": True, "distinct_ids": sorted(ids),
                  "actual_clients": 2, "images": images,
                  "client_sha256": sha256(client_binary),
                  "server_sha256": sha256(server_binary),
                  "lock_sha256": sha256(Path(__file__).resolve().parents[1] / "Cargo.lock"),
                  "validation_requested": os.environ.get("VK_INSTANCE_LAYERS", ""),
                  "reference_assets": str(references) if references else None,
                  "terrain_world": str(terrain) if terrain else None,
                  "player_assets": str(player_assets) if player_assets else None,
                  "static_collision": collision,
                  "scope": "loopback QUIC + two Bevy presentations; no capacity, texture fidelity or latency benchmark"}
        proof.write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps(result, indent=2))
    finally:
        for process in reversed(processes):
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        for log in logs:
            log.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--binaries", type=Path, default=Path("target/debug"))
    parser.add_argument("--reference-assets", type=Path)
    parser.add_argument("--terrain-world", type=Path)
    parser.add_argument("--player-assets", type=Path)
    parser.add_argument("--static-collision", action="store_true")
    args = parser.parse_args()
    run(args.output.resolve(), args.binaries.resolve(), args.reference_assets.resolve() if args.reference_assets else None, args.terrain_world.resolve() if args.terrain_world else None, args.player_assets.resolve() if args.player_assets else None, args.static_collision)

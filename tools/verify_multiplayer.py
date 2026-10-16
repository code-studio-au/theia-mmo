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


def run(output, binaries):
    suffix = ".exe" if os.name == "nt" else ""
    server_binary = binaries / f"theia-server{suffix}"
    client_binary = binaries / f"theia-client{suffix}"
    output.mkdir(parents=True, exist_ok=True)
    proof = output / "verification.json"
    proof.unlink(missing_ok=True)
    certificate = output / "server-cert.der"
    certificate.unlink(missing_ok=True)
    processes, logs = [], []
    try:
        log = (output / "server.log").open("w")
        logs.append(log)
        server = subprocess.Popen(
            [str(server_binary), "127.0.0.1:0", str(certificate)],
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
        for name, direction in [("a", "z"), ("b", "x")]:
            directory = output / name
            directory.mkdir(exist_ok=True)
            (directory / "network.json").unlink(missing_ok=True)
            log = (directory / "run.log").open("w")
            logs.append(log)
            process = subprocess.Popen(
                [str(client_binary), "--network-smoke", address,
                 str(certificate), str(directory), direction],
                stdout=log, stderr=subprocess.STDOUT,
            )
            clients.append(process)
            processes.append(process)
        for client in clients:
            assert client.wait(timeout=45) == 0, "Client failed; inspect run.log"
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
            assert max(abs(local[0] - authoritative[0]), abs(local[2] - authoritative[1])) < .01
            remote, = report["remote_render_positions"]
            assert remote["id"] in ids - {report["id"]}
            authoritative = players[remote["id"]]["xz"]
            assert max(abs(remote["position"][0] - authoritative[0]),
                       abs(remote["position"][2] - authoritative[1])) < .05
            png = output / name / "scene.png"
            image = Image.open(png).convert("RGB")
            assert image.size == (1280, 720)
            blue = sum(b > r * 2.5 and b > g * 1.8 for r, g, b in image.getdata())
            red = sum(r > g * 2 and r > b * 2 for r, g, b in image.getdata())
            assert blue > 100 and red > 100, f"Missing pawn pixels: {blue=}, {red=}"
            log = (output / name / "run.log").read_text()
            for marker in ("VUID-", "Validation Error", " ERROR ", "panicked at"):
                assert marker not in log, f"Diagnostic {marker}: inspect {name}/run.log"
            validation_active = 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in log
            if "VK_LAYER_KHRONOS_validation" in os.environ.get("VK_INSTANCE_LAYERS", ""):
                assert validation_active, "Enable VK_LOADER_DEBUG=layer to prove layer activation"
            images.append({"client": name, "blue_pixels": blue, "red_pixels": red,
                           "image_sha256": sha256(png), "gpu": report["gpu"],
                           "validation_active": validation_active})
        for player_id in ids:
            poses = [next(p["xz"] for p in r["final"]["players"] if p["id"] == player_id) for r in reports]
            assert max(abs(a-b) for a, b in zip(*poses)) < .05
        result = {"schema": 1, "passed": True, "distinct_ids": sorted(ids),
                  "actual_clients": 2, "images": images,
                  "client_sha256": sha256(client_binary),
                  "server_sha256": sha256(server_binary),
                  "lock_sha256": sha256(Path(__file__).resolve().parents[1] / "Cargo.lock"),
                  "validation_requested": os.environ.get("VK_INSTANCE_LAYERS", ""),
                  "scope": "loopback QUIC + two Bevy presentations; no capacity or latency benchmark"}
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
    args = parser.parse_args()
    run(args.output.resolve(), args.binaries.resolve())

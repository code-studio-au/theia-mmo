"""Linux process/DRM sampling for offline native qualification, not game runtime.

DRM counters follow https://docs.kernel.org/gpu/drm-usage-stats.html. Missing
counters remain unavailable. Duplicate file descriptors are counted once per
(device, drm-client-id); legacy amdgpu memory aliases never add to resident data.
"""
import json
from pathlib import Path
import re
import threading
import time


def memory_value(value, proc=False):
    match = re.fullmatch(r'(\d+)(?:\s+(KiB|MiB|kB))?', value.strip())
    if not match:
        return None
    unit = match[2]
    if unit == 'kB' and not proc:
        return None
    return int(match[1]) * {None: 1, 'KiB': 1024, 'MiB': 1024**2, 'kB': 1024}[unit]


def fields(text):
    return dict(line.split(':', 1) for line in text.splitlines() if ':' in line)


def drm_clients(fdinfos):
    clients = {}
    for text in fdinfos:
        info = {k: v.strip() for k, v in fields(text).items()}
        if 'drm-driver' not in info:
            continue
        # Without an identifier, duplicates cannot be safely combined.
        if 'drm-client-id' not in info:
            continue
        key = (info.get('drm-pdev', 'global'), info['drm-client-id'])
        memory = {}
        for tag, value in info.items():
            if re.match(r'drm-(total|shared|resident|active|purgeable|memory)-', tag):
                number = memory_value(value)
                if number is not None:
                    memory[tag] = number
        for tag, value in list(memory.items()):
            if tag.startswith('drm-memory-'):
                memory.setdefault(tag.replace('drm-memory-', 'drm-resident-', 1), value)
                del memory[tag]
        candidate = {'device': key[0], 'client_id': key[1], 'driver': info['drm-driver'], 'memory_bytes': memory}
        if key not in clients:
            clients[key] = candidate
        else:
            # Duplicated FDs can be read at slightly different times; retain the
            # maximum observed value, never sum the same client twice.
            for tag, value in memory.items():
                old = clients[key]['memory_bytes']
                old[tag] = max(value, old.get(tag, 0))
    return list(clients.values())


def process_table(proc):
    result = {}
    for directory in proc.iterdir():
        if not directory.name.isdecimal():
            continue
        try:
            text = (directory/'stat').read_text()
            end = text.rindex(')');tail = text[end+2:].split()
            result[int(directory.name)] = {'ppid': int(tail[1]), 'start_ticks': int(tail[19]),
                                           'name': text[text.index('(')+1:end]}
        except (OSError, ValueError, IndexError):
            continue
    return result


def snapshot(root_pid, proc=Path('/proc')):
    table = process_table(proc)
    descendants = {root_pid}
    while True:
        added = {pid for pid, info in table.items() if info['ppid'] in descendants} - descendants
        if not added:
            break
        descendants.update(added)
    result = []
    for pid in sorted(descendants):
        info = table.get(pid)
        if info is None or info['name'] not in ('theia-client', 'theia-server'):
            continue
        directory = proc/str(pid)
        record = {'pid': pid, **info, 'cpu_bytes': {}, 'drm': [], 'unavailable': []}
        try:
            status = fields((directory/'status').read_text())
            for name in ('VmRSS', 'VmHWM', 'VmSwap'):
                if name in status:
                    value = memory_value(status[name], proc=True)
                    if value is not None:record['cpu_bytes'][name] = value
        except OSError:
            record['unavailable'].append('status')
        try:
            smaps = fields((directory/'smaps_rollup').read_text())
            for name in ('Rss', 'Pss', 'Private_Clean', 'Private_Dirty', 'SwapPss'):
                if name in smaps:
                    value = memory_value(smaps[name], proc=True)
                    if value is not None:record['cpu_bytes'][name] = value
        except OSError:
            record['unavailable'].append('smaps_rollup')
        texts = []
        try:
            for fdinfo in (directory/'fdinfo').iterdir():
                try:texts.append(fdinfo.read_text())
                except OSError:continue
        except OSError:
            record['unavailable'].append('fdinfo')
        record['drm'] = drm_clients(texts)
        result.append(record)
    return result


class Recorder:
    def __init__(self, output, root_pid, interval=.25):
        self.output = Path(output);self.root_pid = root_pid;self.interval = interval
        self.stop_event = threading.Event();self.thread = None;self.error = None
        self.count = 0;self.peaks = {};self.started = time.monotonic()

    def __enter__(self):
        self.output.mkdir(parents=True, exist_ok=False)
        self.thread = threading.Thread(target=self.run, daemon=True);self.thread.start()
        return self

    def run(self):
        try:
            with (self.output/'samples.jsonl').open('x') as output:
                while not self.stop_event.is_set():
                    now = time.monotonic()-self.started
                    records = snapshot(self.root_pid)
                    output.write(json.dumps({'seconds': now, 'processes': records})+'\n');output.flush();self.count += 1
                    for record in records:
                        key = f"{record['pid']}:{record['start_ticks']}"
                        peak = self.peaks.setdefault(key, {'pid': record['pid'], 'name': record['name'], 'first_seen_seconds': now, 'last_seen_seconds': now, 'cpu_bytes': {}, 'drm': {}})
                        peak['last_seen_seconds'] = now
                        for name, value in record['cpu_bytes'].items():peak['cpu_bytes'][name] = max(value, peak['cpu_bytes'].get(name, 0))
                        for client in record['drm']:
                            drm_key = client['device']+'/'+client['client_id']
                            old = peak['drm'].setdefault(drm_key, {'device': client['device'], 'client_id': client['client_id'], 'driver': client['driver'], 'memory_bytes': {}})
                            for name, value in client['memory_bytes'].items():old['memory_bytes'][name] = max(value, old['memory_bytes'].get(name, 0))
                    self.stop_event.wait(self.interval)
        except BaseException as error:
            self.error = error

    def __exit__(self, kind, error, traceback):
        self.stop_event.set();self.thread.join(timeout=30)
        if self.thread.is_alive():raise TimeoutError('Process sampler did not stop')
        if self.error:raise self.error
        result = {'schema': 1, 'sample_interval_requested_seconds': self.interval, 'samples': self.count,
                  'processes': list(self.peaks.values()),
                  'scope': 'Linux sampled per-process RSS/PSS/high-water and deduplicated DRM per-client counters; sampled maxima may miss short peaks; counter categories are not additive'}
        (self.output/'summary.json').write_text(json.dumps(result, indent=2)+'\n')

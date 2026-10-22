"""Do not inflate GPU measurements by counting duplicate FDs or legacy aliases."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from process_metrics import drm_clients, memory_value


class ProcessMetricsTests(unittest.TestCase):
    def test_units_and_unavailable_values_are_distinct_from_zero(self):
        self.assertEqual(memory_value('2048'),2048)
        self.assertEqual(memory_value('2 MiB'),2*1024**2)
        self.assertEqual(memory_value('2 kB',proc=True),2048)
        self.assertIsNone(memory_value('2 kB'))
        self.assertIsNone(memory_value('unavailable'))

    def test_duplicate_fd_and_legacy_alias_deduplicate_by_device_and_client(self):
        common='drm-driver: amdgpu\ndrm-client-id: 7\ndrm-pdev: 0000:03:00.0\n'
        one=common+'drm-memory-vram: 2 MiB\ndrm-resident-vram: 3 MiB\n'
        duplicate=common+'drm-resident-vram: 4 MiB\n'
        other=one.replace('0000:03:00.0','0000:15:00.0')
        result=drm_clients([one,duplicate,other,'drm-driver: amdgpu\ndrm-resident-vram: 99 MiB\n'])
        self.assertEqual(len(result),2)
        self.assertEqual(result[0]['memory_bytes'],{'drm-resident-vram':4*1024**2})
        self.assertEqual(result[1]['memory_bytes'],{'drm-resident-vram':3*1024**2})

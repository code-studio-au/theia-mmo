"""Strict native GPU checks with one explicit, counted fullscreen resize exception.

wgpu-hal 30.0.1 itself suppresses this exact diagnostic as a resize race:
https://github.com/gfx-rs/wgpu/blob/40f4a34ebaf56f9a046231f54125ad046239d3f3/wgpu-hal/src/vulkan/instance.rs#L44-L50
The forced external validation layer can still print it directly to stderr.
"""
import re

SURFACE_RESIZE = 'VUID-VkSwapchainCreateInfoKHR-pNext-07781'


def check_gpu_diagnostics(text, *, allow_surface_resize=False):
    known = 0
    for line in text.splitlines():
        ids = set(re.findall(r'VUID-[A-Za-z0-9_-]+', line))
        if (allow_surface_resize and ids == {SURFACE_RESIZE}
                and 'vkCreateSwapchainKHR()' in line and 'imageExtent' in line):
            known += 1
            continue
        if ids or any(bad in line.lower() for bad in ['validation error', 'panicked at', 'device lost']):
            raise AssertionError('Unexpected native GPU/UI diagnostic')
    if known > 2:
        raise AssertionError('Repeated surface resize diagnostic; fullscreen did not settle')
    return known

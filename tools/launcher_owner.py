"""Observe the lifetime pipe of a launcher owner without consuming application input."""

def watch_owner(stream, stopped):
    """A private piped stdin closes even if the owning editor crashes."""
    try:
        while stream.read(4096):
            pass
    finally:
        stopped.set()

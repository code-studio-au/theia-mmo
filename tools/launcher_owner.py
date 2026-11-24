"""Observe the lifetime pipe of a launcher owner without consuming application input."""
import os


def watch_owner(stream, stopped):
    """A private piped stdin closes even if the owning editor crashes."""
    try:
        # A daemon blocked in BufferedReader.read holds its Python lock. SIGINT
        # or a child exit can then abort interpreter shutdown while the owner is
        # still alive. Reading the private descriptor keeps that lock free.
        descriptor = stream.fileno()
        while os.read(descriptor, 4096):
            pass
    except (OSError, ValueError):
        # A closed/unavailable owner pipe also requests ordinary child cleanup.
        pass
    finally:
        stopped.set()

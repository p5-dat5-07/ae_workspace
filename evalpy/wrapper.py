import evalpy

class MidiObject:

    def __init__(self, file) -> None:
        id = evalpy.load_midi(file)
        self._resource_id = id

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self.close()

    def close(self):
        evalpy.unload_object(self._resource_id)
        self._resource_id = -1


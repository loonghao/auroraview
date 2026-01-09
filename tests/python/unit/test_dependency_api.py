import threading

from gallery.backend import dependency_api


class DummyEmitter:
    def __init__(self, events, done_event):
        self._events = events
        self._done_event = done_event

    def emit(self, event_name, payload):
        self._events.append((event_name, payload, threading.get_ident()))
        if event_name == "dep:complete":
            self._done_event.set()


class DummyView:
    def __init__(self):
        self.calls = {}
        self.emitter_thread = None

    def bind_call(self, name):
        def decorator(fn):
            self.calls[name] = fn
            return fn

        return decorator

    def create_emitter(self):
        self.emitter_thread = threading.get_ident()
        return DummyEmitter(self._events, self._done_event)


class DummyInstaller:
    def install_missing(self, missing, on_progress, cancel_event=None):
        on_progress(
            {
                "type": "start",
                "package": missing[0],
                "index": 0,
                "total": len(missing),
                "message": "start",
            }
        )
        on_progress(
            {
                "type": "complete",
                "package": missing[0],
                "message": "done",
            }
        )
        return {"success": True, "installed": missing}


def test_install_dependencies_creates_emitter_on_main_thread(monkeypatch, tmp_path):
    main_thread = threading.get_ident()
    events = []
    done_event = threading.Event()

    # Prepare view dependencies
    view = DummyView()
    view._events = events
    view._done_event = done_event

    # Patch dependencies to avoid real installs
    monkeypatch.setattr(dependency_api, "DependencyInstaller", DummyInstaller)
    monkeypatch.setattr(dependency_api, "parse_requirements_from_docstring", lambda doc: ["pkgA"])
    monkeypatch.setattr(dependency_api, "get_missing_requirements", lambda reqs: reqs)
    monkeypatch.setattr(
        dependency_api,
        "get_sample_by_id",
        lambda sample_id: {
            "id": sample_id,
            "source_file": "dummy.py",
        },
    )

    sample_file = tmp_path / "dummy.py"
    sample_file.write_text('"""Requirements: pkgA"""\n')
    monkeypatch.setattr(dependency_api, "EXAMPLES_DIR", tmp_path)

    dependency_api.register_dependency_apis(view)

    result = view.calls["api.install_dependencies"](sample_id="dummy")

    assert result["ok"] is True
    assert view.emitter_thread == main_thread

    assert done_event.wait(timeout=1.0)
    assert any(evt == "dep:start" for evt, _, _ in events)
    assert any(evt == "dep:complete" for evt, _, _ in events)
    assert any(thread_id != main_thread for _, _, thread_id in events)

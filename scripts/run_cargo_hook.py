import os
import subprocess
import sys


def _sanitize_windows_path(path_value: str) -> str:
    parts = []
    for entry in path_value.split(os.pathsep):
        normalized = entry.replace('/', '\\').lower()
        if normalized.endswith('\\git\\usr\\bin'):
            continue
        parts.append(entry)
    return os.pathsep.join(parts)


def main() -> int:
    if len(sys.argv) < 2:
        print('Usage: python scripts/run_cargo_hook.py <cargo-args...>', file=sys.stderr)
        return 2

    env = os.environ.copy()
    if os.name == 'nt':
        env['PATH'] = _sanitize_windows_path(env.get('PATH', ''))

    cmd = ['cargo', *sys.argv[1:]]
    completed = subprocess.run(cmd, env=env)
    return completed.returncode


if __name__ == '__main__':
    raise SystemExit(main())

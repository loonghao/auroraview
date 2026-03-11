# -*- coding: utf-8 -*-
"""Auto bootstrap telemetry for AuroraView examples.

Python automatically imports ``sitecustomize`` during interpreter startup
(when ``site`` is enabled). Placing this file in ``examples/`` allows every
example script to opt into telemetry without touching each file.
"""

from __future__ import annotations

import atexit
import os
import sys


def _is_disabled(value):
    if not value:
        return False
    lowered = str(value).lower()
    return lowered in ("1", "true", "yes", "on")


def _to_float(value, default):
    if value is None:
        return default
    try:
        return float(value)
    except (TypeError, ValueError):
        return default


def _pick_env(*keys):
    for key in keys:
        value = os.environ.get(key)
        if value:
            return value
    return None


def _bootstrap_example_telemetry():
    if _is_disabled(os.environ.get("AURORAVIEW_EXAMPLE_TELEMETRY_DISABLED")):
        return

    sentry_dsn = _pick_env(
        "AURORAVIEW_EXAMPLE_SENTRY_DSN",
        "AURORAVIEW_GALLERY_RUST_SENTRY_DSN",
        "AURORAVIEW_SENTRY_DSN",
    )
    otlp_endpoint = _pick_env(
        "AURORAVIEW_EXAMPLE_OTLP_ENDPOINT",
        "AURORAVIEW_GALLERY_RUST_OTLP_ENDPOINT",
        "AURORAVIEW_OTLP_ENDPOINT",
    )

    # No telemetry endpoints configured: keep examples behavior unchanged.
    if not sentry_dsn and not otlp_endpoint:
        return

    try:
        from auroraview.telemetry import TelemetryConfig, init, shutdown
    except Exception:
        return

    script_name = os.path.splitext(os.path.basename(sys.argv[0] if sys.argv else "example"))[0]
    service_name = _pick_env(
        "AURORAVIEW_EXAMPLE_SERVICE_NAME",
        "AURORAVIEW_GALLERY_RUST_SERVICE_NAME",
    ) or ("auroraview-example-" + script_name)

    config = TelemetryConfig(
        service_name=service_name,
        log_level=os.environ.get("AURORAVIEW_EXAMPLE_LOG_LEVEL", "info"),
        otlp_endpoint=otlp_endpoint,
        sentry_dsn=sentry_dsn,
        sentry_environment=_pick_env(
            "AURORAVIEW_EXAMPLE_SENTRY_ENV",
            "AURORAVIEW_GALLERY_RUST_SENTRY_ENV",
            "AURORAVIEW_SENTRY_ENV",
        )
        or "examples",
        sentry_release=_pick_env(
            "AURORAVIEW_EXAMPLE_SENTRY_RELEASE",
            "AURORAVIEW_GALLERY_RUST_SENTRY_RELEASE",
            "AURORAVIEW_SENTRY_RELEASE",
        ),
        sentry_sample_rate=_to_float(
            _pick_env("AURORAVIEW_EXAMPLE_SENTRY_SAMPLE_RATE", "AURORAVIEW_SENTRY_SAMPLE_RATE"),
            1.0,
        ),
        sentry_traces_sample_rate=_to_float(
            _pick_env(
                "AURORAVIEW_EXAMPLE_SENTRY_TRACES_SAMPLE_RATE",
                "AURORAVIEW_SENTRY_TRACES_SAMPLE_RATE",
            ),
            0.2,
        ),
    )

    try:
        init(config)
        atexit.register(shutdown)
    except Exception:
        return


_bootstrap_example_telemetry()

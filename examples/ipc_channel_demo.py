#!/usr/bin/env python3
"""IPC Channel Demo - Demonstrates bidirectional JSON messaging.

This example shows how to use the IPC channel for efficient communication
between a parent AuroraView process and a child Python script.

When spawned with `spawn_ipc_channel`, this script can:
1. Send structured JSON messages to the parent
2. Receive JSON messages from the parent
3. Report progress and results

Usage:
    # From Gallery with use_channel=True
    # Or directly test with environment variable:
    # AURORAVIEW_IPC_CHANNEL=test_channel python ipc_channel_demo.py
"""

import os
import sys
import time

# Check if we're running in IPC channel mode
IPC_CHANNEL = os.environ.get("AURORAVIEW_IPC_CHANNEL")
IPC_MODE = os.environ.get("AURORAVIEW_IPC_MODE")


def main():
    print("[IPC Demo] Starting...")
    print(f"[IPC Demo] IPC_CHANNEL: {IPC_CHANNEL}")
    print(f"[IPC Demo] IPC_MODE: {IPC_MODE}")

    if IPC_MODE == "channel" and IPC_CHANNEL:
        # Running in channel mode - use IpcChannel for communication
        try:
            from auroraview.core.ipc_channel import IpcChannel, IpcChannelError

            print("[IPC Demo] Connecting to IPC channel...")

            with IpcChannel.connect() as channel:
                print(f"[IPC Demo] Connected to channel: {channel.channel_name}")

                # Send initial status
                channel.send({"type": "status", "message": "Demo started"})

                # Simulate some work with progress updates
                for i in range(1, 6):
                    progress = i * 20
                    print(f"[IPC Demo] Progress: {progress}%")
                    channel.send(
                        {
                            "type": "progress",
                            "value": progress,
                            "message": f"Processing step {i}/5",
                        }
                    )
                    time.sleep(0.5)

                # Send some structured data
                channel.send(
                    {
                        "type": "data",
                        "items": [
                            {"id": 1, "name": "Item A", "value": 100},
                            {"id": 2, "name": "Item B", "value": 200},
                            {"id": 3, "name": "Item C", "value": 300},
                        ],
                    }
                )

                # Send final result
                channel.send(
                    {
                        "type": "result",
                        "success": True,
                        "data": {
                            "total_steps": 5,
                            "duration_ms": 2500,
                            "message": "Demo completed successfully",
                        },
                    }
                )

                print("[IPC Demo] All messages sent!")

        except ImportError:
            print("[IPC Demo] ERROR: auroraview.core.ipc_channel not available")
            print("[IPC Demo] Falling back to stdout mode")
            fallback_stdout_mode()
        except IpcChannelError as e:
            print(f"[IPC Demo] ERROR: Failed to connect to IPC channel: {e}")
            print("[IPC Demo] Falling back to stdout mode")
            fallback_stdout_mode()
    else:
        # Running in pipe mode or standalone - use stdout
        print("[IPC Demo] Running in stdout mode (no IPC channel)")
        fallback_stdout_mode()


def fallback_stdout_mode():
    """Fallback to stdout-based communication."""
    import json

    print("[IPC Demo] Using stdout for output")

    for i in range(1, 6):
        progress = i * 20
        # Print JSON to stdout for parent to parse
        print(json.dumps({"type": "progress", "value": progress}))
        sys.stdout.flush()
        time.sleep(0.5)

    print(json.dumps({"type": "result", "success": True}))
    print("[IPC Demo] Done!")


if __name__ == "__main__":
    main()

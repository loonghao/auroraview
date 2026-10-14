<#
.SYNOPSIS
    Minimal non-Python AuroraView parent host.

.DESCRIPTION
    Reference implementation of the *parent* side of the AuroraView
    parent/child IPC protocol, written in PowerShell so the protocol can be
    validated without Python on either end of the socket.

    It binds a loopback TCP port, launches a child process with the
    AURORAVIEW_* environment, and walks the full dialogue:

      handshake -> hello / hello_ack
      receive   -> child:ready, child:hello
      send      -> parent:ping   (child answers child:pong)
      send      -> parent:command { command: "close" }
      receive   -> child:closing
      teardown  -> socket closes, child process exits

    In -Mode Disconnect the socket is dropped instead of sending `close`,
    which exercises the disconnect half of the protocol.

    Wire format is newline-delimited JSON; see docs/guide/child-windows.md.

.PARAMETER ChildExe
    Path to the child executable. Defaults to the parent_ipc_child example
    built by `cargo build -p auroraview-core --example parent_ipc_child`.

.PARAMETER Port
    TCP port to bind. 0 (default) picks a free port automatically.

.PARAMETER Mode
    Handshake (default) runs the full close-command dialogue.
    Disconnect drops the socket instead and waits for the child to exit.

.PARAMETER TimeoutSeconds
    Socket read timeout. Default 15.

.EXAMPLE
    ./parent_host.ps1

.EXAMPLE
    ./parent_host.ps1 -Mode Disconnect
#>
[CmdletBinding()]
param(
    [string] $ChildExe = "",
    [int] $Port = 0,
    [ValidateSet("Handshake", "Disconnect")]
    [string] $Mode = "Handshake",
    [int] $TimeoutSeconds = 15
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# --------------------------------------------------------------------------
# Helpers
# --------------------------------------------------------------------------

$script:failures = New-Object System.Collections.ArrayList

function Write-Step([string] $message) {
    Write-Host "[parent] $message"
}

function Assert-Equal([string] $what, $expected, $actual) {
    if ($expected -eq $actual) {
        Write-Host "  PASS  $what = $actual"
    } else {
        Write-Host "  FAIL  $what : expected '$expected', got '$actual'"
        [void] $script:failures.Add("$what (expected '$expected', got '$actual')")
    }
}

function Assert-True([string] $what, [bool] $condition) {
    if ($condition) {
        Write-Host "  PASS  $what"
    } else {
        Write-Host "  FAIL  $what"
        [void] $script:failures.Add($what)
    }
}

function Receive-Frame($reader) {
    # Returns the next frame, or $null at EOF. Blank lines are skipped and a
    # leading UTF-8 BOM is stripped, because a peer may emit one.
    while ($true) {
        $line = $reader.ReadLine()
        if ($null -eq $line) { return $null }
        $trimmed = $line.Trim().TrimStart([char] 0xFEFF)
        if ($trimmed.Length -eq 0) { continue }
        return ($trimmed | ConvertFrom-Json)
    }
}

function Receive-Event($reader) {
    # Control frames (error, pong) carry no `event` name. Skip them so the
    # dialogue only sees application events.
    while ($true) {
        $frame = Receive-Frame $reader
        if ($null -eq $frame) { return $null }
        if ($null -ne (Get-Member -InputObject $frame -Name "event" -MemberType Properties)) {
            return $frame
        }
        Write-Host "  SKIP  control frame: type=$($frame.type) code=$($frame.code)"
    }
}

function Send-Frame($writer, [string] $json) {
    # One LF-terminated frame. StreamWriter.NewLine is set to "`n" below.
    $writer.Write($json)
    $writer.Write("`n")
    $writer.Flush()
}

# --------------------------------------------------------------------------
# Locate the child binary
# --------------------------------------------------------------------------

if ([string]::IsNullOrWhiteSpace($ChildExe)) {
    $candidates = @(
        (Join-Path $PSScriptRoot "..\..\target\debug\examples\parent_ipc_child.exe"),
        (Join-Path $PSScriptRoot "..\..\target\release\examples\parent_ipc_child.exe")
    )
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) { $ChildExe = (Resolve-Path $candidate).Path; break }
    }
}

if ([string]::IsNullOrWhiteSpace($ChildExe) -or -not (Test-Path $ChildExe)) {
    Write-Error "Child executable not found. Build it first: cargo build -p auroraview-core --example parent_ipc_child"
}

Write-Step "child : $ChildExe"
Write-Step "mode  : $Mode"

# --------------------------------------------------------------------------
# Bind the listener, then launch the child
# --------------------------------------------------------------------------

$listener = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Loopback, $Port)
$listener.Start()
$Port = ([System.Net.IPEndPoint] $listener.LocalEndpoint).Port
Write-Step "listening on 127.0.0.1:$Port"

$env:AURORAVIEW_PARENT_ID = "powershell-host"
$env:AURORAVIEW_PARENT_PORT = "$Port"
$env:AURORAVIEW_CHILD_ID = "child-1"
$env:AURORAVIEW_EXAMPLE_NAME = "parent_ipc_child"
if ($Mode -eq "Disconnect") {
    $env:AURORAVIEW_CHILD_EXIT_ON_DISCONNECT = "1"
} else {
    $env:AURORAVIEW_CHILD_EXIT_ON_DISCONNECT = "0"
}

# Launch via ProcessStartInfo rather than `Start-Process -PassThru`: the latter
# does not reliably populate ExitCode when the streams are redirected, and this
# script asserts on the child's exit code.
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $ChildExe
$psi.UseShellExecute = $false
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
foreach ($name in @(
    "AURORAVIEW_PARENT_ID",
    "AURORAVIEW_PARENT_PORT",
    "AURORAVIEW_CHILD_ID",
    "AURORAVIEW_EXAMPLE_NAME",
    "AURORAVIEW_CHILD_EXIT_ON_DISCONNECT"
)) {
    $psi.EnvironmentVariables[$name] = [Environment]::GetEnvironmentVariable($name)
}

$child = [System.Diagnostics.Process]::Start($psi)
Write-Step "child pid $($child.Id)"

# Drain both pipes concurrently: a child that writes more than the pipe buffer
# would otherwise block forever and never reach its exit.
$stdoutTask = $child.StandardOutput.ReadToEndAsync()
$stderrTask = $child.StandardError.ReadToEndAsync()

# --------------------------------------------------------------------------
# Dialogue
# --------------------------------------------------------------------------

$client = $null
$reader = $null
$writer = $null

try {
    $acceptTask = $listener.AcceptTcpClientAsync()
    if (-not $acceptTask.Wait([TimeSpan]::FromSeconds($TimeoutSeconds))) {
        throw "timed out waiting for the child to connect"
    }
    $client = $acceptTask.Result
    $client.ReceiveTimeout = $TimeoutSeconds * 1000
    $client.SendTimeout = $TimeoutSeconds * 1000

    $stream = $client.GetStream()
    # `Encoding.UTF8` carries a BOM preamble. The protocol is BOM-free, so use
    # an explicitly BOM-less encoding in both directions.
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    $reader = New-Object System.IO.StreamReader($stream, $utf8NoBom)
    $writer = New-Object System.IO.StreamWriter($stream, $utf8NoBom)
    $writer.AutoFlush = $false
    $writer.NewLine = "`n"

    Write-Step "handshake"
    $hello = Receive-Frame $reader
    Assert-True "received a frame after connect" ($null -ne $hello)
    if ($null -eq $hello) { throw "no hello frame" }
    Assert-Equal "hello.type" "hello" $hello.type
    Assert-Equal "hello.protocol" "1" "$($hello.protocol)"
    Assert-Equal "hello.child_id" "child-1" $hello.child_id

    Send-Frame $writer '{"type":"hello_ack","protocol":1,"parent_id":"powershell-host","accepted":true}'
    Write-Host "  SENT  hello_ack"

    Write-Step "child announcements"
    $ready = Receive-Event $reader
    Assert-Equal "child:ready is an event" "event" $ready.type
    Assert-Equal "child:ready event name" "child:ready" $ready.event
    Assert-Equal "child:ready child_id" "child-1" $ready.child_id

    $greeting = Receive-Event $reader
    Assert-Equal "child:hello event name" "child:hello" $greeting.event
    Assert-True "child:hello carries a pid" ($null -ne $greeting.data.pid)

    Write-Step "ping / pong round trip"
    Send-Frame $writer '{"type":"event","event":"parent:ping","data":{"nonce":42}}'
    Write-Host "  SENT  parent:ping"
    $pong = Receive-Event $reader
    Assert-Equal "child:pong event name" "child:pong" $pong.event
    Assert-Equal "child:pong echoes the nonce" "42" "$($pong.data.echo.nonce)"

    if ($Mode -eq "Handshake") {
        Write-Step "close command"
        Send-Frame $writer '{"type":"event","event":"parent:command","data":{"command":"close"}}'
        Write-Host "  SENT  parent:command close"
        $closing = Receive-Event $reader
        Assert-Equal "child:closing event name" "child:closing" $closing.event
        Assert-Equal "child:closing child_id" "child-1" $closing.child_id
    } else {
        Write-Step "dropping the socket to exercise disconnect"
        $client.Close()
        $client = $null
    }
} catch {
    Write-Host "  FAIL  exception: $($_.Exception.Message)"
    [void] $script:failures.Add("exception: $($_.Exception.Message)")
} finally {
    if ($null -ne $writer) { $writer.Dispose() }
    if ($null -ne $reader) { $reader.Dispose() }
    if ($null -ne $client) { $client.Close() }
    $listener.Stop()
}

# --------------------------------------------------------------------------
# The child must exit on its own
# --------------------------------------------------------------------------

Write-Step "waiting for the child to exit"
$exited = $child.WaitForExit(([TimeSpan]::FromSeconds($TimeoutSeconds)).TotalMilliseconds)
$stdout = $stdoutTask.Result
$stderr = $stderrTask.Result

if (-not $exited) {
    Write-Host "  FAIL  child did not exit within $TimeoutSeconds s"
    [void] $script:failures.Add("child did not exit")
    $child.Kill()
} else {
    $code = $child.ExitCode
    Write-Host "  PASS  child exited with code $code"
    Assert-Equal "child exit code" "0" "$code"
}

if (-not [string]::IsNullOrWhiteSpace($stdout)) {
    Write-Step "child stdout"
    $stdout.TrimEnd() -split "`r?`n" | ForEach-Object { Write-Host "  | $_" }
}
if (-not [string]::IsNullOrWhiteSpace($stderr)) {
    Write-Step "child stderr"
    Write-Host $stderr
}

# --------------------------------------------------------------------------
# Verdict
# --------------------------------------------------------------------------

if ($script:failures.Count -gt 0) {
    Write-Host ""
    Write-Host "RESULT: FAILED ($($script:failures.Count) check(s))" -ForegroundColor Red
    $script:failures | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}

Write-Host ""
Write-Host "RESULT: PASSED ($Mode)" -ForegroundColor Green
exit 0

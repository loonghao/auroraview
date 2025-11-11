//! PyO3-facing minimal Windows WebView2 API (feature-gated)
//!
//! Exposes simple handle-based functions to create/manage an embedded WebView2
//! as a child of a given parent HWND. Designed to be used from Python directly
//! for DCC hosts running a Qt event loop.

#![cfg(all(target_os = "windows", feature = "win-webview2"))]

use pyo3::prelude::*;
use std::cell::RefCell;

use crate::platform::windows::webview2::win::WinWebView;

// Thread-local registry because WebView2 COM objects are !Send/!Sync.
thread_local! {
    static REGISTRY: RefCell<Vec<Option<WinWebView>>> = const { RefCell::new(Vec::new()) };
}

fn insert(view: WinWebView) -> u64 {
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        if let Some((idx, slot)) = reg.iter_mut().enumerate().find(|(_, s)| s.is_none()) {
            *slot = Some(view);
            return idx as u64;
        }
        reg.push(Some(view));
        reg.len() as u64 - 1
    })
}

fn with_view<F, R>(handle: u64, f: F) -> PyResult<R>
where
    F: FnOnce(&mut WinWebView) -> anyhow::Result<R>,
{
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let slot = reg
            .get_mut(handle as usize)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyIndexError, _>("invalid handle"))?;
        let view = slot
            .as_mut()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("disposed handle"))?;
        f(view).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    })
}

#[pyfunction]
pub fn win_webview2_create_embedded(
    parent_hwnd: u64,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    url: Option<&str>,
) -> PyResult<u64> {
    // Safety note: WebView2 requires STA thread with message pump (hosted by Qt).
    // We create controller under the provided parent HWND.
    let view = WinWebView::create_embedded(parent_hwnd as isize, x, y, w, h)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    if let Some(u) = url {
        view.navigate(u)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    }

    Ok(insert(view))
}

#[pyfunction]
pub fn win_webview2_set_bounds(handle: u64, x: i32, y: i32, w: i32, h: i32) -> PyResult<()> {
    with_view(handle, |v| {
        v.set_bounds(x, y, w, h);
        Ok(())
    })
}

#[pyfunction]
pub fn win_webview2_navigate(handle: u64, url: &str) -> PyResult<()> {
    with_view(handle, |v| v.navigate(url))
}

#[pyfunction]
pub fn win_webview2_eval(handle: u64, script: &str) -> PyResult<()> {
    with_view(handle, |v| v.eval(script))
}

#[pyfunction]
pub fn win_webview2_post_message(handle: u64, json: &str) -> PyResult<()> {
    with_view(handle, |v| v.post_message(json))
}

#[pyfunction]
pub fn win_webview2_dispose(handle: u64) -> PyResult<()> {
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        if let Some(slot) = reg.get_mut(handle as usize) {
            if let Some(view) = slot.take() {
                view.dispose();
            }
            return Ok(());
        }
        Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
            "invalid handle",
        ))
    })
}

#[pyfunction]
pub fn win_webview2_on_message(py: Python<'_>, handle: u64, callback: PyObject) -> PyResult<()> {
    // Retain a reference to the Python callback for use from WebView2 event
    let cb = callback.clone_ref(py);
    with_view(handle, |v| {
        v.on_message(move |json: String| {
            // Invoke Python callback on GIL
            Python::with_gil(|py| {
                let _ = cb.call1(py, (json,));
            });
        })
    })
}

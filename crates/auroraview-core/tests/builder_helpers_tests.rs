//! Regression test for the `&Arc<S>` design contract in
//! `attach_drag_drop_handler`. RFC 0015 §3.3 mandates that `capture=false`
//! must NOT clone the sink Arc; downstream RFCs (0016 / 0017) rely on this
//! invariant. This test guards against accidental ownership creep.

use std::sync::Arc;

use auroraview_core::builder::{attach_drag_drop_handler, DispatchError, DragDropIpcSink};

struct NoopSink;
impl DragDropIpcSink for NoopSink {
    fn dispatch(
        &self,
        _event_name: &str,
        _data: serde_json::Value,
    ) -> Result<(), DispatchError> {
        Ok(())
    }
}

#[test]
fn attach_drag_drop_handler_does_not_clone_sink_when_capture_false() {
    let sink: Arc<NoopSink> = Arc::new(NoopSink);
    let before = Arc::strong_count(&sink);

    let builder = wry::WebViewBuilder::new();
    let _builder = attach_drag_drop_handler(builder, false, &sink);

    assert_eq!(Arc::strong_count(&sink), before);
}

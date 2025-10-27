// Simple Rust test to verify Tao event loop works
use tao::event_loop::{EventLoop, ControlFlow};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

fn main() {
    println!("Creating event loop...");
    let event_loop = EventLoop::new();

    println!("Creating window...");
    let window = WindowBuilder::new()
        .with_title("Test Window")
        .with_inner_size(tao::dpi::LogicalSize::new(400.0, 300.0))
        .build(&event_loop)
        .expect("Failed to create window");

    println!("Creating WebView...");
    let _webview = WebViewBuilder::new()
        .with_html("<h1>Hello from Rust!</h1>")
        .build_as_child(&window)
        .expect("Failed to create WebView");

    println!("Setting window visible...");
    window.set_visible(true);

    println!("Running event loop...");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            tao::event::Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } => {
                println!("Close requested");
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });

    println!("Event loop finished");
}


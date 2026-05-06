//!
//! API Handlers for Chrome Extension APIs
//!
//! This module contains handler functions for each Chrome Extension API.
//! Each API (storage, tabs, runtime, etc.) has its own submodule.

// API handler submodules
mod action;
mod alarms;
mod commands;
mod context_menus;
mod declarative_net_request;
mod identity;
mod management;
mod notifications;
mod offscreen;
mod permissions;
mod runtime;
mod scripting;
mod side_panel;
mod storage;
mod tabs;
mod web_request;
mod windows;

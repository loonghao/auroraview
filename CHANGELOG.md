# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.0...auroraview-v0.2.1) (2025-11-01)


### Bug Fixes

* resolve CI build and deployment issues ([0af4121](https://github.com/loonghao/auroraview/commit/0af41218c12b3f431c1c555202901730eead1283))

## [0.2.0](https://github.com/loonghao/auroraview/compare/auroraview-v0.1.0...auroraview-v0.2.0) (2025-11-01)


### ⚠ BREAKING CHANGES

* WebView initialization now requires explicit backend selection

### Features

* add comprehensive testing framework and backend extension support ([5f1ede3](https://github.com/loonghao/auroraview/commit/5f1ede3888b228a80514702a0a16e34584bfc257))
* add decorations parameter to control window title bar ([a41dadc](https://github.com/loonghao/auroraview/commit/a41dadcb8f650707f90c72f5a2114857467a0d06))
* add embedded WebView integration for Maya ([98f3c6b](https://github.com/loonghao/auroraview/commit/98f3c6b9d3fbe0f655aab6128c38ffb18b91e843))
* add factory methods and tree view for better UX ([671318c](https://github.com/loonghao/auroraview/commit/671318c7a32029bc98c4dad001dd4b457eeb162d))
* add non-blocking show_async() method for DCC integration ([790a750](https://github.com/loonghao/auroraview/commit/790a750a57beb109200ec2a292e86ba155ebb74b))
* add performance optimization infrastructure and Servo evaluation ([6b89036](https://github.com/loonghao/auroraview/commit/6b8903620708933c020add120218ec3ffc606ce2))
* add thread-safe event queue for DCC integration ([77bf270](https://github.com/loonghao/auroraview/commit/77bf27036879a6482c12c1f1006a13587d075ecb))
* enhance Maya integration and backend architecture ([cbb9e86](https://github.com/loonghao/auroraview/commit/cbb9e861f67f358062fe3f9693b851099d9e2eac))
* initial project setup with comprehensive tests and CI/CD ([99ae846](https://github.com/loonghao/auroraview/commit/99ae8461d54475cb40fc6cfa851d7de9f96a7c8c))


### Bug Fixes

* add missing libgio-2.0-dev dependency for Linux builds ([ab2ab0b](https://github.com/loonghao/auroraview/commit/ab2ab0bdcbb2b269d851b4419ba375a36a73269b))
* add platform-specific allow attributes for CI lint compliance ([b324a80](https://github.com/loonghao/auroraview/commit/b324a806b0d421f97e248fb4b4ccb5d946923c1b))
* add system dependencies for CI builds ([7cff90d](https://github.com/loonghao/auroraview/commit/7cff90d809a8e01189b246c62bf98218f992c13f))
* allow event loop creation on any thread for DCC integration ([405b0c2](https://github.com/loonghao/auroraview/commit/405b0c25fa34e87eb07115ce7a1477bc4ef22df1))
* change daemon thread to False and document threading issues ([08840e9](https://github.com/loonghao/auroraview/commit/08840e91bec23ffa67ba99f890b7c02605fbed00))
* correct CI workflow YAML structure ([d048752](https://github.com/loonghao/auroraview/commit/d048752a6c928de570252ce3b65aa765b0b696d5))
* correct system dependency package name for Linux builds ([8170478](https://github.com/loonghao/auroraview/commit/8170478a30c19b1645118592591657f845554de3))
* disable manylinux and use correct Ubuntu 24.04 webkit package ([ef46e3d](https://github.com/loonghao/auroraview/commit/ef46e3d97e519ca8d1bc5d34de63a069306009a7))
* improve module loading for Maya environment ([d0a120b](https://github.com/loonghao/auroraview/commit/d0a120bca6b55e2db8cd811b6a7b3f0e19f1ef14))
* organize imports and remove unused imports in test_webview.py ([5f062b5](https://github.com/loonghao/auroraview/commit/5f062b50f2fe161dd3689383a7edc27a21aa5b4b))
* Python 3.7 compatibility and tree view initialization ([45a85fc](https://github.com/loonghao/auroraview/commit/45a85fcfae5955ae9be8f616adb3b7e19adb2141))
* remove problematic rustflags that break CI builds ([ba4bac9](https://github.com/loonghao/auroraview/commit/ba4bac9b63b62c145bc8c0f2cf156ab9f1e230df))
* remove unsupported Linux i686 build target ([5e84bbc](https://github.com/loonghao/auroraview/commit/5e84bbc7ac8a0166e7e2e0964a04cc5b419e6744))
* remove unused imports and mut variables for CI compliance ([a66bf2a](https://github.com/loonghao/auroraview/commit/a66bf2a7b95004c8c0089c1b5cfa24940970dff2))
* resolve all clippy lint errors and code formatting issues ([c3a666d](https://github.com/loonghao/auroraview/commit/c3a666df309838b162ddda6f0fcf79bed3054e19))
* resolve all Rust compiler warnings ([0b921a2](https://github.com/loonghao/auroraview/commit/0b921a269f6a3a2a89ea8593e9c9ef317f84f05f))
* resolve CI lint errors for production readiness ([d1283ba](https://github.com/loonghao/auroraview/commit/d1283ba368ee8609fae09f02d1afdc310574fcf2))
* resolve CI lint errors for production readiness ([500e34d](https://github.com/loonghao/auroraview/commit/500e34d2506fcb9771beea30d160313e3bbda6d6))
* resolve CI linting and coverage issues ([bf2a6d5](https://github.com/loonghao/auroraview/commit/bf2a6d5ff8ed76eeb125a57ab7cd6aa417bfde18))
* resolve close button bug using event loop proxy pattern ([c42c233](https://github.com/loonghao/auroraview/commit/c42c2338cf6aa15f576af4092db80f1d25315b1f))
* resolve JavaScript syntax errors in Maya outliner example ([c91647b](https://github.com/loonghao/auroraview/commit/c91647b70187ee534751b5365835fb1299f4fd1f))
* resolve Linux glib-sys and Windows architecture build errors ([fbd0933](https://github.com/loonghao/auroraview/commit/fbd0933af35462f96fb7cdcfeadf533aba78a626))
* resolve Maya freezing issue by using correct threading model ([1d60a13](https://github.com/loonghao/auroraview/commit/1d60a130f57ff58ce13839c6a72bb4a6223b2661))
* resolve thread safety issue in show_async() ([f2874da](https://github.com/loonghao/auroraview/commit/f2874daf791535839d694037d85726ccb8145bf1))
* update ci-install command to use optional-dependencies instead of dependency-groups ([1ebf39b](https://github.com/loonghao/auroraview/commit/1ebf39b83b1ec321ade75c594e394b4e6c8b234a))
* upgrade PyO3 to 0.24.2 and fix deprecated API usage ([da4541a](https://github.com/loonghao/auroraview/commit/da4541a01136f522194c761f3a6e02743ce21f41))
* use correct Ubuntu package names for GTK dependencies ([f0c619c](https://github.com/loonghao/auroraview/commit/f0c619c068dab597a5b062b80050b1a549177c9d))


### Code Refactoring

* implement modular backend architecture with native and qt support ([fd46e3d](https://github.com/loonghao/auroraview/commit/fd46e3dd4724b348c092a24b62d4d09804734677))
* migrate to PEP 735 dependency-groups following PyRustor pattern ([bd4db4e](https://github.com/loonghao/auroraview/commit/bd4db4e4185aecda8096c8f502f0ddd9fdc39ea7))
* remove unused event_loop_v2.rs ([22a4746](https://github.com/loonghao/auroraview/commit/22a4746707069b9415e06aa67d7b99009dd8a1a9))
* rename PyWebView to AuroraView ([4834842](https://github.com/loonghao/auroraview/commit/48348420f23475c1d4090286eb030d741e48161b))


### Documentation

* add action plan for user testing ([75e4322](https://github.com/loonghao/auroraview/commit/75e432247c54c9beacf1f31dad057f5ebbb4ac3d))
* add CI testing setup summary ([6dcfc7f](https://github.com/loonghao/auroraview/commit/6dcfc7fa3b270379b04f8a317b7cf63b01a7048c))
* add complete solution summary ([f2b3c7d](https://github.com/loonghao/auroraview/commit/f2b3c7d7b797c384e21f6b4f22bd874d3c2042cf))
* add comprehensive local test summary with coverage report ([002b415](https://github.com/loonghao/auroraview/commit/002b415539c69927875a6deff544a9ea4a37fad1))
* add comprehensive Maya integration summary ([90dd29f](https://github.com/loonghao/auroraview/commit/90dd29fe18a914bc078c28f663b4960571c5006c))
* add comprehensive Maya testing examples and guides ([bf268b6](https://github.com/loonghao/auroraview/commit/bf268b63be89c7eaeb3672d9d6767580d8979d9e))
* add comprehensive Maya testing guide ([d9db98b](https://github.com/loonghao/auroraview/commit/d9db98b0bacef06f40416622cefba094acde173b))
* add comprehensive testing guide with just commands ([1e70bd1](https://github.com/loonghao/auroraview/commit/1e70bd174b72ce7e2b6d786e1c4d859078653caf))
* add comprehensive threading diagnosis and fix guide ([463c10d](https://github.com/loonghao/auroraview/commit/463c10d67e287f7070e020b1a454c978bb50c039))
* add critical fix instructions for .pyd file update ([04fff27](https://github.com/loonghao/auroraview/commit/04fff276cd7a2ff438a5087d6d23a382087fac29))
* add detailed testing instructions for Maya integration ([8c077a7](https://github.com/loonghao/auroraview/commit/8c077a71f8dfe613ce7d2ea2cffd1f5dcc920f1a))
* add event loop fix documentation ([e4f200b](https://github.com/loonghao/auroraview/commit/e4f200b5337b68f299872e80f6939dd07662ba45))
* add final CI/CD fixes summary ([16196ea](https://github.com/loonghao/auroraview/commit/16196ea9ae64d88a14ddde471056facb64f7a950))
* add final summary of Maya WebView integration ([641b00e](https://github.com/loonghao/auroraview/commit/641b00e6ab73e82af73c53b71f6b4b5ff46fc3bc))
* add final threading issues summary ([d299e72](https://github.com/loonghao/auroraview/commit/d299e72dab05a24e031189820bfb97fb747b9a09))
* add fix summary documentation ([5c5fed7](https://github.com/loonghao/auroraview/commit/5c5fed7395e8bbebb6deb854323841a82d522e38))
* add Maya integration README ([0ee0aef](https://github.com/loonghao/auroraview/commit/0ee0aef41b3045511c8bcb29c941858a1fdd4fe7))
* add Maya quick start guide ([feb2cca](https://github.com/loonghao/auroraview/commit/feb2ccab1ef372da43e201806e6044220f3b27b8))
* add next steps for testing event loop fix ([86800e8](https://github.com/loonghao/auroraview/commit/86800e856e8013ea000d39c2589791c7c01d4c96))
* add rebuild instructions for event loop fix ([09f3fef](https://github.com/loonghao/auroraview/commit/09f3fefaad006015950d87662a769db42f853a51))
* add threading solution summary ([08ea57f](https://github.com/loonghao/auroraview/commit/08ea57f9905199cecbddf68e58a0f42772f6f794))
* reorganize examples with clear structure and documentation ([29198b5](https://github.com/loonghao/auroraview/commit/29198b51599a783f394358cd66ea80c158eadc9a))
* update CI fixes summary to reflect removal of i686 support ([2a8e996](https://github.com/loonghao/auroraview/commit/2a8e9968c83fb92643baf67dcda28932f045e141))
* update quick start guide with thread safety fix ([02dee4d](https://github.com/loonghao/auroraview/commit/02dee4d826d06dcea4a08e17ad672fc48300e330))
* update quick start with embedded mode recommendations ([d0b0f1f](https://github.com/loonghao/auroraview/commit/d0b0f1f990cf588003acad4169e4cfad4468486d))
* update testing instructions with event loop fix ([dee4158](https://github.com/loonghao/auroraview/commit/dee4158980d79a5a9b30885967b495af2738454b))

## [0.1.0] - 2025-10-28

### Added
- Initial release of AuroraView
- Rust-powered WebView for Python applications
- DCC (Digital Content Creation) software integration support
- PyO3 bindings with abi3 support for Python 3.7+
- WebView builder API with configuration options
- Event system for bidirectional communication between Python and JavaScript
- Support for Maya, 3ds Max, Houdini, and Blender
- Cross-platform support (Windows, macOS, Linux)
- Comprehensive test suite
- Documentation and examples

### Features
- Lightweight WebView framework (~5MB vs ~120MB for Electron)
- Fast performance with <30MB memory footprint
- Seamless DCC integration
- Modern web stack support (React, Vue, etc.)
- Type-safe Rust implementation
- Cross-platform compatibility

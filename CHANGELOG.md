# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.12](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.11...auroraview-v0.3.12) (2025-12-14)


### Features

* **pack:** add automatic Python dependency collection ([6430170](https://github.com/loonghao/auroraview/commit/64301703ce1c8624483e09bcae7520a115846611))
* **pack:** add hooks.collect support for bundling examples ([4e4a893](https://github.com/loonghao/auroraview/commit/4e4a893f046047bf9cb5bed05a6b43781d098937))
* **pack:** add standalone Python runtime bundling strategy ([906aec4](https://github.com/loonghao/auroraview/commit/906aec4389aae4fb047376b92254b6f87675139d))
* **pack:** add standalone Python runtime bundling strategy ([6c914ef](https://github.com/loonghao/auroraview/commit/6c914ef88511d9bafb640a091d83f37d48dd75ad))


### Bug Fixes

* **ci:** update gallery pack workflow for direct executable output ([9c1a27c](https://github.com/loonghao/auroraview/commit/9c1a27c0fd733535af1ace50f4609a096deada08))


### Performance Improvements

* **pack:** streaming decompression and WebView2 warmup ([f47b132](https://github.com/loonghao/auroraview/commit/f47b1322c3b902a1c9f19178bebf95958a8487cf))

## [0.3.11](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.10...auroraview-v0.3.11) (2025-12-13)


### Features

* add Gallery app with plugin system and API unification ([51307db](https://github.com/loonghao/auroraview/commit/51307dbab10a93adcffdfc6ffbea6814a913c939))


### Bug Fixes

* add cfg guards for Windows-only code paths ([458a89f](https://github.com/loonghao/auroraview/commit/458a89f3c42e936e454caf30162a36b57d20468d))
* add libxdo-dev dependency for Linux builds and fix dead_code warning ([8e3620f](https://github.com/loonghao/auroraview/commit/8e3620fcf471e19be19ae464d6af2e8a2b213bf8))
* update wry WebViewBuilder API from with_web_context to new_with_web_context ([4d30b05](https://github.com/loonghao/auroraview/commit/4d30b0530235c25f16d5cddec87d0bec02532524))


### Documentation

* add new features documentation (Gallery, System Tray, Floating Panels, Plugin System) ([87a9d2b](https://github.com/loonghao/auroraview/commit/87a9d2babad62252bbcff7cb54c37cd6d2981236))

## [0.3.10](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.9...auroraview-v0.3.10) (2025-12-11)


### Features

* add native file drop events and shell plugin enhancements ([6729a56](https://github.com/loonghao/auroraview/commit/6729a565cc0ffa0fe593b3a6708f1be42aab4c21))


### Bug Fixes

* add cfg(windows) for platform-specific imports ([93d13d4](https://github.com/loonghao/auroraview/commit/93d13d4e8f16e1174023fd69c5ce0893505faf07))
* correct dev_tools default test assertion ([800b363](https://github.com/loonghao/auroraview/commit/800b363f43a387392d2ffed6e5a61a093a55c7e2))
* init_com_sta available on all platforms ([fbdfe56](https://github.com/loonghao/auroraview/commit/fbdfe562b6de5f871a82b02a0895c42823278363))
* resolve fmt issues and add tests for builder module ([c31b417](https://github.com/loonghao/auroraview/commit/c31b417637b49c998381c87472d6a1d46949c236))


### Code Refactoring

* extract more shared WebView builder logic to auroraview-core ([e782c86](https://github.com/loonghao/auroraview/commit/e782c865f33c03aa4cb6d76c8dc1adea873ac34a))
* extract shared WebView builder logic to auroraview-core ([19a02dd](https://github.com/loonghao/auroraview/commit/19a02dd551b25dd157eda4011ae218a7e424e6cc))
* rename standalone to desktop with backward compatibility ([c43b6bb](https://github.com/loonghao/auroraview/commit/c43b6bb3611b89e3867e9349dd5b24c7eacd462a))

## [0.3.9](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.8...auroraview-v0.3.9) (2025-12-11)


### Bug Fixes

* **ci:** use correct tag format auroraview-v* for release-please ([678f0fa](https://github.com/loonghao/auroraview/commit/678f0fa128be931e4e6f4c3fa2f2f34c6752d552))


### Documentation

* add Python 3.7 Windows CLI workaround for uv/uvx ([ce92fd0](https://github.com/loonghao/auroraview/commit/ce92fd00d8726c7fa8f039b872d6dc7ba99ea993))

## [0.3.8](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.7...auroraview-v0.3.8) (2025-12-11)


### Features

* implement desktop events with plugin system integration ([f60d2c7](https://github.com/loonghao/auroraview/commit/f60d2c7c4f663ff96ba560c3ec147c333b212178))
* implement IPC emit and async callback mechanisms ([8ff92ce](https://github.com/loonghao/auroraview/commit/8ff92ce6d021200fec8c3f051a1ee7214e3bde98))


### Bug Fixes

* emit returns False when no handlers registered ([d93921d](https://github.com/loonghao/auroraview/commit/d93921dbe2ded419202bb47663e57a1769c14b35))
* emit() returns False when no handlers registered ([bfe6cc5](https://github.com/loonghao/auroraview/commit/bfe6cc5f0b1512fb2aec567bb2f3748844da4b93))
* improve timer test stability on macOS CI ([8ebb6d9](https://github.com/loonghao/auroraview/commit/8ebb6d933f24c338d6cca840a19407c81a1d0d81))
* resolve get_hwnd returning None and add URL auto-normalization ([8a82b8e](https://github.com/loonghao/auroraview/commit/8a82b8e5fef53f267131c39b37968569fe0464a0))
* use native load_url for splash screen navigation ([d13fda0](https://github.com/loonghao/auroraview/commit/d13fda0cc7e10dd2ca7f13e7d33e18ee97393202))

## [0.3.7](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.6...auroraview-v0.3.7) (2025-12-10)


### Features

* enhance test coverage and IPC benchmarks ([8a994d1](https://github.com/loonghao/auroraview/commit/8a994d1dbc04afb7e5f6497990f47dd766be8ecd))


### Bug Fixes

* remove useless format! macro in benchmark ([4f6f558](https://github.com/loonghao/auroraview/commit/4f6f5583d18bd6638819464f7b3352a25da6c325))

## [0.3.6](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.5...auroraview-v0.3.6) (2025-12-10)


### Bug Fixes

* add UTF-8 encoding declarations for Python 3.7 compatibility ([b24beb1](https://github.com/loonghao/auroraview/commit/b24beb14569ab64afa438bec4a54fe81f15364ca))

## [0.3.5](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.4...auroraview-v0.3.5) (2025-12-10)


### Bug Fixes

* add python-bindings feature to all maturin-args in CI workflows ([d79d8b8](https://github.com/loonghao/auroraview/commit/d79d8b806f7c364199bc7c417ba0f7cc51302b33))

## [0.3.4](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.3...auroraview-v0.3.4) (2025-12-10)


### Bug Fixes

* **ci:** align artifact names between build-wheels and release workflows ([dd6fd61](https://github.com/loonghao/auroraview/commit/dd6fd611977c9b2535c50f0860ac43c9495c3d8d))
* **ci:** use PySide6&gt;=6.8 for Python 3.13+ compatibility ([2aa8281](https://github.com/loonghao/auroraview/commit/2aa8281e5ca1a888b960dc7cfe773e62a8850434))


### Code Refactoring

* **ci:** unify PR and release build flows using build-wheel action ([2b0bc97](https://github.com/loonghao/auroraview/commit/2b0bc977a80f1c817bbf9cb0ec033d483d23386b))


### Documentation

* update Python version range to include 3.13 ([f5e2f63](https://github.com/loonghao/auroraview/commit/f5e2f63cf9188382e54adbbebbcab2833a35ca40))

## [0.3.3](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.2...auroraview-v0.3.3) (2025-12-09)


### Bug Fixes

* **ci:** add Python 3.13 support and fix Windows/macOS abi3 wheel builds ([0517854](https://github.com/loonghao/auroraview/commit/05178547ec5880a606686aa3db489470febc9e15))

## [0.3.2](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.1...auroraview-v0.3.2) (2025-12-09)


### Bug Fixes

* **ci:** correct PyPI artifact filtering to preserve sdist ([9ca85f1](https://github.com/loonghao/auroraview/commit/9ca85f18239f3179448c2445924af341a804c3d1))
* rustdoc bare URLs and timer test flakiness on macOS CI ([56cc05b](https://github.com/loonghao/auroraview/commit/56cc05be4237dfc0ba5a919d6aa0200a39b1a53f))

## [0.3.1](https://github.com/loonghao/auroraview/compare/auroraview-v0.3.0...auroraview-v0.3.1) (2025-12-09)


### Features

* add Python 3.7 support with separate non-abi3 builds ([bfcc0c8](https://github.com/loonghao/auroraview/commit/bfcc0c8c373d41a4587ee6f38a2bfb4914e4ac35))


### Bug Fixes

* add missing serde_json::Value import in tests ([6877c45](https://github.com/loonghao/auroraview/commit/6877c455f259e459599b5b0fbec428f4ff16a7b9))
* add Qt system dependencies for Linux CI ([c521dd0](https://github.com/loonghao/auroraview/commit/c521dd04d3aabf1b735ce6f55162abff9d7ed658))
* always show window when show() is called explicitly ([f5d07a0](https://github.com/loonghao/auroraview/commit/f5d07a04c71420ea89ec0a7f9e067bc8cdef9f1e))
* **ci:** add python-bindings feature for Qt tests build ([24080f7](https://github.com/loonghao/auroraview/commit/24080f7dcd26e4792b895e3b7e253a7138066a86))
* disable sccache for Python 3.7 builds to avoid GLIBC incompatibility ([ae895f7](https://github.com/loonghao/auroraview/commit/ae895f789825e6608c18eacd173cfd711c93e9fc))
* downgrade image to 0.24 and add Qt test dependencies ([b0e21dd](https://github.com/loonghao/auroraview/commit/b0e21dd68a8b7068057f42ff988248ca9887fab3))
* downgrade simd-json to 0.13 for Rust 1.80 compatibility ([3be5429](https://github.com/loonghao/auroraview/commit/3be542950a4feb1a1887b6d72185e9ce6cd99306))
* downgrade tao to 0.33 to avoid dlopen2 edition2024 requirement ([d8dfa15](https://github.com/loonghao/auroraview/commit/d8dfa158c0b5da693d345c0aec9f4d11be6f141e))
* ensure all type annotations are Python 3.7+ compatible ([5dc07e1](https://github.com/loonghao/auroraview/commit/5dc07e1ff0f18b5b7f055625530d643ccb607c59))
* improve test compatibility for cross-platform and CI environments ([08c37a7](https://github.com/loonghao/auroraview/commit/08c37a79c810653ce7ba544cb376791c3ea0e23e))
* only download wheel artifacts for PyPI publish, exclude CLI binaries ([bf0c0fd](https://github.com/loonghao/auroraview/commit/bf0c0fd46b71c4ce1a2ca47e855eabae30542ba3))
* pin dlopen2 to 0.8.1 to avoid edition2024 requirement ([9454d5b](https://github.com/loonghao/auroraview/commit/9454d5b7659fdb51c8bbd781a4dcf6404d6a73d3))
* Python 3.7/3.8 type annotation compatibility and timer test stability ([386367c](https://github.com/loonghao/auroraview/commit/386367c233bbf07dae51de9204c10dd7baa9e438))
* Qt placeholder tests to work in CI environments ([039cffe](https://github.com/loonghao/auroraview/commit/039cffe14d7d97e7b9200a870409c66a16a91fac))
* remove unused import and add ruff select options to pre-commit ([e0d29af](https://github.com/loonghao/auroraview/commit/e0d29af46fc48a53f9b9be73fd1b3278dcdc4759))
* remove unused sys import and add auto_show unit tests ([de413b4](https://github.com/loonghao/auroraview/commit/de413b45f10e1a58121c5fe88b4ddcf7c0269b56))
* resolve abi3-py38 conflict with Python 3.7 and CI test issues ([558df0f](https://github.com/loonghao/auroraview/commit/558df0f1e3bb318a277aaebdad7b2cc82f5f1340))
* skip WebView browser tests in CI on all platforms ([200d76c](https://github.com/loonghao/auroraview/commit/200d76c11e93f606b9206e7a1f8b7dfe44958186))
* use typing_extensions.Literal for Python 3.7 compatibility ([00519d1](https://github.com/loonghao/auroraview/commit/00519d18d7e633970ae7fb91ce8578d34341f50c))

## [0.3.0](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.22...auroraview-v0.3.0) (2025-12-08)


### ⚠ BREAKING CHANGES

* Major architecture refactoring

### Features

* add apply_qt6_dialog_optimizations function ([434236d](https://github.com/loonghao/auroraview/commit/434236d23679307d419d61a138fc849fafb6260a))
* add cross-platform PlaywrightBrowser tests with CI browser installation ([d1dbc5b](https://github.com/loonghao/auroraview/commit/d1dbc5b6eb225dffe591dce40ccaa04e102267ee))
* add cross-platform Python tests (Linux, Windows, macOS) ([016ae05](https://github.com/loonghao/auroraview/commit/016ae05174adaf4b3e60cf53aa9a8960ae7d5de0))
* add Qt5/Qt6 compatibility layer and diagnostics ([acf9fe8](https://github.com/loonghao/auroraview/commit/acf9fe846ae044fe8d332322b511eb4179faa9e6))
* add xvfb support for headless UI tests in CI ([5eaf136](https://github.com/loonghao/auroraview/commit/5eaf136100f0d929ba630a41664951095c9629b7))
* **pack:** complete MVP for auroraview-pack and auroraview-cli crates ([72811de](https://github.com/loonghao/auroraview/commit/72811de66c277e69fe0ddad92ad6b5b721d2f9e3))
* refactor architecture - migrate core modules to auroraview-core crate ([2085d94](https://github.com/loonghao/auroraview/commit/2085d94fe9a26cc25909f105d890dd360c949b7e))
* **signals:** add Qt-inspired signal-slot event system ([373b93d](https://github.com/loonghao/auroraview/commit/373b93dbf6feaaf22d5896294c3b34e72ff8e3af))
* **testing:** add PlaywrightBrowser for Playwright-based UI testing ([3988d1a](https://github.com/loonghao/auroraview/commit/3988d1aafc977f426e5f222216c9e97aef8c61c7))


### Bug Fixes

* add python-bindings feature to Linux build in CI ([8fabe2c](https://github.com/loonghao/auroraview/commit/8fabe2c25fde84191c06ebe4b0ac51a3c7e19c81))
* add url-utils to python-bindings feature and export png_bytes_to_ico ([156a4d3](https://github.com/loonghao/auroraview/commit/156a4d35f3d5851f7d0b191b4e3ec67513e54091))
* add x-release-please-version tag to version fields ([f1d4ec0](https://github.com/loonghao/auroraview/commit/f1d4ec0ff09a4941bfeae4a0d6afa76f0062ad7d))
* **ci:** correct Rust integration test names in CI configuration ([1efed3a](https://github.com/loonghao/auroraview/commit/1efed3a6edabc3091bd3e5e163222aabe4993250))
* **ci:** correct rust-toolchain action name ([d688621](https://github.com/loonghao/auroraview/commit/d6886217307bd9f7923d362a8078a545de95064e))
* correct function name in cli_utils test ([402c629](https://github.com/loonghao/auroraview/commit/402c629e366ed72b9c50d30607aadf0f59bd0f45))
* correct Qt test paths in pr-checks.yml ([4b562a6](https://github.com/loonghao/auroraview/commit/4b562a619d1da0ffffaefe658c10446e803dd30d))
* correct test file paths in noxfile.py and README ([404b17a](https://github.com/loonghao/auroraview/commit/404b17ada4cbe5d14a9813fb479086ddf69de869))
* correct wry API and image crate import in CLI ([75c00ef](https://github.com/loonghao/auroraview/commit/75c00efd6acdafd651697ba5b51e3086df01a50a))
* handle poisoned mutex gracefully to prevent panic on close ([a5a6cd3](https://github.com/loonghao/auroraview/commit/a5a6cd316566dad1b28ad9c7f8b1518e6895d8b3))
* import testing fixtures in conftest and skip UI tests in CI ([635368b](https://github.com/loonghao/auroraview/commit/635368b9b74318c6d086f23500acc87bd954613b))
* replace deprecated macos-13 runner with macos-latest cross-compile ([ebc0842](https://github.com/loonghao/auroraview/commit/ebc0842c4bc48730d72c86a56aa7865c3db0c216))
* resolve clippy warnings and CI ruff installation ([68c3e18](https://github.com/loonghao/auroraview/commit/68c3e185d69fa256ae3c8acf43c6506c82a00de5))
* resolve cross-platform compilation errors for Linux CI ([652b4fa](https://github.com/loonghao/auroraview/commit/652b4fa710d2137f3af2f154d6e20753b86bcf9e))
* resolve lint and clippy warnings ([c15ad33](https://github.com/loonghao/auroraview/commit/c15ad33230e6455a3cff37e3a16ce4db8ce72230))
* resolve unit test collection errors on Linux CI ([3fe9007](https://github.com/loonghao/auroraview/commit/3fe9007771d91fd72c390c7ca5102fa8817322b5))
* skip Browser tests on Linux CI and fix localStorage test ([df162ed](https://github.com/loonghao/auroraview/commit/df162eda8960c9b504d3be72df50430be25bf9df))
* skip Browser tests on macOS CI and fix actions/checkout version ([57fcd73](https://github.com/loonghao/auroraview/commit/57fcd732e266e69990332be33eff781a0c4903f9))
* skip PlaywrightBrowser native mode tests in CI ([cbd6e88](https://github.com/loonghao/auroraview/commit/cbd6e8811a1cfb5dc1af9c7a1fcec739b99f9304))
* unused variable in shell plugin test ([a5964ba](https://github.com/loonghao/auroraview/commit/a5964ba8d74d5b5e5461a308c379ef70dbfee27a))
* unused variable in shell plugin test ([eddde5a](https://github.com/loonghao/auroraview/commit/eddde5aa37390d7fb21749f9d3e3d76e2f29e18b))
* update bom test to match fallback pattern in JS templates ([afc10ed](https://github.com/loonghao/auroraview/commit/afc10ed605de2004ce1e44e579203ba6257f6ba7))
* update integration tests for CI compatibility ([d2ababf](https://github.com/loonghao/auroraview/commit/d2ababf242be5c45cd08ff490b448f0b08f36b84))
* use find_spec instead of import for playwright check ([5dfc9e0](https://github.com/loonghao/auroraview/commit/5dfc9e032008bcacaf75ad9e5c38fab304d709a1))
* use force=True in Locator tests to skip actionability checks ([c0dcd76](https://github.com/loonghao/auroraview/commit/c0dcd7638116dd7a9bc49d412a6e2d06613cd4b6))


### Performance Improvements

* **logging:** add conditional verbose logging for DCC environments ([004ffe1](https://github.com/loonghao/auroraview/commit/004ffe1f1b27e1854bb50c5d60bba3efea8118d6))
* **qt:** optimize window style operations and fix initialization timing ([3fe3025](https://github.com/loonghao/auroraview/commit/3fe3025caf68d4a5928529b3abb670bf65067fe3))
* **warmup:** auto-start WebView2 warmup on module import ([4f04815](https://github.com/loonghao/auroraview/commit/4f04815d06bbf00962e5bdc6ee4aac68c19f74e8))


### Code Refactoring

* clean up dead code and fix integration test imports ([39b116c](https://github.com/loonghao/auroraview/commit/39b116cbe68cf390e1530686816012506e37d997))
* replace Playwright native tests with WebView2 Browser tests ([a658808](https://github.com/loonghao/auroraview/commit/a65880878ccee04f1fff446ce12c0ccdaa94848c))
* **testing:** replace legacy testing modules with HeadlessWebView framework ([5e67fc7](https://github.com/loonghao/auroraview/commit/5e67fc7d7979263128102d8d292567a8ad425d07))
* update test files for cross-platform WebView testing ([8a393df](https://github.com/loonghao/auroraview/commit/8a393dfdfba4987c32e695828d6ce0d74d70bae1))
* update test_auroratest_browser.py for cross-platform WebView testing ([c2cfa72](https://github.com/loonghao/auroraview/commit/c2cfa726e94f40ee709e574e7f98829954dead3c))


### Documentation

* add JS-Python communication guide and improve coverage config ([89c350d](https://github.com/loonghao/auroraview/commit/89c350dd5eb268864db9c20e4032c07a6c70ca0e))
* update API examples and add llms.txt index file ([d441e69](https://github.com/loonghao/auroraview/commit/d441e696a0a552595e9254ee8356d252109d5528))

## [0.2.22](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.21...auroraview-v0.2.22) (2025-11-30)


### Features

* **dom:** add comprehensive DOM manipulation module ([20cb0fb](https://github.com/loonghao/auroraview/commit/20cb0fb2f89467e8d98cb87cb9fed750b4caead5))
* **dom:** add Rust-native DOM batch operations for high performance ([64ccb50](https://github.com/loonghao/auroraview/commit/64ccb5036bfa76581e88d0af0c5a3138d01a9df2))
* **testing:** add DOM-based testing framework ([d7e8227](https://github.com/loonghao/auroraview/commit/d7e8227a1af601d2e0a6e7b173fe6b37757cf28f))


### Bug Fixes

* improve CI test stability ([b9f7ca3](https://github.com/loonghao/auroraview/commit/b9f7ca3300d14bee9d8a0035687e132b9143a54d))
* Python 3.7 compatibility and format test file ([9295609](https://github.com/loonghao/auroraview/commit/92956097dde2d5b8499edb431fc67d6dca419fa3))
* **tests:** update tests for DOM API migration ([d55968e](https://github.com/loonghao/auroraview/commit/d55968e5c02aa82d2f9ef6270e05e7beae56ed11))


### Documentation

* deep comparison between AuroraView and PyWebView ([a958523](https://github.com/loonghao/auroraview/commit/a958523105fa16a98c0c7a597a04340a87164315))
* update comparison with comprehensive DOM API section ([ce5c878](https://github.com/loonghao/auroraview/commit/ce5c878a275bf8eceaa3a3b1c34847f7cf35e86c))

## [0.2.21](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.20...auroraview-v0.2.21) (2025-11-28)


### Features

* auto-convert relative paths to auroraview:// protocol and add always_on_top ([66fc2a1](https://github.com/loonghao/auroraview/commit/66fc2a1a4a20c50cea53168a1c1e68f639e0eeaf))

## [0.2.20](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.19...auroraview-v0.2.20) (2025-11-27)


### Features

* add window maximization when width/height is 0 ([930da84](https://github.com/loonghao/auroraview/commit/930da8408ba7b5829d9afa339b293a769c23707c))


### Documentation

* add run_standalone examples for local resource loading ([2d8f254](https://github.com/loonghao/auroraview/commit/2d8f254fcceab156a18d91ef9e61204e03715ae5))

## [0.2.19](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.18...auroraview-v0.2.19) (2025-11-27)


### Bug Fixes

* use Optional[] instead of | union syntax for Python 3.7 compatibility ([330759e](https://github.com/loonghao/auroraview/commit/330759e8c8955cb385802ace2d81f0fc07cd606e))

## [0.2.18](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.17...auroraview-v0.2.18) (2025-11-27)


### Features

* add allow_file_protocol parameter for local resource loading ([37f2bea](https://github.com/loonghao/auroraview/commit/37f2bea513e6442a91629768f2c548a77836ebf6))


### Code Refactoring

* remove debug file logging, use tracing instead ([497b6c4](https://github.com/loonghao/auroraview/commit/497b6c48394f695b2471e40290f745413b5fee1a))


### Documentation

* add custom protocol best practices and security documentation ([e520924](https://github.com/loonghao/auroraview/commit/e5209246cb57c6806248eb7f1acc776b04e85ccc))

## [0.2.17](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.16...auroraview-v0.2.17) (2025-11-26)


### Bug Fixes

* make CI tests more robust for Windows and macOS ([0f303bf](https://github.com/loonghao/auroraview/commit/0f303bf8415857dbbf3f2fb7f0dbf2ff86007555))

## [0.2.16](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.15...auroraview-v0.2.16) (2025-11-26)


### Features

* support Windows absolute paths in auroraview:// protocol ([1fd4431](https://github.com/loonghao/auroraview/commit/1fd4431776f5c0a487474130cdfa96365abe22a4))

## [0.2.15](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.14...auroraview-v0.2.15) (2025-11-25)


### Features

* add _post_eval_js_hook support for Qt integration and testing ([233332f](https://github.com/loonghao/auroraview/commit/233332f29922584eb7540b2ce04b326a8b4e800e))
* add file:// protocol support and reorganize test files ([226ce71](https://github.com/loonghao/auroraview/commit/226ce714490a8cdf28a8c02b50d4a954a384d1d2))
* add prepare_html_with_local_assets utility function ([5952ca3](https://github.com/loonghao/auroraview/commit/5952ca3a2195b735b49eb435bb6396c0f4d2eaa0))
* improve DCC mode HWND handling and reduce log noise ([ef43cfd](https://github.com/loonghao/auroraview/commit/ef43cfd49674dec0d9e4591152de7b6231741cce))
* **qt:** improve embedded WebView geometry sync and window styling ([ae091e9](https://github.com/loonghao/auroraview/commit/ae091e9dc6a01f7c6e989e8fcb83c8dcd6a004d0))


### Bug Fixes

* add __future__ annotations for Python 3.9 compatibility ([cb04732](https://github.com/loonghao/auroraview/commit/cb04732181ff902d2af927b0bdf237af189f35aa))
* add skipif decorator to test_api_injection_timing for Qt dependency ([e0c4181](https://github.com/loonghao/auroraview/commit/e0c4181c28df15c58398ad0da6e234a2822ae965))
* make ctypes and wintypes imports platform-specific ([d76f452](https://github.com/loonghao/auroraview/commit/d76f4523994fad40a19be2d641d1436bd9704392))
* use Union syntax for Python 3.7-3.9 compatibility ([4d46cb2](https://github.com/loonghao/auroraview/commit/4d46cb2e78231f025ea5e6338430cc8043b0e484))


### Performance Improvements

* add resize event throttling to maintain 60 FPS ([0b1f12d](https://github.com/loonghao/auroraview/commit/0b1f12d1a24cd9bbc382d3cc1f76c448dae0c4b1))


### Code Refactoring

* move in-block imports to module top level ([cb6cc3b](https://github.com/loonghao/auroraview/commit/cb6cc3b12fc82ebcbc5a7bea810bafd44bac27cc))
* use qtpy for QTimer import instead of direct Qt binding imports ([b2651f7](https://github.com/loonghao/auroraview/commit/b2651f7f621d18d2b7349d8a1b50f07d1944c477))

## [0.2.14](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.13...auroraview-v0.2.14) (2025-11-24)


### Features

* add always-on-top window option ([0e7de4c](https://github.com/loonghao/auroraview/commit/0e7de4cbece4cec5043eb0f69fa53dd0e7ba8069))
* add window maximization and file protocol support ([9c09855](https://github.com/loonghao/auroraview/commit/9c0985597fad4cd537c6cb6cad363c9a42a8dd74))


### Bug Fixes

* update file protocol tests to use valid HTTP URIs ([12ba42c](https://github.com/loonghao/auroraview/commit/12ba42ce80f30b8342036b67a00cf3b35f63426f))

## [0.2.13](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.12...auroraview-v0.2.13) (2025-11-24)


### Bug Fixes

* resolve macOS abi3 wheel build issues ([3fa8d78](https://github.com/loonghao/auroraview/commit/3fa8d785e14d89cb1a48b861dffa4070f827a964))

## [0.2.12](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.11...auroraview-v0.2.12) (2025-11-23)


### Bug Fixes

* resolve macOS CI cache setup hashFiles errors ([5fc5968](https://github.com/loonghao/auroraview/commit/5fc59686275a6595a946342e784e4c7b5ee7c329))


### Code Refactoring

* implement dependency injection for EventTimer backends ([9b942a3](https://github.com/loonghao/auroraview/commit/9b942a367d58f2c712787bce96b9e1485a883de8))
* simplify EventTimer by removing DCC-specific implementations ([32fa559](https://github.com/loonghao/auroraview/commit/32fa559be544fbd771d3bac99c2f89346cad5cb5))

## [0.2.11](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.10...auroraview-v0.2.11) (2025-11-22)


### Features

* optimize standalone mode with loading screen and unified event loop ([5975b79](https://github.com/loonghao/auroraview/commit/5975b790a0eb2bfc4e88dca5dc6ebedec08845c0))


### Bug Fixes

* add Cargo.lock to ensure reproducible builds ([63b2420](https://github.com/loonghao/auroraview/commit/63b2420256eb9b39b90154831f1c1ad37b479e5e))
* mock run_standalone instead of WebView in CLI tests ([584ec21](https://github.com/loonghao/auroraview/commit/584ec2196292b5e184de07a08178bbfcfdcd6343))
* update documentation examples to use ignore instead of no_run ([c3842ea](https://github.com/loonghao/auroraview/commit/c3842ea76b13e71e7b2fb8a70561fc72db0a70ce))

## [0.2.10](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.9...auroraview-v0.2.10) (2025-11-21)


### Features

* expose Rust CLI utilities to Python for enhanced functionality ([bc96ed5](https://github.com/loonghao/auroraview/commit/bc96ed55cb5d1b13b0f7c6e223179f2e025be2ea))


### Bug Fixes

* implement pure Python CLI for uvx compatibility ([96f7c94](https://github.com/loonghao/auroraview/commit/96f7c9434ae51f52bc6cc50edccc4fa00727a450))

## [0.2.9](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.8...auroraview-v0.2.9) (2025-11-21)


### Features

* migrate all Rust tests to rstest integration tests ([085e524](https://github.com/loonghao/auroraview/commit/085e52480132f773c3ebdb1e142eb3de4b397c7e))
* migrate all unit tests to rstest framework ([5d0ec35](https://github.com/loonghao/auroraview/commit/5d0ec35a8985d0d872ab5235de931ce0b72fd11d))
* migrate IPC and HTTP discovery tests to rstest integration tests ([20a6c61](https://github.com/loonghao/auroraview/commit/20a6c61a69b435b8db922338bb3b148437c4e60a))
* upgrade windows crate to 0.62 ([4667a8f](https://github.com/loonghao/auroraview/commit/4667a8f163e18e0272e0d13fabe254872d401653))


### Bug Fixes

* add conditional compilation for test imports on Windows-only tests ([7f8b6ad](https://github.com/loonghao/auroraview/commit/7f8b6adf3c36391e03518daa878021622b02712d))
* add test-helpers feature to Timer conditional compilation ([2f0d2cc](https://github.com/loonghao/auroraview/commit/2f0d2ccdc9bd7389854fa4804329694374df4861))
* correct PyList creation in batch processing ([38c2304](https://github.com/loonghao/auroraview/commit/38c23046fed50ae5e1daa96782e540081fd72436))
* remove equals sign from mDNS metadata key in test ([4a95aee](https://github.com/loonghao/auroraview/commit/4a95aee53d59617dc117f990f1b5654f9594a6e9))
* remove race condition in port availability test ([ca091f3](https://github.com/loonghao/auroraview/commit/ca091f3c9df7a121404d931c2af21388b1b84edc))
* remove unused Duration import on non-Windows platforms ([541d9b7](https://github.com/loonghao/auroraview/commit/541d9b7ab41f4872bf4be6bbffe7244cc76789f2))
* resolve unused import and add more config tests ([4a25926](https://github.com/loonghao/auroraview/commit/4a2592601c484e4d9495e68f3952e7fb7ae58319))
* update PyList API for PyO3 0.27.1 compatibility ([06afb93](https://github.com/loonghao/auroraview/commit/06afb93c6c36263c269d534bcc909436e81630c5))
* use empty PyList in ipc_batch integration test ([ff9d989](https://github.com/loonghao/auroraview/commit/ff9d989bcf31d478aec5d5abdeb304da6c7dea8d))
* use localhost.local. for mDNS hostname to comply with spec ([def1d93](https://github.com/loonghao/auroraview/commit/def1d93545fe42e01095ceb4e88b2929cd2644e4))

## [0.2.8](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.7...auroraview-v0.2.8) (2025-11-18)


### Features

* improve codecov integration with comprehensive configuration ([d3594e2](https://github.com/loonghao/auroraview/commit/d3594e247eb4e69881d31a23ffa12999641ac658))

## [0.2.7](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.6...auroraview-v0.2.7) (2025-11-17)


### Features

* add configurable context menu support ([e2551ec](https://github.com/loonghao/auroraview/commit/e2551ecf6cf19dcb628c1533ddad91d5a38c7df3))
* add custom protocol handlers with mime_guess integration ([9862870](https://github.com/loonghao/auroraview/commit/986287044948c4ff3b1f8f11ce829026bc4688be))


### Bug Fixes

* add custom protocol support to Python WebView API ([5d3322e](https://github.com/loonghao/auroraview/commit/5d3322e1724cee458d5e41a41d6c154d4f07c301))
* correct MIME types and URI handling in protocol tests ([992ecbe](https://github.com/loonghao/auroraview/commit/992ecbe51f7d614b277654aca707d4de4668dc27))
* correct URI path extraction for custom protocols ([ecaf64b](https://github.com/loonghao/auroraview/commit/ecaf64bcc43d1a96298784ad5b265e56e033b2d3))
* disable context menu using JavaScript event prevention ([4382be7](https://github.com/loonghao/auroraview/commit/4382be7b242496a52a9101af9ba963b4219fd37c))
* remove debug markers and fix failing unit tests ([f06b056](https://github.com/loonghao/auroraview/commit/f06b056be5419b4cd6fd66883515fce42abb5896))
* resolve doctest compilation errors ([efbae30](https://github.com/loonghao/auroraview/commit/efbae30b0bd1eaba7b94df30d6ce13ab561879c2))
* strengthen directory traversal protection with path canonicalization ([e17dc35](https://github.com/loonghao/auroraview/commit/e17dc3507be62535f3ccec146cbc4d6fd136896b))


### Code Refactoring

* apply js_assets to backend/native.rs and standalone.rs ([8dd3cc9](https://github.com/loonghao/auroraview/commit/8dd3cc9bd37ca521c8958a81e68b93348edc1c0d))
* complete code cleanup and add IPC metrics API ([03a7244](https://github.com/loonghao/auroraview/commit/03a7244918ac9cc86e995270e2d1769adb48e92c))
* consolidate bindings and remove legacy embedded.rs ([3875d40](https://github.com/loonghao/auroraview/commit/3875d40cf91dab699c468edf0f6df29e8f58a84f))
* extract JavaScript to separate files and apply to embedded.rs ([ebd069e](https://github.com/loonghao/auroraview/commit/ebd069ee5a9a56c6545fadb0b029a2976e77f06a))

## [0.2.6](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.5...auroraview-v0.2.6) (2025-11-17)


### Features

* add automatic event processing for Qt integration ([ac8a689](https://github.com/loonghao/auroraview/commit/ac8a68969275f6723e3ce3ff3e7b92936b94d593))

## [0.2.5](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.4...auroraview-v0.2.5) (2025-11-16)


### Features

* **timer,docs,webview:** add callback deregistration and type hints; embedded helper\n\n- EventTimer: add off_close() and off_tick() for deregistration\n- EventTimer: introduce TimerType Literal for timer backend types\n- WebView: add run_embedded() convenience helper (auto_show + auto_timer)\n- Docs: update EventTimer guide (qtpy note, semantics), add Python embedded best practices\n- Tests: add unit tests for off_close/off_tick\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([61a7e70](https://github.com/loonghao/auroraview/commit/61a7e70a65772a1d53e8324bd9d1fee1d9fddccc))
* **timer:** use qtpy for Qt QTimer backend to support PySide6/PyQt via unified API\n\n- Replace PySide2 direct import with qtpy.QtCore.QTimer\n- Keeps graceful fallback if qtpy not installed (auroraview[qt] installs it)\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([2f4c4dd](https://github.com/loonghao/auroraview/commit/2f4c4dd913bdd97bf0b269fd02c17637ece44ec2))
* upgrade pyo3 to 0.27.1 and warp to 0.4 ([7737c93](https://github.com/loonghao/auroraview/commit/7737c93b8c9e1d7bce3c2dc807c4b5f99e4502cc))


### Bug Fixes

* add delay to ensure port is set before test assertions ([811bf96](https://github.com/loonghao/auroraview/commit/811bf964826464781e72a729b426c6fe0a6b1d5c))
* resolve HTTP discovery test port binding issues ([46b0857](https://github.com/loonghao/auroraview/commit/46b08576c0e130024b6471aac182d6df9b65aefb))
* **rust,features:** gate PyO3 imports and #[pymodule] behind feature python-bindings so rust-coverage can build with --no-default-features; test gating under cfg(all(test, feature))\nci(qt): install pytest-qt, pin PySide6&lt;6.7 and enable QT_DEBUG_PLUGINS for verbose plugin diagnostics\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([81b233f](https://github.com/loonghao/auroraview/commit/81b233f576a32077898785e1e6e17fd74ab5456d))
* support zero-parameter auroraview.call handlers ([93eb389](https://github.com/loonghao/auroraview/commit/93eb389393c80074a2865d9edb10f99bb60474af))
* update tests for dependency upgrades ([129bf5b](https://github.com/loonghao/auroraview/commit/129bf5b8492c1cf066393ffa10e98ae8675166bd))
* use Arc&lt;Mutex&gt; to properly synchronize port binding ([0144f2d](https://github.com/loonghao/auroraview/commit/0144f2dfa54ee412d7eb9f904c82b5ce72f1dc89))
* use mpsc channel for proper address synchronization ([b680067](https://github.com/loonghao/auroraview/commit/b680067bab4c1b07f7c2f7cc0a3d662f3facd648))


### Code Refactoring

* drop EventBridge compatibility from Qt backend ([938d928](https://github.com/loonghao/auroraview/commit/938d92884d2f0631a6743da20617f489ef6f3a59))


### Documentation

* **badges:** add PyPI, Python versions, downloads(pepy), Codecov and PR Checks badges to README/README_zh; fix CI badge to pr-checks.yml\n\nci(coverage): ensure pytest XML coverage uploaded (essentials+qt) and rust doc-test coverage via llvm-cov\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([a1bf2e6](https://github.com/loonghao/auroraview/commit/a1bf2e6f04b034d43a9d718eb903b97eb394c59c))
* document auroraview.call parameter encoding ([7677b51](https://github.com/loonghao/auroraview/commit/7677b51ea0b60188ccb113ff01e89975cd4fc3cf))
* **maya:** qt import via qtpy; finalize NativeWebView -&gt; WebView.create migration\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([b860f83](https://github.com/loonghao/auroraview/commit/b860f835a2fd185cbb6c729d52d7f55b6c8c70f7))
* **qt:** replace PySide2 imports with qtpy across proposal/research docs for consistency\n\n- QT_INTEGRATION_PROPOSAL.md: QWidget/QDialog/QWebEngine* and QtCore -&gt; qtpy\n- RESEARCH_FLET_PYWEBVIEW.md: QWebEngineView -&gt; qtpy import\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([0099469](https://github.com/loonghao/auroraview/commit/0099469f93f9c7f317679c99c229c369ebac134c))
* **readme,maya:** update API references for qtpy + WebView.create; add run_embedded + EventTimer off_* examples\n\n- README/README_zh: add Embedded helper and deregistration samples\n- MAYA_INTEGRATION: migrate NativeWebView -&gt; WebView.create; process_events public API; fix QWidget import and HWND cast\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([07f0736](https://github.com/loonghao/auroraview/commit/07f073666d25af425093436691fd7a2789ae41b0))
* **readme:** add CI/CodeQL/release badges and quick links to CoC/Security ([f9b0c79](https://github.com/loonghao/auroraview/commit/f9b0c791d2c9f61e4f0e830a300499ff91e95aa0))
* **readme:** enrich badges (stars/downloads/activity/issues/cc/mypy/ruff/dependabot/release-please) ([803d1e6](https://github.com/loonghao/auroraview/commit/803d1e6a8ef01715b4f0947542e198f0c93cae41))
* unify API examples to WebView.create and qtpy; fix event processing references; fix pre-commit clippy flag ([44a0a51](https://github.com/loonghao/auroraview/commit/44a0a516fa604447970d141be83d5b3718790691))

## [0.2.4](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.3...auroraview-v0.2.4) (2025-11-12)


### Features

* add __init__.py to example packages and update import docs ([b9acd72](https://github.com/loonghao/auroraview/commit/b9acd7256e29fb61897110cb0d8cf9cd7aef33b8))
* add auto-ready bridge with Qt-style API ([556e84e](https://github.com/loonghao/auroraview/commit/556e84ec6b9e7f46d994c7234c4e1009170f4441))
* add Qt-style signal/slot system for robust event binding ([8f8908b](https://github.com/loonghao/auroraview/commit/8f8908b7de3fc90cb8c4e9090bce3644d2a15a7c))
* **service-discovery:** add module sources (mod, port allocator, python bindings) for CI build\n\n- Add missing service_discovery sources required by new http_discovery tests\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([bc07782](https://github.com/loonghao/auroraview/commit/bc0778256dd4be3ebd3a0f9ccd3a4bb331da70fe))


### Bug Fixes

* change doctest code blocks from text to python to prevent compilation ([cbb0c6b](https://github.com/loonghao/auroraview/commit/cbb0c6b2e400613f0ce53460741c3171107b1c9e))
* **ci, rust:** resolve pytest import error on Windows CI and silence clippy dead_code/unused warnings\n\n- Revert Qt test invocation to 'uv run python -m pytest' (fix No module named pytest)\n- Force software rendering already applied in previous commits\n- Silence Rust warnings to pass -D warnings in CI:\n  * Gate Duration imports with cfg and remove unused imports\n  * Prefix unused function params with underscore on non-Windows\n  * Add #[allow(dead_code)] for public API and platform stubs\n  * Linux platform module: #![allow(dead_code)] and import cleanup\n- Non-Windows message_pump is_window_valid marked #[allow(dead_code)]\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([24e3b7f](https://github.com/loonghao/auroraview/commit/24e3b7f76edbcc776361fbf9de89e850859c91b8))
* **ci:** silence Rust dead_code warnings and harden Qt Windows tests\n\n- timer.rs: cfg-gate should_tick to windows|test to avoid dead_code under clippy all-targets\n- message_pump.rs: add #[allow(dead_code)] to non-windows stub\n- pr-checks.yml: add WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS to disable GPU in headless CI\n- python: format with ruff to satisfy --check\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([737edae](https://github.com/loonghao/auroraview/commit/737edae4cc9d1f66d1358258edd0ea154e3bb1cc))
* handle Qt warnings on Nuke window close gracefully ([957d893](https://github.com/loonghao/auroraview/commit/957d893ef3a94b9d796d7c047a3f090f295ef061))
* implement complete window.auroraview API in initialization scripts ([293dd5c](https://github.com/loonghao/auroraview/commit/293dd5cb73764c9f49d7f59ffd14579672fe6ba8))
* improve Qt backend import error handling for Maya users ([aaf601d](https://github.com/loonghao/auroraview/commit/aaf601dc17ce9aedab08bfcba8b7567690cbf71c))
* inline AuroraViewBridge in test scripts to avoid file loading issues ([6a4ddd7](https://github.com/loonghao/auroraview/commit/6a4ddd70d79a021467c1e83b68769bd79b8eaeb1))
* prevent Nuke from hanging on exit after WebView close ([c84d0f3](https://github.com/loonghao/auroraview/commit/c84d0f3dbd315aae7250c07ecf26a556abf26d69))
* **py:** restore backwards-compat API for tests (on_event, NativeWebView, show_async) ([67bda7c](https://github.com/loonghao/auroraview/commit/67bda7c332beaf3925885abbb2a5a9d4827b0fab))
* **qt:** export _HAS_QT and AuroraViewQt; use qtpy QWebEngineSettings for backend-agnostic devtools; expose alias in qt_integration\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([ed09760](https://github.com/loonghao/auroraview/commit/ed097600b80331a030d2f311e226c76d4ea68a49))
* remove duplicate win module and add dead_code allows ([45c5c9f](https://github.com/loonghao/auroraview/commit/45c5c9fa26b770eafb7512a42f779eaf14d2d153))
* resolve all clippy warnings ([337f9c5](https://github.com/loonghao/auroraview/commit/337f9c5caabd0291b60e6f5544dc1af553eba51f))
* resolve CI lint failures and update deprecated PyO3 APIs ([eb123f8](https://github.com/loonghao/auroraview/commit/eb123f807b6f8710eb9731383e7c49ab4ac5f46b))
* resolve release workflow error and update documentation ([d061a54](https://github.com/loonghao/auroraview/commit/d061a547ce910417108bbf9887c6adbe69db5e81))
* use pytest directly instead of uv run to access built extension ([b05550e](https://github.com/loonghao/auroraview/commit/b05550eb02581e175c1222bf8d8a3df139762bfc))


### Code Refactoring

* cleanup codebase and enhance examples ([0657b3e](https://github.com/loonghao/auroraview/commit/0657b3e2084b322b8ed77b772b3364225a4fc352))
* migrate all Nuke examples to simplified API ([fcebb21](https://github.com/loonghao/auroraview/commit/fcebb21a0dcb285861e427e96cd6d5e5d8765e9a))
* rename example directories to avoid DCC namespace conflicts ([3db8d30](https://github.com/loonghao/auroraview/commit/3db8d30a33aa5dced5dccc63f2abb29bbc0eefdb))
* unify WebView API and remove compatibility layers\n\n- Remove DCC-specific factories (maya/houdini/blender), for_dcc(), process_messages(), and NativeWebView\n- Keep a single entry point: WebView.create(...) with mode=auto (parent -&gt; owner automatically)\n- Fix: expose show_async() as non-blocking helper (equiv. to show(wait=False))\n- Tests: align with unified API; ensure multiple show_async calls are idempotent\n- Docs: simplify DCC examples to rely on parent only (mode implicit)\n\nSigned-off-by: longhao &lt;hal.long@outlook.com&gt; ([120e7b6](https://github.com/loonghao/auroraview/commit/120e7b6191026a33d9828c337f1a8d636c21cb0a))


### Documentation

* add comprehensive installation guides for DCC environments ([f75f225](https://github.com/loonghao/auroraview/commit/f75f225378e4e54ef96297ab6a22e8cc8d7c5a09))
* add comprehensive Nuke IPC testing guide ([a2a311c](https://github.com/loonghao/auroraview/commit/a2a311c4c95c26a6eb298fef86155e44360b84f0))
* add simplified API guide ([743be1f](https://github.com/loonghao/auroraview/commit/743be1f532b69e447869e210f81907bc806555e4))
* add white screen troubleshooting guide and diagnostic tools ([5b55752](https://github.com/loonghao/auroraview/commit/5b55752ff8bbce02b6e21be3d240d7a188ff45a4))
* move examples to separate repository and update README ([1dfd755](https://github.com/loonghao/auroraview/commit/1dfd75546e4cc30df615a6317d96a23957999cd5))

## [0.2.3](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.2...auroraview-v0.2.3) (2025-11-01)


### Bug Fixes

* build only universal2 wheels on macOS ARM64 runners ([f4f26d3](https://github.com/loonghao/auroraview/commit/f4f26d3c386fac9d4098f5ca4e02d6a038626cdf))
* exclude Linux wheels from PyPI and update installation docs ([222f69b](https://github.com/loonghao/auroraview/commit/222f69be41704b5cb71de33b60b7a78345d10633))

## [0.2.2](https://github.com/loonghao/auroraview/compare/auroraview-v0.2.1...auroraview-v0.2.2) (2025-11-01)


### Bug Fixes

* build Linux wheels on host instead of manylinux container ([a42a2ea](https://github.com/loonghao/auroraview/commit/a42a2ea554cb178e5d34ebba6bb82ac9ff10355e))
* install system dependencies in manylinux container and add wheel build test to PR checks ([583bc1a](https://github.com/loonghao/auroraview/commit/583bc1a11ac0d651e7c92ba8a905e0f66a8fb988))
* remove --compatibility flag when manylinux is off ([32706ec](https://github.com/loonghao/auroraview/commit/32706ecc4ea46b60cc69038ff19b56eba55604a6))

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

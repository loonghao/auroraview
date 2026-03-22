"""Three.js PRD Material Demo - Real-time material preview for DCC workflows.

This example demonstrates a Three.js-based PRD (PBR-like) material preview scene.
It is designed as a lightweight reference for DCC integration scenarios where artists
need to tune surface response quickly.

Features:
- Real-time 3D preview with OrbitControls
- PRD-style defaults using MeshPhysicalMaterial
- Live controls for color, roughness, metalness, clearcoat, and emissive
- One-click reset to default material values
"""

from __future__ import annotations

from auroraview import WebView

HTML = """
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Three.js PRD Material Demo</title>
  <style>
    :root {
      --bg: #020617;
      --panel: rgba(15, 23, 42, 0.86);
      --panel-border: rgba(148, 163, 184, 0.25);
      --text: #e2e8f0;
      --sub: #94a3b8;
      --accent: #38bdf8;
    }

    * { box-sizing: border-box; }

    html, body {
      margin: 0;
      width: 100%;
      height: 100%;
      overflow: hidden;
      font-family: Inter, "Segoe UI", Roboto, sans-serif;
      background: radial-gradient(circle at top, #0f172a 0%, var(--bg) 65%);
      color: var(--text);
    }

    #app {
      position: relative;
      width: 100%;
      height: 100%;
    }

    #canvas {
      width: 100%;
      height: 100%;
      display: block;
    }

    .panel {
      position: absolute;
      top: 16px;
      right: 16px;
      width: 340px;
      max-height: calc(100% - 32px);
      overflow: auto;
      background: var(--panel);
      border: 1px solid var(--panel-border);
      border-radius: 14px;
      backdrop-filter: blur(8px);
      box-shadow: 0 18px 40px rgba(0, 0, 0, 0.35);
      padding: 14px;
    }

    .title {
      font-size: 16px;
      font-weight: 700;
      margin-bottom: 4px;
    }

    .desc {
      color: var(--sub);
      font-size: 12px;
      line-height: 1.4;
      margin-bottom: 14px;
    }

    .group {
      margin-bottom: 12px;
      padding-bottom: 10px;
      border-bottom: 1px dashed rgba(148, 163, 184, 0.2);
    }

    .group:last-child {
      border-bottom: 0;
      margin-bottom: 0;
      padding-bottom: 0;
    }

    .label-row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 5px;
      font-size: 12px;
      color: #cbd5e1;
    }

    .value {
      color: var(--accent);
      font-variant-numeric: tabular-nums;
      min-width: 46px;
      text-align: right;
    }

    input[type="range"], input[type="color"], select {
      width: 100%;
    }

    input[type="range"] {
      accent-color: #22d3ee;
    }

    input[type="color"] {
      height: 34px;
      border-radius: 8px;
      border: 1px solid rgba(148, 163, 184, 0.35);
      background: #0b1220;
      padding: 3px;
    }

    select {
      height: 34px;
      border-radius: 8px;
      border: 1px solid rgba(148, 163, 184, 0.35);
      background: #0b1220;
      color: var(--text);
      padding: 0 10px;
    }

    .actions {
      display: flex;
      gap: 8px;
      margin-top: 8px;
    }

    button {
      flex: 1;
      height: 34px;
      border-radius: 8px;
      border: 1px solid rgba(56, 189, 248, 0.4);
      background: linear-gradient(180deg, #0ea5e9, #0284c7);
      color: white;
      font-weight: 600;
      cursor: pointer;
    }

    button.secondary {
      border-color: rgba(148, 163, 184, 0.45);
      background: #1e293b;
      color: #cbd5e1;
    }

    .hint {
      margin-top: 8px;
      color: var(--sub);
      font-size: 11px;
      line-height: 1.45;
    }

    .badge {
      display: inline-block;
      margin-top: 6px;
      font-size: 11px;
      color: #bae6fd;
      border: 1px solid rgba(56, 189, 248, 0.3);
      border-radius: 999px;
      padding: 2px 8px;
      background: rgba(14, 165, 233, 0.12);
    }

    .error {
      position: absolute;
      left: 16px;
      bottom: 16px;
      max-width: 70%;
      background: rgba(127, 29, 29, 0.9);
      border: 1px solid rgba(248, 113, 113, 0.6);
      border-radius: 8px;
      padding: 10px 12px;
      color: #fecaca;
      font-size: 12px;
      line-height: 1.5;
      white-space: pre-wrap;
    }
  </style>
</head>
<body>
  <div id="app">
    <canvas id="canvas"></canvas>

    <div class="panel">
      <div class="title">Three.js PRD Material Demo</div>
      <div class="desc">
        实时预览 PRD（PBR 风格）材质参数。默认材质已内置，可一键恢复。
      </div>

      <div class="group">
        <div class="label-row">
          <span>Material Type</span>
        </div>
        <select id="materialType">
          <option value="physical">MeshPhysicalMaterial (默认)</option>
          <option value="standard">MeshStandardMaterial</option>
          <option value="phong">MeshPhongMaterial</option>
        </select>
        <div class="badge">Default: PRD Preset</div>
      </div>

      <div class="group">
        <div class="label-row">
          <span>Base Color</span>
        </div>
        <input id="baseColor" type="color" />
      </div>

      <div class="group">
        <div class="label-row"><span>Roughness</span><span id="roughnessValue" class="value"></span></div>
        <input id="roughness" type="range" min="0" max="1" step="0.01" />
      </div>

      <div class="group">
        <div class="label-row"><span>Metalness</span><span id="metalnessValue" class="value"></span></div>
        <input id="metalness" type="range" min="0" max="1" step="0.01" />
      </div>

      <div class="group">
        <div class="label-row"><span>Clearcoat</span><span id="clearcoatValue" class="value"></span></div>
        <input id="clearcoat" type="range" min="0" max="1" step="0.01" />
      </div>

      <div class="group">
        <div class="label-row"><span>Emissive Intensity</span><span id="emissiveValue" class="value"></span></div>
        <input id="emissiveIntensity" type="range" min="0" max="2" step="0.01" />
      </div>

      <div class="actions">
        <button id="reset">恢复默认材质</button>
        <button id="toggleWireframe" class="secondary">线框: 关闭</button>
      </div>

      <div class="hint">
        鼠标左键旋转、滚轮缩放、右键平移。<br />
        该示例用于 Gallery 中快速验证 three.js 材质参数联动。
      </div>
    </div>

    <div id="errorBox" class="error" style="display:none;"></div>
  </div>

  <script type="module">
    import * as THREE from 'https://unpkg.com/three@0.160.1/build/three.module.js';
    import { OrbitControls } from 'https://unpkg.com/three@0.160.1/examples/jsm/controls/OrbitControls.js';

    const DEFAULTS = {
      materialType: 'physical',
      baseColor: '#8b5cf6',
      roughness: 0.26,
      metalness: 0.58,
      clearcoat: 0.42,
      emissiveIntensity: 0.12,
      wireframe: false,
    };

    const state = { ...DEFAULTS };

    function showError(message) {
      const box = document.getElementById('errorBox');
      box.style.display = 'block';
      box.textContent = String(message);
    }

    try {
      const canvas = document.getElementById('canvas');
      const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false });
      renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
      renderer.setSize(window.innerWidth, window.innerHeight);
      renderer.toneMapping = THREE.ACESFilmicToneMapping;
      renderer.toneMappingExposure = 1.0;

      const scene = new THREE.Scene();
      scene.background = new THREE.Color(0x050b16);

      const camera = new THREE.PerspectiveCamera(45, window.innerWidth / window.innerHeight, 0.1, 200);
      camera.position.set(2.8, 1.7, 3.2);

      const controls = new OrbitControls(camera, renderer.domElement);
      controls.enableDamping = true;
      controls.target.set(0, 0.45, 0);
      controls.update();

      const hemi = new THREE.HemisphereLight(0xcce7ff, 0x1b1b28, 0.8);
      scene.add(hemi);

      const keyLight = new THREE.DirectionalLight(0xffffff, 2.4);
      keyLight.position.set(3.0, 4.2, 2.8);
      scene.add(keyLight);

      const rimLight = new THREE.DirectionalLight(0x88c9ff, 1.4);
      rimLight.position.set(-3.5, 2.0, -2.6);
      scene.add(rimLight);

      const fillLight = new THREE.PointLight(0xa78bfa, 1.1, 20);
      fillLight.position.set(0.4, 1.8, 2.2);
      scene.add(fillLight);

      const floor = new THREE.Mesh(
        new THREE.CircleGeometry(5, 96),
        new THREE.MeshStandardMaterial({ color: 0x0f172a, roughness: 0.92, metalness: 0.05 })
      );
      floor.rotation.x = -Math.PI / 2;
      floor.position.y = -0.62;
      scene.add(floor);

      const geo = new THREE.SphereGeometry(0.78, 128, 128);
      let material = null;
      const mesh = new THREE.Mesh(geo);
      mesh.position.y = 0.22;
      scene.add(mesh);

      const helperGeo = new THREE.TorusKnotGeometry(0.32, 0.08, 140, 18);
      const helper = new THREE.Mesh(
        helperGeo,
        new THREE.MeshStandardMaterial({ color: 0x22d3ee, roughness: 0.35, metalness: 0.1 })
      );
      helper.position.set(-1.1, 0.3, -0.6);
      scene.add(helper);

      function makeMaterial() {
        if (state.materialType === 'phong') {
          return new THREE.MeshPhongMaterial({
            color: state.baseColor,
            shininess: Math.round((1 - state.roughness) * 100),
            wireframe: state.wireframe,
          });
        }

        if (state.materialType === 'standard') {
          return new THREE.MeshStandardMaterial({
            color: state.baseColor,
            roughness: state.roughness,
            metalness: state.metalness,
            emissive: new THREE.Color(state.baseColor),
            emissiveIntensity: state.emissiveIntensity,
            wireframe: state.wireframe,
          });
        }

        return new THREE.MeshPhysicalMaterial({
          color: state.baseColor,
          roughness: state.roughness,
          metalness: state.metalness,
          clearcoat: state.clearcoat,
          clearcoatRoughness: Math.min(1, state.roughness + 0.15),
          sheen: 0.32,
          sheenColor: new THREE.Color(0xe2e8f0),
          emissive: new THREE.Color(state.baseColor),
          emissiveIntensity: state.emissiveIntensity,
          transmission: 0.0,
          ior: 1.45,
          wireframe: state.wireframe,
        });
      }

      function rebuildMaterial() {
        if (material) material.dispose();
        material = makeMaterial();
        mesh.material = material;
      }

      function syncUi() {
        document.getElementById('materialType').value = state.materialType;
        document.getElementById('baseColor').value = state.baseColor;

        document.getElementById('roughness').value = String(state.roughness);
        document.getElementById('metalness').value = String(state.metalness);
        document.getElementById('clearcoat').value = String(state.clearcoat);
        document.getElementById('emissiveIntensity').value = String(state.emissiveIntensity);

        document.getElementById('roughnessValue').textContent = state.roughness.toFixed(2);
        document.getElementById('metalnessValue').textContent = state.metalness.toFixed(2);
        document.getElementById('clearcoatValue').textContent = state.clearcoat.toFixed(2);
        document.getElementById('emissiveValue').textContent = state.emissiveIntensity.toFixed(2);
        document.getElementById('toggleWireframe').textContent = `线框: ${state.wireframe ? '开启' : '关闭'}`;
      }

      function bindRange(id, valueId, key) {
        const el = document.getElementById(id);
        const valueEl = document.getElementById(valueId);
        el.addEventListener('input', (ev) => {
          state[key] = Number(ev.target.value);
          valueEl.textContent = state[key].toFixed(2);
          rebuildMaterial();
        });
      }

      document.getElementById('materialType').addEventListener('change', (ev) => {
        state.materialType = ev.target.value;
        rebuildMaterial();
      });

      document.getElementById('baseColor').addEventListener('input', (ev) => {
        state.baseColor = ev.target.value;
        rebuildMaterial();
      });

      bindRange('roughness', 'roughnessValue', 'roughness');
      bindRange('metalness', 'metalnessValue', 'metalness');
      bindRange('clearcoat', 'clearcoatValue', 'clearcoat');
      bindRange('emissiveIntensity', 'emissiveValue', 'emissiveIntensity');

      document.getElementById('reset').addEventListener('click', () => {
        Object.assign(state, DEFAULTS);
        syncUi();
        rebuildMaterial();
      });

      document.getElementById('toggleWireframe').addEventListener('click', () => {
        state.wireframe = !state.wireframe;
        syncUi();
        rebuildMaterial();
      });

      rebuildMaterial();
      syncUi();

      const clock = new THREE.Clock();
      function animate() {
        const t = clock.getElapsedTime();
        helper.rotation.x = t * 0.6;
        helper.rotation.y = t * 1.1;
        mesh.rotation.y += 0.0035;

        controls.update();
        renderer.render(scene, camera);
        requestAnimationFrame(animate);
      }
      animate();

      window.addEventListener('resize', () => {
        const w = window.innerWidth;
        const h = window.innerHeight;
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
        renderer.setSize(w, h);
      });
    } catch (err) {
      showError(`Failed to initialize Three.js scene:\n${err?.stack || err}`);
    }
  </script>
</body>
</html>
"""


def main() -> None:
    """Run the Three.js PRD material demo."""
    view = WebView(
        title="Three.js PRD Material Demo",
        html=HTML,
        width=1360,
        height=860,
    )
    view.show()


if __name__ == "__main__":
    main()

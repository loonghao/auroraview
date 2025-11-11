import { useState, useEffect } from 'react';
import './App.css';

// Declare AuroraView global API
declare global {
  interface Window {
    auroraview: {
      call: (method: string, params?: any) => Promise<any>;
    };
  }
}

function App() {
  const [connected, setConnected] = useState(false);
  const [blurRadius, setBlurRadius] = useState(5);
  const [contrastFactor, setContrastFactor] = useState(1.5);
  const [preview, setPreview] = useState<string | null>(null);
  const [processing, setProcessing] = useState(false);

  useEffect(() => {
    // Listen for Photoshop connection events
    window.addEventListener('photoshop-connected', () => {
      setConnected(true);
    });

    // Listen for layer creation events
    window.addEventListener('layer-created', (e: any) => {
      console.log('Layer created:', e.detail);
    });

    // Listen for image received events
    window.addEventListener('image-received', (e: any) => {
      console.log('Image received:', e.detail);
      setPreview(e.detail.image);
    });

    // Check initial status
    checkStatus();
  }, []);

  const checkStatus = async () => {
    try {
      const status = await window.auroraview.call('get_status');
      setConnected(status.photoshop_connected);
    } catch (error) {
      console.error('Error checking status:', error);
    }
  };

  const applyBlur = async () => {
    if (!preview) {
      alert('No image loaded. Please get image from Photoshop first.');
      return;
    }

    setProcessing(true);
    try {
      const result = await window.auroraview.call('apply_filter', {
        type: 'gaussian_blur',
        radius: blurRadius,
        image: preview
      });

      if (result.preview) {
        setPreview(result.preview);
      }
    } catch (error) {
      console.error('Error applying blur:', error);
      alert('Failed to apply blur');
    } finally {
      setProcessing(false);
    }
  };

  const enhanceContrast = async () => {
    if (!preview) {
      alert('No image loaded');
      return;
    }

    setProcessing(true);
    try {
      const result = await window.auroraview.call('apply_filter', {
        type: 'enhance_contrast',
        factor: contrastFactor,
        image: preview
      });

      if (result.preview) {
        setPreview(result.preview);
      }
    } catch (error) {
      console.error('Error enhancing contrast:', error);
    } finally {
      setProcessing(false);
    }
  };

  const detectEdges = async () => {
    if (!preview) {
      alert('No image loaded');
      return;
    }

    setProcessing(true);
    try {
      const result = await window.auroraview.call('apply_filter', {
        type: 'edge_detection',
        image: preview
      });

      if (result.preview) {
        setPreview(result.preview);
      }
    } catch (error) {
      console.error('Error detecting edges:', error);
    } finally {
      setProcessing(false);
    }
  };

  const requestImage = async () => {
    try {
      await window.auroraview.call('send_to_photoshop', {
        command: 'get_active_layer_image',
        params: {}
      });
    } catch (error) {
      console.error('Error requesting image:', error);
    }
  };

  return (
    <div className="app">
      <header>
        <h1>Photoshop AI Tools</h1>
        <div className={`status ${connected ? 'connected' : 'disconnected'}`}>
          {connected ? '🟢 Connected' : '🔴 Disconnected'}
        </div>
      </header>

      <main>
        <section className="controls">
          <h2>Image Processing</h2>
          
          <button onClick={requestImage} disabled={!connected}>
            📷 Get Image from Photoshop
          </button>

          <div className="filter-group">
            <h3>Gaussian Blur</h3>
            <input
              type="range"
              min="1"
              max="20"
              value={blurRadius}
              onChange={(e) => setBlurRadius(Number(e.target.value))}
            />
            <span>Radius: {blurRadius}</span>
            <button onClick={applyBlur} disabled={processing || !preview}>
              Apply Blur
            </button>
          </div>

          <div className="filter-group">
            <h3>Enhance Contrast</h3>
            <input
              type="range"
              min="0.5"
              max="3"
              step="0.1"
              value={contrastFactor}
              onChange={(e) => setContrastFactor(Number(e.target.value))}
            />
            <span>Factor: {contrastFactor.toFixed(1)}</span>
            <button onClick={enhanceContrast} disabled={processing || !preview}>
              Enhance Contrast
            </button>
          </div>

          <div className="filter-group">
            <h3>Edge Detection</h3>
            <button onClick={detectEdges} disabled={processing || !preview}>
              Detect Edges
            </button>
          </div>
        </section>

        <section className="preview">
          <h2>Preview</h2>
          {preview ? (
            <img src={preview} alt="Preview" />
          ) : (
            <div className="placeholder">
              No image loaded
            </div>
          )}
        </section>
      </main>
    </div>
  );
}

export default App;


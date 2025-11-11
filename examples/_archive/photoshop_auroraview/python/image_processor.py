"""Image processing module using Python ecosystem (Pillow, OpenCV, NumPy)."""

import base64
import io
import logging
from typing import Dict, Any, Optional
import numpy as np

try:
    from PIL import Image, ImageFilter, ImageEnhance
    PIL_AVAILABLE = True
except ImportError:
    PIL_AVAILABLE = False
    
try:
    import cv2
    CV2_AVAILABLE = True
except ImportError:
    CV2_AVAILABLE = False

logger = logging.getLogger(__name__)


class ImageProcessor:
    """Image processing using Python libraries.
    
    This class provides various image processing operations using
    Pillow, OpenCV, and NumPy.
    """
    
    def __init__(self):
        """Initialize the image processor."""
        if not PIL_AVAILABLE:
            logger.warning("⚠️  Pillow not available. Install with: pip install Pillow")
        if not CV2_AVAILABLE:
            logger.warning("⚠️  OpenCV not available. Install with: pip install opencv-python")
            
    def base64_to_image(self, base64_str: str) -> Optional[Image.Image]:
        """Convert base64 string to PIL Image.
        
        Args:
            base64_str: Base64 encoded image string
            
        Returns:
            PIL Image object or None if conversion fails
        """
        if not PIL_AVAILABLE:
            logger.error("Pillow not available")
            return None
            
        try:
            # Remove data URL prefix if present
            if ',' in base64_str:
                base64_str = base64_str.split(',')[1]
                
            image_data = base64.b64decode(base64_str)
            return Image.open(io.BytesIO(image_data))
        except Exception as e:
            logger.error(f"Error converting base64 to image: {e}")
            return None
            
    def image_to_base64(self, image: Image.Image, format: str = 'PNG') -> str:
        """Convert PIL Image to base64 string.
        
        Args:
            image: PIL Image object
            format: Image format (PNG, JPEG, etc.)
            
        Returns:
            Base64 encoded image string
        """
        buffer = io.BytesIO()
        image.save(buffer, format=format)
        buffer.seek(0)
        return base64.b64encode(buffer.read()).decode('utf-8')
        
    def apply_gaussian_blur(self, image_data: str, radius: int = 5) -> Dict[str, Any]:
        """Apply Gaussian blur filter.
        
        Args:
            image_data: Base64 encoded image
            radius: Blur radius
            
        Returns:
            Result dict with processed image
        """
        if not PIL_AVAILABLE:
            return {"error": "Pillow not available"}
            
        try:
            img = self.base64_to_image(image_data)
            if img is None:
                return {"error": "Failed to decode image"}
                
            # Apply Gaussian blur
            blurred = img.filter(ImageFilter.GaussianBlur(radius=radius))
            
            # Convert back to base64
            result_base64 = self.image_to_base64(blurred)
            
            return {
                "status": "success",
                "preview": f"data:image/png;base64,{result_base64}",
                "operation": "gaussian_blur",
                "params": {"radius": radius}
            }
        except Exception as e:
            logger.error(f"Error applying Gaussian blur: {e}")
            return {"error": str(e)}
            
    def enhance_contrast(self, image_data: str, factor: float = 1.5) -> Dict[str, Any]:
        """Enhance image contrast.
        
        Args:
            image_data: Base64 encoded image
            factor: Contrast enhancement factor (1.0 = no change)
            
        Returns:
            Result dict with processed image
        """
        if not PIL_AVAILABLE:
            return {"error": "Pillow not available"}
            
        try:
            img = self.base64_to_image(image_data)
            if img is None:
                return {"error": "Failed to decode image"}
                
            # Enhance contrast
            enhancer = ImageEnhance.Contrast(img)
            enhanced = enhancer.enhance(factor)
            
            result_base64 = self.image_to_base64(enhanced)
            
            return {
                "status": "success",
                "preview": f"data:image/png;base64,{result_base64}",
                "operation": "enhance_contrast",
                "params": {"factor": factor}
            }
        except Exception as e:
            logger.error(f"Error enhancing contrast: {e}")
            return {"error": str(e)}
            
    def sharpen(self, image_data: str, factor: float = 2.0) -> Dict[str, Any]:
        """Sharpen image.
        
        Args:
            image_data: Base64 encoded image
            factor: Sharpness factor (1.0 = no change)
            
        Returns:
            Result dict with processed image
        """
        if not PIL_AVAILABLE:
            return {"error": "Pillow not available"}
            
        try:
            img = self.base64_to_image(image_data)
            if img is None:
                return {"error": "Failed to decode image"}
                
            # Sharpen
            enhancer = ImageEnhance.Sharpness(img)
            sharpened = enhancer.enhance(factor)
            
            result_base64 = self.image_to_base64(sharpened)
            
            return {
                "status": "success",
                "preview": f"data:image/png;base64,{result_base64}",
                "operation": "sharpen",
                "params": {"factor": factor}
            }
        except Exception as e:
            logger.error(f"Error sharpening image: {e}")
            return {"error": str(e)}
            
    def edge_detection(self, image_data: str) -> Dict[str, Any]:
        """Detect edges using OpenCV Canny algorithm.
        
        Args:
            image_data: Base64 encoded image
            
        Returns:
            Result dict with edge-detected image
        """
        if not CV2_AVAILABLE:
            return {"error": "OpenCV not available"}
            
        try:
            img = self.base64_to_image(image_data)
            if img is None:
                return {"error": "Failed to decode image"}
                
            # Convert to numpy array
            img_array = np.array(img)
            
            # Convert to grayscale if needed
            if len(img_array.shape) == 3:
                gray = cv2.cvtColor(img_array, cv2.COLOR_RGB2GRAY)
            else:
                gray = img_array
                
            # Apply Canny edge detection
            edges = cv2.Canny(gray, 100, 200)
            
            # Convert back to PIL Image
            result_img = Image.fromarray(edges)
            result_base64 = self.image_to_base64(result_img)
            
            return {
                "status": "success",
                "preview": f"data:image/png;base64,{result_base64}",
                "operation": "edge_detection"
            }
        except Exception as e:
            logger.error(f"Error detecting edges: {e}")
            return {"error": str(e)}


# Example usage
if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    
    processor = ImageProcessor()
    
    # Test with a sample image
    # img = Image.new('RGB', (100, 100), color='red')
    # base64_img = processor.image_to_base64(img)
    # result = processor.apply_gaussian_blur(base64_img, radius=10)
    # print(result)


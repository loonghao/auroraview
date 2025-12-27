"""Example backend module demonstrating code protection.

When `python.protection.enabled = true`, this module will be compiled to a
native extension (`.pyd`/`.so`) during packaging.
"""

import json
from typing import Any, Dict, Optional


class SecretAlgorithm:
    """A class containing proprietary logic that needs protection."""

    def __init__(self, api_key: str):
        self._api_key = api_key
        self._secret_multiplier = 42
        self._internal_state = {}

    def calculate_price(self, quantity: int, unit_price: float) -> float:
        """Calculate price with secret discount algorithm."""
        # This logic will be protected
        base_price = quantity * unit_price

        # Secret discount tiers
        if quantity > 1000:
            discount = 0.25
        elif quantity > 500:
            discount = 0.15
        elif quantity > 100:
            discount = 0.10
        else:
            discount = 0.0

        # Apply secret multiplier
        final_price = base_price * (1 - discount) * (self._secret_multiplier / 100 + 1)

        return round(final_price, 2)

    def validate_license(self, license_key: str) -> bool:
        """Validate a license key using proprietary algorithm."""
        # Secret validation logic
        if len(license_key) != 32:
            return False

        checksum = sum(ord(c) for c in license_key)
        return checksum % 97 == 0

    def process_data(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Process data using proprietary transformation."""
        result = {}

        for key, value in data.items():
            # Secret transformation
            if isinstance(value, (int, float)):
                result[key] = value * self._secret_multiplier
            elif isinstance(value, str):
                result[key] = value[::-1]  # Reverse string
            else:
                result[key] = value

        return result


def initialize_backend(config: Optional[Dict[str, Any]] = None) -> SecretAlgorithm:
    """Initialize the backend with configuration."""
    api_key = config.get("api_key", "default_key") if config else "default_key"
    return SecretAlgorithm(api_key)


def run():
    """Main entry point for the protected application."""
    print("Starting protected application...")

    # Initialize backend
    backend = initialize_backend({"api_key": "secret_api_key_12345"})

    # Demo calculations
    price = backend.calculate_price(150, 9.99)
    print(f"Calculated price: ${price}")

    # Demo license validation
    valid = backend.validate_license("a" * 32)
    print(f"License valid: {valid}")

    # Demo data processing
    result = backend.process_data({"name": "test", "value": 100, "count": 5})
    print(f"Processed data: {json.dumps(result)}")

    print("Application running successfully!")


if __name__ == "__main__":
    run()

#!/usr/bin/env python3
"""
Interactive CLI client for D&D Backstory Generator API
"""

import requests
import sys
import json
from typing import Optional


class APIClient:
    """Client for interacting with the backstory generator API."""
    
    def __init__(self, base_url: str = "http://localhost:5000"):
        self.base_url = base_url
    
    def health(self) -> bool:
        """Check if API is running."""
        try:
            response = requests.get(f"{self.base_url}/health", timeout=5)
            return response.status_code == 200
        except requests.exceptions.RequestException:
            return False
    
    def generate(
        self,
        name: str,
        race: str,
        char_class: str,
        max_tokens: int = 180
    ) -> Optional[str]:
        """Generate backstory for a single character."""
        try:
            data = {
                "name": name.strip(),
                "race": race.strip(),
                "class": char_class.strip(),
                "max_new_tokens": max_tokens
            }
            
            response = requests.post(
                f"{self.base_url}/generate",
                json=data,
                timeout=60
            )
            
            if response.status_code == 200:
                result = response.json()
                if result['success']:
                    return result['backstory']
                else:
                    print(f"Error: {result['error']}")
                    return None
            else:
                print(f"HTTP Error {response.status_code}")
                print(response.json())
                return None
                
        except requests.exceptions.Timeout:
            print("Error: Request timeout (generation took too long)")
            return None
        except requests.exceptions.ConnectionError:
            print("Error: Could not connect to API")
            return None
        except Exception as e:
            print(f"Error: {str(e)}")
            return None


def main():
    """Main CLI loop."""
    client = APIClient()
    
    # Check if API is running
    print("Checking API status...")
    if not client.health():
        print("API is not running!")
        print("Please start it with: python app.py")
        sys.exit(1)
    
    print("API is running!\n")
    print("=== DnD Character Backstory Generator ===\n")
    print("Commands:")
    print("  generate - Generate a backstory")
    print("  quit     - Exit\n")
    
    while True:
        try:
            cmd = input(">> ").strip().lower()
            
            if cmd == "quit" or cmd == "exit" or cmd == "q":
                print("Goodbye!")
                break
            
            elif cmd == "generate" or cmd == "gen":
                print("\nEnter character details:")
                name = input("Name: ").strip()
                if not name:
                    print("Name cannot be empty\n")
                    continue
                
                race = input("Race: ").strip()
                if not race:
                    print("Race cannot be empty\n")
                    continue
                
                char_class = input("Class: ").strip()
                if not char_class:
                    print("Class cannot be empty\n")
                    continue
                
                try:
                    max_tokens = input("Max tokens (default 180): ").strip()
                    max_tokens = int(max_tokens) if max_tokens else 180
                    
                    if max_tokens < 50 or max_tokens > 512:
                        print("Max tokens must be between 50 and 512\n")
                        continue
                except ValueError:
                    print("Invalid max tokens value\n")
                    continue
                
                print(f"\nGenerating backstory for {name}...")
                backstory = client.generate(name, race, char_class, max_tokens)
                
                if backstory:
                    print("\n" + "="*50)
                    print(f"{name} - {race} {char_class}")
                    print("="*50)
                    print(backstory)
                    print("="*50 + "\n")
                else:
                    print()
            
            elif cmd == "help":
                print("\nAvailable commands:")
                print("  generate - Generate a backstory")
                print("  quit     - Exit")
                print("  help     - Show this help\n")
            
            else:
                print("Unknown command. Type 'help' for commands.\n")
        
        except KeyboardInterrupt:
            print("\n\nGoodbye!")
            break
        except Exception as e:
            print(f"Error: {str(e)}\n")


if __name__ == "__main__":
    main()

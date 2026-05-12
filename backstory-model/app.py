import os
import torch
from flask import Flask, request, jsonify
from peft import AutoPeftModelForCausalLM
from transformers import AutoTokenizer
from wsgiref.simple_server import make_server


APP_DIR = os.path.dirname(os.path.abspath(__file__))
BASE_DIR = os.getenv("BACKSTORY_ARTIFACTS_DIR", os.path.join(APP_DIR, "artifacts"))
FINETUNED_DIR = os.getenv("BACKSTORY_FINETUNED_DIR", os.path.join(BASE_DIR, "finetuned"))
HF_TOKEN = os.getenv("HF_TOKEN")

app = Flask(__name__)
model = None
tokenizer = None


def load_model():
    """Load fine-tuned model and tokenizer from disk."""
    global model, tokenizer
    
    if model is not None:
        return
    
    print("Loading tokenizer...")
    tokenizer = AutoTokenizer.from_pretrained(
        FINETUNED_DIR,
        use_fast=True,
        token=HF_TOKEN,
    )
    tokenizer.pad_token = tokenizer.eos_token
    tokenizer.padding_side = "right"
    
    print("Loading fine-tuned model (4-bit LoRA)...")
    compute_dtype = torch.float16 if torch.cuda.is_available() else torch.float32
    
    if torch.cuda.is_available():
        device_map = "auto"
        free_bytes, _ = torch.cuda.mem_get_info()
        free_gb = free_bytes / (1024 ** 3)
        print(f"VRAM available: {free_gb:.2f} GB")
    else:
        device_map = "cpu"
    
    model = AutoPeftModelForCausalLM.from_pretrained(
        FINETUNED_DIR,
        device_map=device_map,
        dtype=compute_dtype,
        low_cpu_mem_usage=True,
        token=HF_TOKEN,
    )
    model.eval()
    print("Model loaded successfully")


def _trim_to_complete_sentence(text: str) -> str:
    """Trim generated text to a complete sentence."""
    text = (text or "").strip()
    if not text:
        return text

    # Cut typical "continuation" tails
    for marker in ("\nWhat is your current goal", "\nA)", "\nB)", "\nC)", "\nQuestion:"):
        pos = text.find(marker)
        if pos != -1:
            text = text[:pos].strip()

    # Keep only complete ending sentence
    last_end = max(text.rfind("."), text.rfind("!"), text.rfind("?"))
    if last_end != -1:
        text = text[: last_end + 1].strip()

    return text


def _decode_completion(output_ids, prompt_len):
    """Decode generated tokens to text."""
    gen_ids = output_ids[0][prompt_len:]
    return tokenizer.decode(gen_ids, skip_special_tokens=True).strip()


def generate_backstory(name: str, race: str, char_class: str, max_new_tokens: int = 180) -> str:
    """
    Generate a D&D character backstory.
    
    Args:
        name: Character name
        race: Character race
        char_class: Character class
        max_new_tokens: Maximum tokens to generate
        
    Returns:
        Generated backstory text
    """
    if model is None:
        raise RuntimeError("Model not loaded. Call load_model() first.")
    
    prompt_template = """Generate Backstory based on following information

Rules:
- Write exactly 3 short paragraphs.
- Paragraph 1: first memories and defining early events.
- Paragraph 2: most valuable actions/turning points and current state.
- Paragraph 3: present goal and one clear adventure hook.
- Keep a dark-fantasy tone.
- No modern technology references.
- Include exactly one flaw and one strong motivation.
- Keep the text concise, coherent, and immersive.
- End with one complete sentence and stop immediately.

Character Name: {}
Character Race: {}
Character Class: {}

Output:
"""
    
    prompt = prompt_template.format(name, race, char_class)
    
    inputs = tokenizer(prompt, return_tensors="pt", truncation=True, max_length=256)
    prompt_len = inputs["input_ids"].shape[1]
    target_device = next(model.parameters()).device
    inputs = {k: v.to(target_device) for k, v in inputs.items()}
    
    # Primary generation with sampling
    with torch.inference_mode():
        output_ids = model.generate(
            **inputs,
            max_new_tokens=max_new_tokens,
            min_new_tokens=72,
            do_sample=True,
            temperature=0.70,
            top_p=0.90,
            repetition_penalty=1.15,
            no_repeat_ngram_size=4,
            pad_token_id=tokenizer.eos_token_id,
            eos_token_id=tokenizer.eos_token_id,
        )
    
    text = _trim_to_complete_sentence(_decode_completion(output_ids, prompt_len))
    
    # Fallback: deterministic decoding if result is empty
    if not text:
        with torch.inference_mode():
            output_ids = model.generate(
                **inputs,
                max_new_tokens=max_new_tokens,
                min_new_tokens=60,
                do_sample=False,
                repetition_penalty=1.20,
                no_repeat_ngram_size=4,
                pad_token_id=tokenizer.eos_token_id,
                eos_token_id=tokenizer.eos_token_id,
            )
        text = _trim_to_complete_sentence(_decode_completion(output_ids, prompt_len))
    
    return text


# API Endpoints

@app.route("/health", methods=["GET"])
def health():
    """Health check endpoint."""
    return jsonify({
        "status": "ok",
        "model_loaded": model is not None,
        "gpu_available": torch.cuda.is_available(),
    })


@app.route("/generate", methods=["POST"])
def generate():
    """
    Generate a D&D character backstory.
    
    Expected JSON payload:
    {
        "name": "Character Name",
        "race": "Human",
        "class": "Paladin",
        "max_new_tokens": 180  # optional
    }
    
    Returns:
    {
        "success": true,
        "backstory": "Generated text...",
        "character": {
            "name": "Character Name",
            "race": "Human",
            "class": "Paladin"
        }
    }
    """
    try:
        data = request.get_json()
        
        if not data:
            return jsonify({"success": False, "error": "Empty request body"}), 400
        
        name = data.get("name", "").strip()
        race = data.get("race", "").strip()
        char_class = data.get("class", "").strip()
        max_new_tokens = data.get("max_new_tokens", 180)
        
        # Validate inputs
        if not name or not race or not char_class:
            return jsonify({
                "success": False,
                "error": "Missing required fields: name, race, class"
            }), 400
        
        if len(name) > 100 or len(race) > 100 or len(char_class) > 100:
            return jsonify({
                "success": False,
                "error": "Input fields too long (max 100 chars)"
            }), 400
        
        if not isinstance(max_new_tokens, int) or max_new_tokens < 50 or max_new_tokens > 512:
            return jsonify({
                "success": False,
                "error": "max_new_tokens must be between 50 and 512"
            }), 400
        
        # Generate backstory
        backstory = generate_backstory(name, race, char_class, max_new_tokens)
        
        if not backstory:
            return jsonify({
                "success": False,
                "error": "Failed to generate backstory (empty result)"
            }), 500
        
        return jsonify({
            "success": True,
            "backstory": backstory,
            "character": {
                "name": name,
                "race": race,
                "class": char_class
            }
        }), 200
        
    except Exception as e:
        print(f"Error: {str(e)}")
        return jsonify({
            "success": False,
            "error": str(e)
        }), 500


@app.route("/batch", methods=["POST"])
def batch_generate():
    """
    Generate backstories for multiple characters in one request.
    
    Expected JSON payload:
    {
        "characters": [
            {"name": "Aragorn", "race": "Human", "class": "Ranger"},
            {"name": "Legolas", "race": "Elf", "class": "Rogue"}
        ]
    }
    
    Returns:
    {
        "success": true,
        "results": [
            {"name": "Aragorn", "backstory": "..."},
            {"name": "Legolas", "backstory": "..."}
        ]
    }
    """
    try:
        data = request.get_json()
        
        if not data or "characters" not in data:
            return jsonify({
                "success": False,
                "error": "Missing 'characters' field"
            }), 400
        
        characters = data.get("characters", [])
        
        if not isinstance(characters, list):
            return jsonify({
                "success": False,
                "error": "'characters' must be a list"
            }), 400
        
        if len(characters) > 10:
            return jsonify({
                "success": False,
                "error": "Maximum 10 characters per request"
            }), 400
        
        results = []
        
        for char in characters:
            try:
                name = char.get("name", "").strip()
                race = char.get("race", "").strip()
                char_class = char.get("class", "").strip()
                
                if not name or not race or not char_class:
                    results.append({
                        "name": name or "Unknown",
                        "success": False,
                        "error": "Missing required fields"
                    })
                    continue
                
                backstory = generate_backstory(name, race, char_class)
                results.append({
                    "name": name,
                    "success": True,
                    "backstory": backstory
                })
                
            except Exception as e:
                results.append({
                    "name": char.get("name", "Unknown"),
                    "success": False,
                    "error": str(e)
                })
        
        return jsonify({
            "success": True,
            "results": results
        }), 200
        
    except Exception as e:
        print(f"Error in batch: {str(e)}")
        return jsonify({
            "success": False,
            "error": str(e)
        }), 500


@app.errorhandler(404)
def not_found(error):
    """Handle 404 errors."""
    return jsonify({
        "success": False,
        "error": "Endpoint not found"
    }), 404


@app.errorhandler(405)
def method_not_allowed(error):
    """Handle 405 errors."""
    return jsonify({
        "success": False,
        "error": "Method not allowed"
    }), 405


@app.before_request
def before_request():
    """Pre-request hook to ensure model is loaded."""
    if model is None and request.path not in ["/health"]:
        load_model()


if __name__ == "__main__":
    print("Starting DnD Backstory Generator API...")
    
    # Check if model exists
    if not os.path.exists(FINETUNED_DIR):
        print(f"Error: Fine-tuned model not found at {FINETUNED_DIR}")
        print("Please train the model first using trainer-new.ipynb")
        exit(1)
    
    # Pre-load model on startup
    load_model()
    
    # Run via a standard WSGI server instead of Flask's dev server.
    server = make_server("0.0.0.0", 5000, app)
    print("Server running on http://0.0.0.0:5000")
    server.serve_forever()

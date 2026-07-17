import json
import os
import platform
from datetime import datetime
from typing import Dict, Any

HISTORY_FILE = os.path.join(os.path.dirname(__file__), "regression_history.jsonl")

def get_reproducibility_context(model: str, prompt_version: str, dataset_version: str, seed: int = 42, temperature: float = 0.0) -> Dict[str, Any]:
    return {
        "timestamp": datetime.utcnow().isoformat() + "Z",
        "model": model,
        "prompt_version": prompt_version,
        "dataset_version": dataset_version,
        "inference_params": {
            "seed": seed,
            "temperature": temperature,
            "top_p": 0.9,
            "provider": "ollama"
        },
        "hardware": {
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
            "processor": platform.processor()
        }
    }

def record_benchmark_run(context: Dict[str, Any], results: Dict[str, Any]) -> None:
    entry = {
        "context": context,
        "results": results
    }
    
    with open(HISTORY_FILE, "a") as f:
        f.write(json.dumps(entry) + "\n")

def load_history() -> list:
    if not os.path.exists(HISTORY_FILE):
        return []
    
    history = []
    with open(HISTORY_FILE, "r") as f:
        for line in f:
            line = line.strip()
            if line:
                history.append(json.loads(line))
    return history

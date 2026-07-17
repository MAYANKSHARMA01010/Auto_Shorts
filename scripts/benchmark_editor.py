import os
import json
import time
import urllib.request
import urllib.error
import random
from typing import List, Dict

# Import evaluation modules
from evaluation.stats import compute_statistics, analyze_statistical_significance
from evaluation.history import get_reproducibility_context, record_benchmark_run
from evaluation.dashboard_generator import generate_dashboard

OLLAMA_URL = "http://localhost:11434/api/chat"
MODEL = "qwen2.5"
DATASET_VERSION = "v2.0"
REPETITIONS = 5
SEED = 42

PROMPT_TEMPLATE = """You are an expert short-form video editor.
Your goal is to create the most engaging story possible. You are NOT a timestamp finder.
==================================================
EDITOR PHILOSOPHY & PRINCIPLES
- Every second must earn its place.
- A complete story with slightly lower viral potential is ALWAYS better than an incomplete story.
==================================================
MULTIMODAL HEURISTICS
- Scene Changes: Prefer beginning a clip immediately after a scene change.
- Silence: Avoid cutting during silence unless intentionally pacing.
- Visual Energy: Prefer cuts when visual energy or motion increases.
- Faces: Prioritize retaining segments where faces are present.
==================================================
INTERNAL MULTI-PASS REASONING
Perform reasoning internally before producing your answer. Return ONLY the required JSON.

Return ONLY valid JSON strictly in the following schema:
{{
  "candidates": [
    {{
      "score": 9.7,
      "hook": "The first spoken sentence that hooks the viewer",
      "rationale": "Why this edit works well",
      "confidence": 0.92,
      "segments": [
        {{
          "start": 12.5,
          "end": 45.2,
          "text": "The exact transcript text here"
        }}
      ]
    }}
  ]
}}

Transcript:
{transcript}"""

def load_dataset():
    dataset_path = os.path.join(os.path.dirname(__file__), "evaluation", "ground_truth_dataset.json")
    with open(dataset_path, "r") as f:
        return json.load(f)

def run_ollama(prompt, temperature=0.0):
    payload = {
        "model": MODEL,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "format": "json",
        "stream": False,
        "options": {
            "temperature": temperature,
            "seed": SEED,
            "top_p": 0.9
        }
    }
    
    req = urllib.request.Request(OLLAMA_URL, data=json.dumps(payload).encode('utf-8'),
                                 headers={'Content-Type': 'application/json'})
    
    start_t = time.time()
    try:
        with urllib.request.urlopen(req, timeout=120) as response:
            result = json.loads(response.read().decode())
            latency = time.time() - start_t
            return result['message']['content'], latency
    except Exception as e:
        print(f"Error querying Ollama: {e}")
        return None, 0.0

def build_transcript_string(video, mode):
    lines = []
    for seg in video["segments"]:
        time_str = seg["time"]
        text_str = seg["text"]
        
        vis_tag = f"[Vis: {seg['vis']}] " if mode in ["Text + Visual", "Text + Audio + Visual"] else ""
        aud_tag = f"[Aud: {seg['aud']}] " if mode in ["Text + Audio", "Text + Audio + Visual"] else ""
        
        lines.append(f"{time_str} {vis_tag}{aud_tag}{text_str}")
    return "\n".join(lines)

def run_experiment_mode(mode: str, dataset: List[Dict], repetitions: int) -> Dict:
    latencies = []
    scores = []
    
    for rep in range(repetitions):
        print(f"  [Repetition {rep+1}/{repetitions}]")
        for vid in dataset:
            transcript_str = build_transcript_string(vid, mode)
            prompt = PROMPT_TEMPLATE.format(transcript=transcript_str)
            
            res_str, lat = run_ollama(prompt, temperature=0.2)
            if res_str:
                try:
                    data = json.loads(res_str)
                    candidates = data.get("candidates", [])
                    if candidates:
                        best_score = max([c.get("score", 0.0) for c in candidates])
                        scores.append(best_score)
                        latencies.append(lat)
                except Exception as e:
                    print(f"Failed to parse JSON: {e}")
                    
    return {
        "latency": compute_statistics(latencies),
        "score": compute_statistics(scores),
        "raw_latency": latencies,
        "raw_score": scores
    }

def main():
    print("Starting Scientific Validation Framework...")
    dataset = load_dataset()
    
    modes_to_test = ["Text Only", "Text + Audio + Visual"]
    results = {}
    
    for mode in modes_to_test:
        print(f"\nEvaluating Mode: {mode}")
        results[mode] = run_experiment_mode(mode, dataset, REPETITIONS)
        
    baseline = results["Text Only"]
    new_mode = results["Text + Audio + Visual"]
    
    sig_results = {
        "latency": analyze_statistical_significance(baseline["raw_latency"], new_mode["raw_latency"], higher_is_better=False),
        "score": analyze_statistical_significance(baseline["raw_score"], new_mode["raw_score"], higher_is_better=True)
    }
    
    context = get_reproducibility_context(MODEL, "v2.1", DATASET_VERSION, SEED, 0.2)
    
    # Log to history
    record_benchmark_run(context, new_mode)
    
    # Generate Dashboard
    dashboard_path = os.path.join(os.path.dirname(__file__), "evaluation_dashboard.md")
    generate_dashboard(context, new_mode, baseline, sig_results, dashboard_path)
    
    print(f"\nExperiment Complete. Dashboard generated at {dashboard_path}.")

if __name__ == "__main__":
    main()

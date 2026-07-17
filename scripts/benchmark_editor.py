import os
import json
import time
import urllib.request
import urllib.error
import csv
from datetime import datetime
import hashlib
import statistics

# Benchmark Script for AutoShorts AI Editing Intelligence (Phase 5)
# Evaluates 4 Multimodal Modes: Text Only, Text+Audio, Text+Visual, Text+All

OLLAMA_URL = "http://localhost:11434/api/chat"
MODEL = "qwen2.5" 

DATASET_VERSION = "v1.1"

# Mock Dataset representing various categories and multimodal hooks
DATASET = [
    {
        "id": "vid_01",
        "category": "Podcast",
        "duration": 60.0,
        "segments": [
            {"time": "[0.00-15.00]", "text": "Speaker 1: AI is changing the world.", "vis": "SceneChange=True, Motion=low, Face=True", "aud": "Silence=False, Vol=-15.0dB"},
            {"time": "[15.00-30.00]", "text": "Speaker 2: Yes, it is moving so fast.", "vis": "SceneChange=False, Motion=medium, Face=True", "aud": "Silence=False, Vol=-14.0dB"},
            {"time": "[30.00-45.00]", "text": "Speaker 1: What happens in 5 years?", "vis": "SceneChange=False, Motion=low, Face=True", "aud": "Silence=False, Vol=-16.0dB"},
            {"time": "[45.00-60.00]", "text": "Speaker 2: We might have AGI.", "vis": "SceneChange=True, Motion=high, Face=True", "aud": "Silence=False, Vol=-12.0dB"}
        ]
    },
    {
        "id": "vid_02",
        "category": "Tutorial",
        "duration": 50.0,
        "segments": [
            {"time": "[0.00-10.00]", "text": "Host: Today I'll show you how to bake a cake.", "vis": "SceneChange=True, Motion=low, Face=True", "aud": "Silence=False, Vol=-18.0dB"},
            {"time": "[10.00-25.00]", "text": "Host: First, get some flour.", "vis": "SceneChange=True, Motion=high, Face=False", "aud": "Silence=False, Vol=-20.0dB"},
            {"time": "[25.00-40.00]", "text": "Host: Next, mix it with eggs.", "vis": "SceneChange=False, Motion=high, Face=False", "aud": "Silence=False, Vol=-19.0dB"},
            {"time": "[40.00-50.00]", "text": "Host: Finally, put it in the oven.", "vis": "SceneChange=True, Motion=low, Face=True", "aud": "Silence=False, Vol=-18.0dB"}
        ]
    },
    {
        "id": "vid_03",
        "category": "Comedy",
        "duration": 40.0,
        "segments": [
            {"time": "[0.00-20.00]", "text": "Comedian: So I went to the store yesterday to buy milk.", "vis": "SceneChange=False, Motion=medium, Face=True", "aud": "Silence=False, Vol=-12.0dB"},
            {"time": "[20.00-40.00]", "text": "Comedian: And the cow was working the register!", "vis": "SceneChange=True, Motion=high, Face=True", "aud": "Silence=False, Vol=-5.0dB"}
        ]
    }
]

PROMPT_TEMPLATE = """You are an expert short-form video editor for YouTube Shorts, Instagram Reels, and TikTok.
Your goal is to create the most engaging, complete, and natural story possible. You are NOT a timestamp finder; you are creating a brand new edit. The transcript represents raw footage.

==================================================
EDITOR PHILOSOPHY & PRINCIPLES
==================================================
- Every second must earn its place. If something does not increase curiosity, emotion, understanding, or move the story forward—remove it.
- Never remove context, setup, or payoff unless they are genuinely unnecessary.
- The AI should NEVER assume that one continuous clip is best. You may trim, remove, merge, stitch, compress, shorten, and combine multiple transcript regions to produce the strongest possible story.
- There is no preferred editing style. Only viewer retention matters.
- A clip MUST be at least 30 seconds long.
- A complete story with a slightly lower viral potential is ALWAYS better than an incomplete story with a higher viral score.

==================================================
MULTIMODAL HEURISTICS (AUDIO & VISUAL SIGNALS)
==================================================
You will receive [Aud: ...] and [Vis: ...] metadata tags attached to transcript segments.
Use these signals to make better editing decisions:
- **Scene Changes**: Prefer beginning a clip immediately after a scene change (e.g. `SceneChange=True`).
- **Silence**: Avoid cutting during silence (`isSilence=True`) unless intentionally pacing a dramatic pause.
- **Visual Energy**: Prefer cuts when visual energy or motion increases (`Motion=high`). Maintain visual continuity.
- **Faces**: Prioritize retaining segments where faces are present (`Face=True`).
These are heuristics, not rigid rules. The narrative story is always the priority.

==================================================
EDITING STYLES & DIVERSITY
==================================================
Choose the structure that maximizes retention. Possible editing styles include:
- continuous clip
- stitched edit
- montage
- before -> after
- problem -> solution
- setup -> payoff
- question -> answer
- challenge -> resolution
- explanation -> demonstration

Candidate Diversity: Do not generate near-duplicate candidates. Each candidate should represent a genuinely different editing strategy whenever appropriate (e.g., Story-focused vs. High-retention fast pacing vs. Educational clarity).

==================================================
STITCHING RULES & PACE OPTIMIZATION
==================================================
- Feel free to take 12 seconds from minute 2, combine with 18 seconds from minute 7, and 15 seconds from minute 14 if together they produce the strongest story.
- Never destroy chronology unless explicitly supported by the source material. Never create misleading narratives or fabricate meaning.
- Constantly ask: "Would the average Shorts viewer swipe away here?" If yes: remove, compress, or jump forward.
- CRITICAL: To make a clip long enough, you MUST combine multiple consecutive transcript lines into a single segment. Use the start time of the FIRST line and the end time of the LAST line to form a long segment. Do NOT just copy a single 2-second line.

==================================================
INTERNAL MULTI-PASS REASONING
==================================================
Perform the following reasoning internally before producing your answer. Do NOT reveal your reasoning. Return ONLY the required JSON.
- Pass 1: Understand transcript
- Pass 2: Identify content type (e.g., Storytelling, Tutorial, Comedy)
- Pass 3: Identify emotional peaks and hooks
- Pass 4: Group related moments
- Pass 5: Choose editing strategy (e.g., problem -> solution, montage)
- Pass 6: Construct complete story
- Pass 7: Optimize pacing
- Pass 8: Merge nearby segments
- Pass 9: Validate final edit

==================================================
OPTIONAL METADATA
==================================================
You may optionally output your chosen `editing_strategy` (e.g. "problem -> solution") and a `confidence` score (e.g. 0.95) for each candidate.

Return ONLY valid JSON strictly in the following schema:
{{
  "candidates": [
    {{
      "score": 9.7,
      "hook": "The first spoken sentence that hooks the viewer",
      "rationale": "Why this edit works well for viral short form",
      "editing_strategy": "problem -> solution",
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

def generate_prompt_version():
    hasher = hashlib.md5()
    hasher.update(PROMPT_TEMPLATE.encode('utf-8'))
    return hasher.hexdigest()[:8]

def run_ollama(prompt, schema=None):
    payload = {
        "model": MODEL,
        "messages": [{"role": "user", "content": prompt}],
        "stream": False,
        "options": {"temperature": 0.2},
    }
    if schema:
        payload["format"] = schema
        
    req = urllib.request.Request(OLLAMA_URL, data=json.dumps(payload).encode('utf-8'), headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(req, timeout=120) as response:
        result = json.loads(response.read().decode())
        content = result.get('message', {}).get('content', '')
        prompt_tokens = result.get('prompt_eval_count', 0)
        completion_tokens = result.get('eval_count', 0)
        return content, prompt_tokens, completion_tokens

def evaluate_mode(mode_name, include_audio, include_visual):
    print(f"\nEvaluating Mode: {mode_name}...")
    schema = {
        "type": "object",
        "properties": {
            "candidates": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "score": { "type": "number" },
                        "hook": { "type": "string" },
                        "rationale": { "type": "string" },
                        "editing_strategy": { "type": "string" },
                        "confidence": { "type": "number" },
                        "segments": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "start": { "type": "number" },
                                    "end": { "type": "number" },
                                    "text": { "type": "string" }
                                },
                                "required": ["start", "end", "text"]
                            }
                        }
                    },
                    "required": ["score", "hook", "rationale", "segments"]
                }
            }
        },
        "required": ["candidates"]
    }

    metrics = {
        "runs": 0,
        "total_time": 0,
        "total_prompt_tokens": 0,
        "total_completion_tokens": 0,
        "avg_confidence": 0,
        "avg_score": 0,
    }

    confidences = []
    scores = []

    for item in DATASET:
        start_time = time.time()
        
        # Build Transcript string for this mode
        transcript_lines = []
        for seg in item["segments"]:
            tags = []
            if include_visual:
                tags.append(f"[Vis: {seg['vis']}]")
            if include_audio:
                tags.append(f"[Aud: {seg['aud']}]")
                
            prefix = " ".join(tags) + " " if tags else ""
            transcript_lines.append(f"{seg['time']} {prefix}{seg['text']}")
            
        transcript_str = "\n".join(transcript_lines)
            
        try:
            prompt = PROMPT_TEMPLATE.replace("{transcript}", transcript_str)
            content, p_tok, c_tok = run_ollama(prompt, schema)
            
            parsed = json.loads(content)
            
            if "candidates" in parsed:
                for idx, c in enumerate(parsed["candidates"]):
                    if "confidence" in c and isinstance(c["confidence"], (int, float)):
                        confidences.append(c["confidence"])
                    if "score" in c and isinstance(c["score"], (int, float)):
                        scores.append(c["score"])
                    
        except Exception as e:
            print(f"Error on {item['id']}: {e}")
            
        metrics['total_time'] += (time.time() - start_time)
        metrics['total_prompt_tokens'] += p_tok
        metrics['total_completion_tokens'] += c_tok
        metrics['runs'] += 1

    metrics['avg_confidence'] = statistics.mean(confidences) if confidences else 0
    metrics['avg_score'] = statistics.mean(scores) if scores else 0
    return metrics

def run_benchmark():
    print(f"Starting Phase 5 Benchmark Framework...")
    
    modes = [
        {"name": "Text Only", "audio": False, "visual": False},
        {"name": "Text + Audio", "audio": True, "visual": False},
        {"name": "Text + Visual", "audio": False, "visual": True},
        {"name": "Text + Audio + Visual", "audio": True, "visual": True},
    ]
    
    results = {}
    for m in modes:
        results[m["name"]] = evaluate_mode(m["name"], m["audio"], m["visual"])
    
    # Generate Output
    report = f"# AutoShorts Phase 5 Benchmark Dashboard\n\n"
    report += f"**Date:** {datetime.now().isoformat()}\n"
    report += f"**Model:** {MODEL}\n"
    
    report += "## Multimodal Performance Comparison\n\n"
    report += "| Mode | Avg Score | Avg Confidence | Avg Latency/Vid | Avg Prompt Tokens | Avg Completion Tokens |\n"
    report += "|---|---|---|---|---|---|\n"
    
    for m in modes:
        res = results[m["name"]]
        runs = max(1, res["runs"])
        report += f"| {m['name']} | {res['avg_score']:.2f} | {res['avg_confidence']:.2f} | {res['total_time']/runs:.2f}s | {res['total_prompt_tokens']/runs:.1f} | {res['total_completion_tokens']/runs:.1f} |\n"
        
    with open("benchmark_dashboard.md", "w", encoding='utf-8') as f:
        f.write(report)
        
    print(f"Benchmark complete. Dashboard saved to benchmark_dashboard.md.")

if __name__ == "__main__":
    run_benchmark()

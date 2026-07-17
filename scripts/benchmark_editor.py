import os
import json
import time
import urllib.request
import urllib.error
import csv
from datetime import datetime
import hashlib
import statistics

# Benchmark Script for AutoShorts AI Editing Intelligence (Phase 4)
# Regression Testing, Mode Comparison, and Multi-metric Evaluation.

OLLAMA_URL = "http://localhost:11434/api/chat"
MODEL = "qwen2.5" 

DATASET_VERSION = "v1.0"

# Mock Dataset representing various categories and multimodal hooks
DATASET = [
    {
        "id": "vid_01",
        "category": "Podcast",
        "duration": 60.0,
        "transcript": "[0.00-15.00] Speaker 1: AI is changing the world.\n[15.00-30.00] Speaker 2: Yes, it is moving so fast.\n[30.00-45.00] Speaker 1: What happens in 5 years?\n[45.00-60.00] Speaker 2: We might have AGI.",
        "hooks": {"scene_changes": [15.0, 30.0, 45.0], "visual_energy": "low"}
    },
    {
        "id": "vid_02",
        "category": "Tutorial",
        "duration": 50.0,
        "transcript": "[0.00-10.00] Host: Today I'll show you how to bake a cake.\n[10.00-25.00] Host: First, get some flour.\n[25.00-40.00] Host: Next, mix it with eggs.\n[40.00-50.00] Host: Finally, put it in the oven.",
        "hooks": {"scene_changes": [10.0, 25.0, 40.0], "visual_energy": "medium"}
    },
    {
        "id": "vid_03",
        "category": "Comedy",
        "duration": 40.0,
        "transcript": "[0.00-20.00] Comedian: So I went to the store yesterday to buy milk.\n[20.00-40.00] Comedian: And the cow was working the register!",
        "hooks": {"scene_changes": [], "visual_energy": "high"}
    }
]

ANALYSIS_PROMPT_TEMPLATE = """You are an expert story analyzer for short-form video.
Analyze the following transcript.
Output exactly 3 parts:
1. CONTENT TYPE:
2. EMOTIONAL PEAKS:
3. RECOMMENDED STRATEGY:

Transcript:
{transcript}"""

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

{analyzer_section}

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

Candidate Diversity: Do not generate near-duplicate candidates. Each candidate should represent a genuinely different editing strategy whenever appropriate.

==================================================
STITCHING RULES & PACE OPTIMIZATION
==================================================
- Feel free to take 12 seconds from minute 2, combine with 18 seconds from minute 7, and 15 seconds from minute 14 if together they produce the strongest story.
- Never destroy chronology unless explicitly supported by the source material. Never create narratives or fabricate meaning.
- CRITICAL: To make a clip long enough, you MUST combine multiple consecutive transcript lines into a single segment. Use the start time of the FIRST line and the end time of the LAST line to form a long segment. Do NOT just copy a single 2-second line.

==================================================
INTERNAL MULTI-PASS REASONING
==================================================
Perform the following reasoning internally before producing your answer. Do NOT reveal your reasoning. Return ONLY the required JSON.

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

def evaluate_mode(mode_name, use_analyzer):
    print(f"\\nEvaluating {mode_name}...")
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
        "json_validity": 0,
        "parsing_success": 0,
        "llm_failures": 0,
        "total_time": 0,
        "total_prompt_tokens": 0,
        "total_completion_tokens": 0,
        "total_candidates": 0,
        "total_segments": 0,
        "avg_confidence": 0,
        "avg_score": 0,
        "csv_data": []
    }

    confidences = []
    scores = []

    for item in DATASET:
        start_time = time.time()
        try:
            prompt_tokens, completion_tokens = 0, 0
            analyzer_section = ""
            
            if use_analyzer:
                analysis_prompt = ANALYSIS_PROMPT_TEMPLATE.replace("{transcript}", item["transcript"])
                analysis_result, p_tok, c_tok = run_ollama(analysis_prompt)
                prompt_tokens += p_tok
                completion_tokens += c_tok
                analyzer_section = f"==================================================\\nSTORY ANALYZER & CONTENT CLASSIFICATION\\n==================================================\\nThe following analysis has been performed on the transcript:\\n{analysis_result}\\n\\nAdapt your editing strategy automatically based on this analysis."
            
            prompt = PROMPT_TEMPLATE.replace("{analyzer_section}", analyzer_section).replace("{transcript}", item["transcript"])
            
            content, p_tok, c_tok = run_ollama(prompt, schema)
            prompt_tokens += p_tok
            completion_tokens += c_tok
            
            parsed = json.loads(content)
            metrics['json_validity'] += 1
            
            if "candidates" in parsed:
                metrics['parsing_success'] += 1
                for idx, c in enumerate(parsed["candidates"]):
                    metrics['total_candidates'] += 1
                    segments = c.get("segments", [])
                    metrics['total_segments'] += len(segments)
                    
                    if "confidence" in c and isinstance(c["confidence"], (int, float)):
                        confidences.append(c["confidence"])
                    if "score" in c and isinstance(c["score"], (int, float)):
                        scores.append(c["score"])
                    
                    metrics['csv_data'].append({
                        "mode": mode_name,
                        "video_id": item["id"],
                        "category": item["category"],
                        "candidate_index": idx,
                        "hook": c.get("hook", ""),
                        "strategy": c.get("editing_strategy", "N/A"),
                        "ai_confidence": c.get("confidence", "N/A"),
                        "ai_score": c.get("score", "N/A"),
                        "human_overall_score": ""
                    })
                    
        except Exception as e:
            metrics['llm_failures'] += 1
            print(f"Error on {item['id']}: {e}")
            
        metrics['total_time'] += (time.time() - start_time)
        metrics['total_prompt_tokens'] += prompt_tokens
        metrics['total_completion_tokens'] += completion_tokens
        metrics['runs'] += 1

    metrics['avg_confidence'] = statistics.mean(confidences) if confidences else 0
    metrics['avg_score'] = statistics.mean(scores) if scores else 0
    return metrics

def run_benchmark():
    print(f"Starting Phase 4 Regression Framework...")
    prompt_version = generate_prompt_version()
    
    # Run Mode A (No Analyzer)
    metrics_a = evaluate_mode("Mode A (Internal Reasoning)", use_analyzer=False)
    
    # Run Mode B (With Analyzer)
    metrics_b = evaluate_mode("Mode B (Story Analyzer)", use_analyzer=True)
    
    # Generate Output
    report = f"# AutoShorts Phase 4 Benchmark Report\\n\\n"
    report += f"**Date:** {datetime.now().isoformat()}\\n"
    report += f"**Model:** {MODEL}\\n"
    report += f"**Prompt Version:** {prompt_version}\\n"
    report += f"**Dataset Version:** {DATASET_VERSION} ({len(DATASET)} videos)\\n\\n"
    
    report += "## Mode A vs Mode B Comparison\\n\\n"
    report += "| Metric | Mode A (Single Pass) | Mode B (Two Pass Analyzer) |\\n"
    report += "|---|---|---|\\n"
    
    for key in ['json_validity', 'parsing_success', 'llm_failures']:
        report += f"| {key} | {metrics_a[key]}/{metrics_a['runs']} | {metrics_b[key]}/{metrics_b['runs']} |\\n"
        
    report += f"| Avg Time per Video | {metrics_a['total_time']/max(1, metrics_a['runs']):.2f}s | {metrics_b['total_time']/max(1, metrics_b['runs']):.2f}s |\\n"
    report += f"| Avg Prompt Tokens | {metrics_a['total_prompt_tokens']/max(1, metrics_a['runs']):.1f} | {metrics_b['total_prompt_tokens']/max(1, metrics_b['runs']):.1f} |\\n"
    report += f"| Avg Completion Tokens | {metrics_a['total_completion_tokens']/max(1, metrics_a['runs']):.1f} | {metrics_b['total_completion_tokens']/max(1, metrics_b['runs']):.1f} |\\n"
    report += f"| Total Candidates | {metrics_a['total_candidates']} | {metrics_b['total_candidates']} |\\n"
    report += f"| Avg Segments/Candidate | {metrics_a['total_segments']/max(1, metrics_a['total_candidates']):.2f} | {metrics_b['total_segments']/max(1, metrics_b['total_candidates']):.2f} |\\n"
    report += f"| Avg AI Confidence | {metrics_a['avg_confidence']:.2f} | {metrics_b['avg_confidence']:.2f} |\\n"
    report += f"| Avg AI Score | {metrics_a['avg_score']:.2f} | {metrics_b['avg_score']:.2f} |\\n\\n"
    
    # Regression logic
    baseline_file = "baseline_metrics.json"
    status = "PASS"
    if os.path.exists(baseline_file):
        with open(baseline_file, 'r') as f:
            baseline = json.load(f)
            
        report += "## Regression Analysis (vs Baseline)\\n"
        old_val = baseline.get("parsing_success", 0)
        new_val = metrics_b["parsing_success"]
        if new_val < old_val:
            report += f"- ❌ Parsing Success degraded from {old_val} to {new_val}.\\n"
            status = "FAIL"
        else:
            report += f"- ✅ Parsing Success steady or improved ({new_val}).\\n"
            
        old_time = baseline.get("avg_time", 999)
        new_time = metrics_b["total_time"]/max(1, metrics_b["runs"])
        if new_time > old_time * 1.5:
            report += f"- ❌ Latency degraded significantly ({old_time:.2f}s -> {new_time:.2f}s).\\n"
            status = "FAIL"
        else:
            report += f"- ✅ Latency acceptable ({new_time:.2f}s).\\n"
    else:
        report += "## Regression Analysis\\n*No baseline found. Saving current Mode B as baseline.*\\n"
        
    report += f"\\n### Overall Status: **{status}**\\n"

    # Save Baseline
    with open(baseline_file, 'w') as f:
        json.dump({
            "parsing_success": metrics_b["parsing_success"],
            "avg_time": metrics_b["total_time"]/max(1, metrics_b["runs"])
        }, f)

    # Save Markdown
    with open("benchmark_dashboard.md", "w") as f:
        f.write(report)
        
    print(f"\\nStatus: {status}")
    print("Generated benchmark_dashboard.md")

    # Write CSV
    all_csv_data = metrics_a['csv_data'] + metrics_b['csv_data']
    if all_csv_data:
        csv_filename = f"human_evaluation_{prompt_version}.csv"
        keys = all_csv_data[0].keys()
        with open(csv_filename, 'w', newline='') as output_file:
            dict_writer = csv.DictWriter(output_file, keys)
            dict_writer.writeheader()
            dict_writer.writerows(all_csv_data)
        print(f"Generated {csv_filename}")

if __name__ == "__main__":
    run_benchmark()

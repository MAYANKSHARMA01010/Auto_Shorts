import os
import json
from typing import Dict, Any

def generate_dashboard(context: Dict[str, Any], results: Dict[str, Any], baseline_results: Dict[str, Any], significance_results: Dict[str, Any], output_path: str):
    dashboard_content = f"""# AutoShorts Evaluation Dashboard

**Date:** {context['timestamp']}
**Model:** {context['model']}
**Dataset:** {context['dataset_version']}
**Prompt Version:** {context['prompt_version']}

## Executive Summary
This dashboard compares the new execution against the baseline to validate whether observed improvements are statistically significant.

### Hardware & Reproducibility Context
- **System:** {context['hardware']['system']} {context['hardware']['release']} ({context['hardware']['processor']})
- **Inference Params:** Seed {context['inference_params']['seed']}, Temp {context['inference_params']['temperature']}

## Quality Metrics
*Note: AI-generated quality scores are subjective. Human evaluation should be cross-referenced.*

| Mode | Avg Score | Median Score | Confidence Interval (95%) |
|---|---|---|---|
| Baseline | {baseline_results['score']['mean']:.2f} | {baseline_results['score']['median']:.2f} | [{baseline_results['score']['ci_lower']:.2f}, {baseline_results['score']['ci_upper']:.2f}] |
| New Mode | {results['score']['mean']:.2f} | {results['score']['median']:.2f} | [{results['score']['ci_lower']:.2f}, {results['score']['ci_upper']:.2f}] |

## Performance Metrics (Latency per Video)

| Mode | Mean Latency | Median Latency | Min | Max |
|---|---|---|---|---|
| Baseline | {baseline_results['latency']['mean']:.2f}s | {baseline_results['latency']['median']:.2f}s | {baseline_results['latency']['min']:.2f}s | {baseline_results['latency']['max']:.2f}s |
| New Mode | {results['latency']['mean']:.2f}s | {results['latency']['median']:.2f}s | {results['latency']['min']:.2f}s | {results['latency']['max']:.2f}s |

## Statistical Validation

### Latency Improvement
- **Improvement:** {significance_results['latency']['improvement_pct']:.2f}% (Higher is better / shorter latency)
- **Statistically Significant?** {"✅ YES" if significance_results['latency']['significant'] else "❌ NO"}
- **Reason:** {significance_results['latency']['reason']}

### Score Improvement
- **Improvement:** {significance_results['score']['improvement_pct']:.2f}%
- **Statistically Significant?** {"✅ YES" if significance_results['score']['significant'] else "❌ NO"}
- **Reason:** {significance_results['score']['reason']}

---
*Generated automatically by AutoShorts Evaluation Framework.*
"""
    with open(output_path, "w") as f:
        f.write(dashboard_content)

    print(f"Dashboard generated at {output_path}")

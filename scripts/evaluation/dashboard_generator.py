import os
from typing import Dict, Any

def generate_dashboard(context: Dict[str, Any], results: Dict[str, Any], baseline_results: Dict[str, Any], significance_results: Dict[str, Any], output_path: str):
    dashboard_content = f"""# AutoShorts Evaluation Dashboard (v2 - Statistical Rigor)

**Date:** {context['timestamp']}
**Model:** {context['model']}
**Dataset:** {context['dataset_version']}
**Prompt Version:** {context['prompt_version']}

## Executive Summary
This dashboard evaluates whether observed benchmark differences are statistically significant using formal hypothesis testing (Paired t-test) and effect size measurements (Cohen's d).

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

## Statistical Validation (Hypothesis Testing)

### Latency Improvement
- **Difference:** {significance_results['latency']['improvement_pct']:.2f}% (Higher is better / shorter latency)
- **p-value:** {significance_results['latency']['p_value']:.4f}
- **Effect Size (Cohen's d):** {significance_results['latency']['effect_size']:.2f} ({significance_results['latency']['effect_size_cat']})
- **Statistical Power Estimate:** {significance_results['latency']['power']}
- **Recommendation:** **{significance_results['latency']['recommendation']}**

### Score Improvement
- **Difference:** {significance_results['score']['improvement_pct']:.2f}%
- **p-value:** {significance_results['score']['p_value']:.4f}
- **Effect Size (Cohen's d):** {significance_results['score']['effect_size']:.2f} ({significance_results['score']['effect_size_cat']})
- **Statistical Power Estimate:** {significance_results['score']['power']}
- **Recommendation:** **{significance_results['score']['recommendation']}**

---
*Generated automatically by AutoShorts Evaluation Framework.*
"""
    with open(output_path, "w") as f:
        f.write(dashboard_content)

    print(f"Dashboard generated at {output_path}")

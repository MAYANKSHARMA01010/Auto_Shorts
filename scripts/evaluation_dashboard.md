# AutoShorts Evaluation Dashboard

**Date:** 2026-07-17T13:20:38.751012Z
**Model:** qwen2.5
**Dataset:** v2.0
**Prompt Version:** v2.1

## Executive Summary
This dashboard compares the new execution against the baseline to validate whether observed improvements are statistically significant.

### Hardware & Reproducibility Context
- **System:** Darwin 25.4.0 (arm)
- **Inference Params:** Seed 42, Temp 0.2

## Quality Metrics
*Note: AI-generated quality scores are subjective. Human evaluation should be cross-referenced.*

| Mode | Avg Score | Median Score | Confidence Interval (95%) |
|---|---|---|---|
| Baseline | 9.70 | 9.70 | [9.70, 9.70] |
| New Mode | 9.70 | 9.70 | [9.70, 9.70] |

## Performance Metrics (Latency per Video)

| Mode | Mean Latency | Median Latency | Min | Max |
|---|---|---|---|---|
| Baseline | 9.89s | 9.46s | 8.18s | 18.48s |
| New Mode | 11.89s | 12.32s | 8.89s | 15.46s |

## Statistical Validation

### Latency Improvement
- **Improvement:** -20.27% (Higher is better / shorter latency)
- **Statistically Significant?** ❌ NO
- **Reason:** 95% Confidence Intervals do not overlap

### Score Improvement
- **Improvement:** 0.00%
- **Statistically Significant?** ❌ NO
- **Reason:** 95% Confidence Intervals overlap

---
*Generated automatically by AutoShorts Evaluation Framework.*

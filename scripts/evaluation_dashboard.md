# AutoShorts Evaluation Dashboard (v2 - Statistical Rigor)

**Date:** 2026-07-17T13:34:46.529884Z
**Model:** qwen2.5
**Dataset:** v2.0
**Prompt Version:** v2.1

## Executive Summary
This dashboard evaluates whether observed benchmark differences are statistically significant using formal hypothesis testing (Paired t-test) and effect size measurements (Cohen's d).

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
| Baseline | 9.54s | 9.13s | 7.98s | 17.18s |
| New Mode | 12.47s | 12.24s | 9.10s | 19.47s |

## Statistical Validation (Hypothesis Testing)

### Latency Improvement
- **Difference:** -30.66% (Higher is better / shorter latency)
- **p-value:** 0.0002
- **Effect Size (Cohen's d):** 1.23 (Large)
- **Statistical Power Estimate:** High
- **Recommendation:** **Reject (Statistically Significant Degradation)**

### Score Improvement
- **Difference:** 0.00%
- **p-value:** 1.0000
- **Effect Size (Cohen's d):** 0.00 (Negligible)
- **Statistical Power Estimate:** Low
- **Recommendation:** **Needs More Data**

---
*Generated automatically by AutoShorts Evaluation Framework.*

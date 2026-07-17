import math
from typing import List, Dict, Tuple

def calculate_mean(values: List[float]) -> float:
    if not values:
        return 0.0
    return sum(values) / len(values)

def calculate_median(values: List[float]) -> float:
    if not values:
        return 0.0
    sorted_vals = sorted(values)
    n = len(sorted_vals)
    mid = n // 2
    if n % 2 == 0:
        return (sorted_vals[mid - 1] + sorted_vals[mid]) / 2.0
    return sorted_vals[mid]

def calculate_stdev(values: List[float], mean: float) -> float:
    if len(values) < 2:
        return 0.0
    variance = sum((x - mean) ** 2 for x in values) / (len(values) - 1)
    return math.sqrt(variance)

def calculate_confidence_interval(mean: float, stdev: float, n: int, z_score: float = 1.96) -> Tuple[float, float]:
    """Calculate 95% CI by default (z=1.96)."""
    if n < 2:
        return (mean, mean)
    margin_of_error = z_score * (stdev / math.sqrt(n))
    return (mean - margin_of_error, mean + margin_of_error)

def calculate_percentage_improvement(baseline: float, new_value: float, higher_is_better: bool = True) -> float:
    if baseline == 0:
        return 0.0
    diff = new_value - baseline
    if not higher_is_better:
        diff = baseline - new_value
    return (diff / baseline) * 100.0

def compute_statistics(values: List[float]) -> Dict[str, float]:
    n = len(values)
    if n == 0:
        return {"mean": 0.0, "median": 0.0, "stdev": 0.0, "ci_lower": 0.0, "ci_upper": 0.0, "min": 0.0, "max": 0.0, "n": 0}
    
    mean = calculate_mean(values)
    median = calculate_median(values)
    stdev = calculate_stdev(values, mean)
    ci_lower, ci_upper = calculate_confidence_interval(mean, stdev, n)
    
    return {
        "mean": mean,
        "median": median,
        "stdev": stdev,
        "ci_lower": ci_lower,
        "ci_upper": ci_upper,
        "min": min(values),
        "max": max(values),
        "n": n
    }

def analyze_statistical_significance(baseline_stats: Dict[str, float], new_stats: Dict[str, float], higher_is_better: bool = True) -> Dict[str, any]:
    """
    Very basic significance check using non-overlapping confidence intervals.
    For a more rigorous test, use scipy.stats.ttest_ind in production.
    """
    if baseline_stats["n"] < 2 or new_stats["n"] < 2:
        return {"significant": False, "improvement_pct": 0.0, "reason": "Insufficient sample size (n<2)"}

    pct_imp = calculate_percentage_improvement(baseline_stats["mean"], new_stats["mean"], higher_is_better)
    
    # Check if CI overlap
    overlap = False
    if baseline_stats["ci_upper"] >= new_stats["ci_lower"] and baseline_stats["ci_lower"] <= new_stats["ci_upper"]:
        overlap = True
        
    significant = not overlap
    # If the change is in the wrong direction, we don't call it an "improvement"
    if pct_imp <= 0:
        significant = False

    reason = "95% Confidence Intervals do not overlap" if not overlap else "95% Confidence Intervals overlap"

    return {
        "significant": significant,
        "improvement_pct": pct_imp,
        "reason": reason
    }

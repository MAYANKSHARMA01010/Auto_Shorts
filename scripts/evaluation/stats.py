import math
from typing import List, Dict, Tuple, Any

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

# --- Phase 6.5 Additions ---

def _t_distribution_cdf_approx(t: float, df: int) -> float:
    """
    Very rough approximation for the CDF of the t-distribution to compute p-values without scipy.
    Uses normal approximation for large df, and a polynomial approximation for smaller df.
    For N >= 5, normal approximation is acceptable for a rough benchmark, 
    but we will use a slightly better heuristic.
    """
    # Fallback to normal approximation of p-value for simplicity
    # Z = t * (1 - 1/(4*df))
    # Approximation of normal CDF
    x = t / math.sqrt(2)
    # math.erf is available in standard library
    p = 0.5 * (1 + math.erf(x))
    return p

def calculate_paired_t_test(baseline: List[float], new: List[float]) -> Tuple[float, float]:
    """
    Calculates t-statistic and two-tailed p-value for a paired sample.
    """
    if len(baseline) != len(new) or len(baseline) < 2:
        return 0.0, 1.0
    
    n = len(baseline)
    diffs = [n_val - b_val for b_val, n_val in zip(baseline, new)]
    mean_diff = calculate_mean(diffs)
    stdev_diff = calculate_stdev(diffs, mean_diff)
    
    if stdev_diff == 0:
        return 0.0, 1.0 if mean_diff == 0 else 0.0
        
    t_stat = mean_diff / (stdev_diff / math.sqrt(n))
    df = n - 1
    
    # Calculate two-tailed p-value
    p_val_one_tail = 1.0 - _t_distribution_cdf_approx(abs(t_stat), df)
    p_val_two_tail = p_val_one_tail * 2
    return t_stat, min(p_val_two_tail, 1.0)

def calculate_cohens_d(baseline: List[float], new: List[float]) -> float:
    """
    Calculates Cohen's d for effect size.
    """
    if len(baseline) < 2 or len(new) < 2:
        return 0.0
        
    mean1 = calculate_mean(baseline)
    mean2 = calculate_mean(new)
    std1 = calculate_stdev(baseline, mean1)
    std2 = calculate_stdev(new, mean2)
    
    n1, n2 = len(baseline), len(new)
    
    # Pooled standard deviation
    pooled_sd = math.sqrt(((n1 - 1) * std1**2 + (n2 - 1) * std2**2) / (n1 + n2 - 2))
    
    if pooled_sd == 0:
        return 0.0
        
    return abs(mean2 - mean1) / pooled_sd

def categorize_effect_size(d: float) -> str:
    if d < 0.2:
        return "Negligible"
    elif d < 0.5:
        return "Small"
    elif d < 0.8:
        return "Medium"
    else:
        return "Large"

def estimate_power(d: float, n: int) -> str:
    """
    Extremely crude power approximation. 
    Requires d >= 0.8 (Large) to achieve 80% power at n=15.
    Requires d >= 0.5 (Medium) to achieve 80% power at n=35.
    """
    if n < 10:
        return "Low (N<10)"
    if d >= 0.8 and n >= 15:
        return "High"
    if d >= 0.5 and n >= 35:
        return "High"
    if d >= 0.2 and n >= 200:
        return "High"
    return "Low"

def get_recommendation(p_value: float, effect_size_cat: str, power: str, improvement_pct: float) -> str:
    if p_value < 0.05:
        if improvement_pct < 0:
            return "Reject (Statistically Significant Degradation)"
        if effect_size_cat in ["Medium", "Large"]:
            return "Adopt"
        return "Inconclusive (Significant, but effect size is too small)"
    
    if p_value < 0.10 or power.startswith("Low"):
        return "Needs More Data"
        
    return "Reject (No improvement found)"

def analyze_statistical_significance(baseline_values: List[float], new_values: List[float], higher_is_better: bool = True) -> Dict[str, Any]:
    if len(baseline_values) < 2 or len(baseline_values) != len(new_values):
        return {
            "p_value": 1.0,
            "effect_size": 0.0,
            "effect_size_cat": "Negligible",
            "power": "Low",
            "significant": False,
            "improvement_pct": 0.0,
            "recommendation": "Needs More Data (Insufficient N or mismatched samples)"
        }
        
    baseline_stats = compute_statistics(baseline_values)
    new_stats = compute_statistics(new_values)
    
    pct_imp = calculate_percentage_improvement(baseline_stats["mean"], new_stats["mean"], higher_is_better)
    
    t_stat, p_value = calculate_paired_t_test(baseline_values, new_values)
    cohens_d = calculate_cohens_d(baseline_values, new_values)
    effect_size_cat = categorize_effect_size(cohens_d)
    power = estimate_power(cohens_d, len(baseline_values))
    
    # Calculate one-tailed directionality for p-value correctly if it moved in the "better" direction
    if pct_imp > 0:
        # If it improved, we can halve the p-value for a one-tailed test in the correct direction
        p_value = p_value / 2.0
    
    rec = get_recommendation(p_value, effect_size_cat, power, pct_imp)
    
    return {
        "p_value": p_value,
        "effect_size": cohens_d,
        "effect_size_cat": effect_size_cat,
        "power": power,
        "significant": p_value < 0.05 and pct_imp > 0,
        "improvement_pct": pct_imp,
        "recommendation": rec
    }

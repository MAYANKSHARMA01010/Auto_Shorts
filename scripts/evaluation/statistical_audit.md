# Statistical Audit: Flaws in Phase 6 Methodology

## The Problem with "Non-Overlapping Confidence Intervals"
In Phase 6, we implemented a crude statistical heuristic: an improvement is only deemed significant if the 95% Confidence Intervals (CIs) of the two distributions do not overlap.

### 1. High False Negative Rate (Type II Errors)
While non-overlapping CIs guarantee statistical significance (p < 0.05), the inverse is false. **Two groups can have overlapping 95% CIs and still be statistically significantly different.** By requiring non-overlap, we created a test that is extremely conservative. We were likely rejecting genuine improvements simply because their CIs touched slightly.

### 2. Ignoring Paired Data
Our benchmark runs the *exact same* videos across both the baseline and new mode. Treating them as independent samples (which the CI overlap method does) throws away massive amounts of statistical power. When you control for the video itself, variance drops significantly. A Paired t-test is required to measure the difference *within* each video execution, rather than comparing the aggregate distributions independently.

### 3. Lack of Effect Size
Knowing a result is statistically significant only tells us that it is not zero. It does not tell us if the improvement actually matters. A 0.1s latency improvement might be statistically significant over 10,000 runs, but it is practically meaningless. We must calculate **Cohen's d** to measure the magnitude of the effect.

### 4. Ignoring Power
Running N=5 repetitions on 4 videos gives 20 paired samples. Depending on the variance of Ollama generation, this might not be enough power (usually >0.8 is desired) to reliably detect a medium effect size. 

## The Solution
We will implement:
1. **Paired t-test** (yielding a true p-value).
2. **Cohen's d** for effect size categorization.
3. **Power Approximation** to warn when sample sizes are dangerously low.

import re

with open('frontend/src/app/page.tsx', 'r') as f:
    content = f.read()

# 1. Add Types
types = r'''type ViralPrediction = {
  id: string;
  candidateId: string;
  evaluationSummary: string;
  avgWatchPercentage: string;
  watchToEndLikelihood: string;
  bestTargetAudience: string;
  bestPlatform: string;
  bestPostingTime: string;
  bestTitle: string;
  bestThumbnailText: string;
  bestHashtags: string[];
  viralReason: string;
  failureReason: string;
  singleImprovement: string;
  viralProbability: number;
  confidence: number;
};

type CandidateReview = {'''
content = re.sub(r'type CandidateReview = \{', types, content)

# 2. Add prediction?: ViralPrediction
content = re.sub(r'  review\?: CandidateReview;\n\};', r'  review?: CandidateReview;\n  prediction?: ViralPrediction;\n};', content)

# 3. Add states
states = r'''  const [predictingCandidateId, setPredictingCandidateId] = useState<string | null>(null);
  const [showPredictionModal, setShowPredictionModal] = useState<ViralPrediction | null>(null);'''
content = re.sub(r'(  const \[showReviewModal, setShowReviewModal\] = useState<CandidateReview \| null>\(null\);)', r'\1\n' + states, content)

# 4. Add predict function
predict_fn = r'''
  const handlePredictViral = async (candidateId: string) => {
    try {
      setPredictingCandidateId(candidateId);
      const prediction = await invoke<ViralPrediction>("predict_viral_cmd", {
        candidateId,
        provider: llmEngine,
        apiKey: 
          llmEngine === "claude" ? anthropicKey :
          llmEngine === "deepseek" ? deepseekKey :
          llmEngine === "gemini" ? geminiKey :
          llmEngine === "openai" ? openaiKey :
          llmEngine === "openrouter" ? openrouterKey : null,
        modelName: llmEngine === "local" ? localLlmModel : null,
      });
      console.log("Prediction complete:", prediction);
      refreshDetail();
    } catch (err: any) {
      setError("Prediction failed: " + err);
    } finally {
      setPredictingCandidateId(null);
    }
  };
'''
content = re.sub(r'(  const handleReviewCandidate = async)', predict_fn + r'\n\1', content)

# 5. Add UI to Candidate Card
card_insert = r'''
                          {candidate.prediction ? (
                            <div className="mt-4 p-3 bg-fuchsia-900/30 rounded-md border border-fuchsia-700/50 flex flex-col gap-2">
                              <div className="flex items-center justify-between">
                                <span className="text-sm font-semibold text-fuchsia-300 flex items-center gap-2">
                                  <TrendingUp size={16} className="text-fuchsia-400" /> Viral Prediction
                                </span>
                                <div className="flex items-center gap-2">
                                  <span className="bg-fuchsia-500/20 text-fuchsia-400 px-2 py-0.5 text-xs font-bold rounded flex items-center gap-1">
                                    🔥 {candidate.prediction.viralProbability}% Viral
                                  </span>
                                </div>
                              </div>
                              <button
                                onClick={() => setShowPredictionModal(candidate.prediction!)}
                                className="text-xs text-fuchsia-400 hover:text-fuchsia-300 text-left underline"
                              >
                                View Strategy & Metadata
                              </button>
                            </div>
                          ) : (
                            <button
                              onClick={() => handlePredictViral(candidate.id)}
                              disabled={predictingCandidateId === candidate.id}
                              className="mt-4 w-full flex items-center justify-center gap-2 bg-slate-700/50 hover:bg-slate-700 text-slate-300 py-1.5 rounded text-sm transition-colors"
                            >
                              {predictingCandidateId === candidate.id ? (
                                <><Loader2 size={16} className="animate-spin" /> Predicting...</>
                              ) : (
                                <><TrendingUp size={16} /> Predict Virality</>
                              )}
                            </button>
                          )}
'''
# I need to insert this right after the review section in the card.
# The review section ends with `)}` and then there's `</div>` and `</div>`.
content = re.sub(r'(                                <><BadgeCheck size=\{16\} /> Critique AI Clip</>\n                              \)}\n                            </button>\n                          \)}\n)', r'\1' + card_insert, content)

# Wait, `TrendingUp` needs to be imported from 'lucide-react'
content = re.sub(r'BadgeCheck,', r'BadgeCheck, TrendingUp,', content)

# 6. Add Prediction Modal at the end
modal = r'''
      {/* Prediction Modal */}
      {showPredictionModal && (
        <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4">
          <div className="bg-slate-800 rounded-lg shadow-xl border border-slate-700 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
            <div className="p-6">
              <div className="flex justify-between items-start mb-6">
                <div>
                  <h2 className="text-xl font-bold flex items-center gap-2 text-fuchsia-400">
                    <TrendingUp className="text-fuchsia-400" /> Viral Growth Strategy
                  </h2>
                  <div className="flex gap-2 mt-2">
                    <span className="bg-fuchsia-500/20 text-fuchsia-400 px-3 py-1 text-sm font-bold rounded flex items-center gap-1">
                      🔥 Viral Probability: {showPredictionModal.viralProbability}%
                    </span>
                    <span className="bg-slate-700 px-3 py-1 text-sm font-bold rounded text-slate-300">
                      Confidence: {showPredictionModal.confidence}%
                    </span>
                  </div>
                </div>
                <button
                  onClick={() => setShowPredictionModal(null)}
                  className="text-slate-400 hover:text-white"
                >
                  ✕
                </button>
              </div>
              
              <div className="bg-slate-900/50 p-4 rounded-lg border border-slate-700 mb-6">
                <h3 className="font-semibold text-slate-300 mb-2">Psychological Evaluation</h3>
                <p className="text-sm text-slate-400 italic">"{showPredictionModal.evaluationSummary}"</p>
              </div>

              <div className="grid grid-cols-2 gap-4 mb-6">
                <div className="space-y-4">
                  <div className="bg-slate-900/30 p-3 rounded-lg border border-slate-700/50">
                    <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-2">Metrics</h3>
                    <div className="flex justify-between text-sm mb-1"><span className="text-slate-400">Est. Watch %</span> <span className="font-medium">{showPredictionModal.avgWatchPercentage}</span></div>
                    <div className="flex justify-between text-sm"><span className="text-slate-400">Watch to End</span> <span className="font-medium">{showPredictionModal.watchToEndLikelihood}</span></div>
                  </div>
                  
                  <div className="bg-slate-900/30 p-3 rounded-lg border border-slate-700/50">
                    <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-2">Targeting</h3>
                    <div className="flex justify-between text-sm mb-1"><span className="text-slate-400">Audience</span> <span className="font-medium text-right">{showPredictionModal.bestTargetAudience}</span></div>
                    <div className="flex justify-between text-sm mb-1"><span className="text-slate-400">Platform</span> <span className="font-medium">{showPredictionModal.bestPlatform}</span></div>
                    <div className="flex justify-between text-sm"><span className="text-slate-400">Post Time</span> <span className="font-medium">{showPredictionModal.bestPostingTime}</span></div>
                  </div>
                </div>

                <div className="space-y-4">
                  <div className="bg-slate-900/30 p-3 rounded-lg border border-slate-700/50 h-full">
                    <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-2">Packaging</h3>
                    <div className="mb-3">
                      <span className="text-slate-400 text-xs block mb-1">Suggested Title</span>
                      <p className="text-sm font-medium text-indigo-300">{showPredictionModal.bestTitle}</p>
                    </div>
                    {showPredictionModal.bestThumbnailText && (
                      <div className="mb-3">
                        <span className="text-slate-400 text-xs block mb-1">Thumbnail Text</span>
                        <p className="text-sm font-bold bg-yellow-500 text-black px-2 py-1 inline-block rounded">{showPredictionModal.bestThumbnailText}</p>
                      </div>
                    )}
                    <div>
                      <span className="text-slate-400 text-xs block mb-1">Hashtags</span>
                      <div className="flex flex-wrap gap-1">
                        {showPredictionModal.bestHashtags.map((tag, i) => (
                          <span key={i} className="text-xs text-blue-400 bg-blue-500/10 px-1.5 py-0.5 rounded">{tag}</span>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <h3 className="font-semibold text-slate-300 mb-3 border-b border-slate-700 pb-2">Analysis</h3>
              <div className="space-y-3">
                <div className="flex items-start gap-3">
                  <span className="text-emerald-400 mt-0.5 text-lg">🚀</span>
                  <div>
                    <span className="text-xs font-semibold text-emerald-400 uppercase">Why it will work</span>
                    <p className="text-sm text-slate-300">{showPredictionModal.viralReason}</p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <span className="text-red-400 mt-0.5 text-lg">⚠️</span>
                  <div>
                    <span className="text-xs font-semibold text-red-400 uppercase">Risk factors</span>
                    <p className="text-sm text-slate-300">{showPredictionModal.failureReason}</p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <span className="text-yellow-400 mt-0.5 text-lg">💡</span>
                  <div>
                    <span className="text-xs font-semibold text-yellow-400 uppercase">Top recommendation</span>
                    <p className="text-sm text-slate-300 font-medium">{showPredictionModal.singleImprovement}</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
'''
content = re.sub(r'(      \{showReviewModal && \()', modal + r'\n\1', content)

with open('frontend/src/app/page.tsx', 'w') as f:
    f.write(content)

print("Patched page.tsx for prediction")

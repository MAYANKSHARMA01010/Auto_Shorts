import re

with open('frontend/src/app/page.tsx', 'r') as f:
    content = f.read()

# 1. Add Types
types = r'''type ReviewScores = {
  storyCompleteness: number;
  hookStrength: number;
  context: number;
  flow: number;
  segmentSelection: number;
  pacing: number;
  ending: number;
  captionQuality: number;
  viralPotential: number;
  viewerRetention: number;
};

type ReviewFlaw = {
  issue: string;
  improvement: string;
};

type CandidateReview = {
  id: string;
  candidateId: string;
  decision: string;
  overallScore: number;
  confidence: number;
  scores: ReviewScores;
  flaws: ReviewFlaw[];
};

type Candidate = {'''
content = re.sub(r'type Candidate = \{', types, content)

# 2. Add review?: CandidateReview
content = re.sub(r'  description\?: string;\n\};', r'  description?: string;\n  review?: CandidateReview;\n};', content)

# 3. Add states
states = r'''  const [reviewingCandidateId, setReviewingCandidateId] = useState<string | null>(null);
  const [showReviewModal, setShowReviewModal] = useState<CandidateReview | null>(null);'''
content = re.sub(r'(  const \[editingSegmentText, setEditingSegmentText\] = useState\(""\);)', r'\1\n' + states, content)

# 4. Add review function
review_fn = r'''
  const handleReviewCandidate = async (candidateId: string) => {
    try {
      setReviewingCandidateId(candidateId);
      const review = await invoke<CandidateReview>("review_candidate_cmd", {
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
      console.log("Review complete:", review);
      refreshDetail();
    } catch (err: any) {
      setError("Review failed: " + err);
    } finally {
      setReviewingCandidateId(null);
    }
  };
'''
content = re.sub(r'(  const refreshDetail = async \(\) => \{)', review_fn + r'\n\1', content)

# 5. Add UI to Candidate Card
card_insert = r'''
                          {candidate.review ? (
                            <div className="mt-4 p-3 bg-slate-900 rounded-md border border-slate-700 flex flex-col gap-2">
                              <div className="flex items-center justify-between">
                                <span className="text-sm font-semibold text-slate-300 flex items-center gap-2">
                                  <BadgeCheck size={16} className="text-indigo-400" /> AI Editor Review
                                </span>
                                <div className="flex items-center gap-2">
                                  <span className={`px-2 py-0.5 text-xs font-bold rounded ${
                                    candidate.review.decision === "APPROVE" ? "bg-emerald-500/20 text-emerald-400" :
                                    candidate.review.decision === "REVISION_REQUIRED" ? "bg-yellow-500/20 text-yellow-400" :
                                    "bg-red-500/20 text-red-400"
                                  }`}>
                                    {candidate.review.decision}
                                  </span>
                                  <span className="bg-indigo-500/20 text-indigo-400 px-2 py-0.5 text-xs font-bold rounded">
                                    {candidate.review.overallScore}/100
                                  </span>
                                </div>
                              </div>
                              <button
                                onClick={() => setShowReviewModal(candidate.review!)}
                                className="text-xs text-indigo-400 hover:text-indigo-300 text-left underline"
                              >
                                View Detailed Critique & Flaws
                              </button>
                            </div>
                          ) : (
                            <button
                              onClick={() => handleReviewCandidate(candidate.id)}
                              disabled={reviewingCandidateId === candidate.id}
                              className="mt-4 w-full flex items-center justify-center gap-2 bg-slate-700/50 hover:bg-slate-700 text-slate-300 py-1.5 rounded text-sm transition-colors"
                            >
                              {reviewingCandidateId === candidate.id ? (
                                <><Loader2 size={16} className="animate-spin" /> Reviewing...</>
                              ) : (
                                <><BadgeCheck size={16} /> Critique AI Clip</>
                              )}
                            </button>
                          )}
'''
# Find where rationale ends
content = re.sub(r'(<p className="text-xs text-slate-400 italic mt-1">\{candidate\.rationale\}</p>)', r'\1\n' + card_insert, content)

# 6. Add Review Modal at the end
modal = r'''
      {/* Review Modal */}
      {showReviewModal && (
        <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4">
          <div className="bg-slate-800 rounded-lg shadow-xl border border-slate-700 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
            <div className="p-6">
              <div className="flex justify-between items-start mb-6">
                <div>
                  <h2 className="text-xl font-bold flex items-center gap-2">
                    <BadgeCheck className="text-indigo-400" /> Senior Editor Critique
                  </h2>
                  <div className="flex gap-2 mt-2">
                    <span className={`px-2 py-1 text-sm font-bold rounded ${
                      showReviewModal.decision === "APPROVE" ? "bg-emerald-500/20 text-emerald-400" :
                      showReviewModal.decision === "REVISION_REQUIRED" ? "bg-yellow-500/20 text-yellow-400" :
                      "bg-red-500/20 text-red-400"
                    }`}>
                      {showReviewModal.decision}
                    </span>
                    <span className="bg-indigo-500/20 text-indigo-400 px-2 py-1 text-sm font-bold rounded">
                      Score: {showReviewModal.overallScore}/100
                    </span>
                    <span className="bg-slate-700 px-2 py-1 text-sm font-bold rounded text-slate-300">
                      Confidence: {showReviewModal.confidence}%
                    </span>
                  </div>
                </div>
                <button
                  onClick={() => setShowReviewModal(null)}
                  className="text-slate-400 hover:text-white"
                >
                  ✕
                </button>
              </div>

              <div className="grid grid-cols-2 gap-4 mb-8">
                <div className="space-y-2">
                  <h3 className="font-semibold text-slate-300 mb-3 border-b border-slate-700 pb-1">Story & Flow</h3>
                  <div className="flex justify-between text-sm"><span className="text-slate-400">Completeness</span> <span>{showReviewModal.scores.storyCompleteness}/10</span></div>
                  <div className="flex justify-between text-sm"><span className="text-slate-400">Context</span> <span>{showReviewModal.scores.context}/10</span></div>
                  <div className="flex justify-between text-sm"><span className="text-slate-400">Flow</span> <span>{showReviewModal.scores.flow}/10</span></div>
                  <div className="flex justify-between text-sm"><span className="text-slate-400">Ending</span> <span>{showReviewModal.scores.ending}/10</span></div>
                  <div className="flex justify-between text-sm"><span className="text-slate-400">Pacing</span> <span>{showReviewModal.scores.pacing}/10</span></div>
                </div>
                <div className="space-y-2">
                  <h3 className="font-semibold text-slate-300 mb-3 border-b border-slate-700 pb-1">Viral Factors</h3>
                  <div className="flex justify-between text-sm"><span className="text-slate-400">Hook Strength</span> <span>{showReviewModal.scores.hookStrength}/10</span></div>
                  <div className="flex justify-between text-sm"><span className="text-slate-400">Segment Selection</span> <span>{showReviewModal.scores.segmentSelection}/10</span></div>
                  <div className="flex justify-between text-sm"><span className="text-slate-400">Viral Potential</span> <span>{showReviewModal.scores.viralPotential}/10</span></div>
                  <div className="flex justify-between text-sm"><span className="text-slate-400">Retention</span> <span>{showReviewModal.scores.viewerRetention}/10</span></div>
                  <div className="flex justify-between text-sm"><span className="text-slate-400">Caption Quality</span> <span>{showReviewModal.scores.captionQuality}/10</span></div>
                </div>
              </div>

              <h3 className="font-semibold text-slate-300 mb-4 border-b border-slate-700 pb-2">Identified Flaws & Improvements</h3>
              {showReviewModal.flaws.length === 0 ? (
                <p className="text-sm text-slate-400 italic">No major flaws identified.</p>
              ) : (
                <div className="space-y-4">
                  {showReviewModal.flaws.map((flaw, idx) => (
                    <div key={idx} className="bg-slate-900/50 p-4 rounded-lg border border-slate-700">
                      <div className="flex items-start gap-2 mb-2">
                        <span className="text-red-400 mt-0.5">⚠️</span>
                        <div>
                          <p className="text-sm font-semibold text-slate-200">{flaw.issue}</p>
                        </div>
                      </div>
                      <div className="flex items-start gap-2 ml-6">
                        <span className="text-emerald-400 mt-0.5">💡</span>
                        <div>
                          <p className="text-sm text-slate-300">{flaw.improvement}</p>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
'''
content = re.sub(r'(      \{showStyleModal && \()', modal + r'\n\1', content)

with open('frontend/src/app/page.tsx', 'w') as f:
    f.write(content)

print("Patched page.tsx")

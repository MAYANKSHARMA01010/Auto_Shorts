import re

with open('frontend/src/app/page.tsx', 'r') as f:
    content = f.read()

# 1. Update Candidate type
type_old = r'type Candidate = \{\n  id: string;\n  projectId: string;\n  startSec: number;\n  endSec: number;'
type_new = r'type Candidate = {\n  id: string;\n  projectId: string;\n  segments: { start: number; end: number }[];'
content = re.sub(type_old, type_new, content)

# 2. Update invoke("add_manual_candidate") args
manual_old = r'        projectId: detail\.project\.id,\n        startSec: start,\n        endSec: end,'
manual_new = r'        projectId: detail.project.id,\n        segments: [{ start: start, end: end }],'
content = re.sub(manual_old, manual_new, content)

# 3. Update candidate rendering
render_old = r'<span>\{formatTime\(candidate\.startSec\)\} - \{formatTime\(candidate\.endSec\)\}</span>'
render_new = r'''<div className="flex flex-wrap gap-1">
                                {candidate.segments.map((seg, i) => (
                                  <span key={i} className="bg-slate-700/50 px-1 rounded">
                                    {formatTime(seg.start)} - {formatTime(seg.end)}
                                  </span>
                                ))}
                              </div>'''
content = re.sub(render_old, render_new, content)

with open('frontend/src/app/page.tsx', 'w') as f:
    f.write(content)

print("Patched frontend")

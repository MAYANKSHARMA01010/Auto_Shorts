import re

with open('src-tauri/src/db.rs', 'r') as f:
    content = f.read()

# Replace self.conn.lock().unwrap() with self.conn.lock().map_err(|e| anyhow::anyhow!("DB lock poisoned: {}", e))?
content = content.replace(
    'self.conn.lock().unwrap()', 
    'self.conn.lock().map_err(|e| anyhow::anyhow!("DB lock poisoned: {}", e))?'
)

with open('src-tauri/src/db.rs', 'w') as f:
    f.write(content)
print("Patched db.rs")

import { confirm } from '@tauri-apps/plugin-dialog';

export async function runTest() {
  const result = await confirm('test');
  return result;
}

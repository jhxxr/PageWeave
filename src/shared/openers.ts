import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import { message } from "antd";

function errorText(e: unknown) {
  return e instanceof Error ? e.message : String(e);
}

export async function openFilePath(path: string) {
  try {
    await openPath(path);
  } catch (e) {
    message.error(`打开文件失败：${errorText(e)}`);
  }
}

export async function revealFilePath(path: string) {
  try {
    await revealItemInDir(path);
  } catch (e) {
    message.error(`打开所在文件夹失败：${errorText(e)}`);
  }
}

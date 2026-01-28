import { zLspCursorResponse } from "./decorations";

export const analyze = async (
  source: string,
  line: number,
  character: number,
) => {
  try {
    const resp = await fetch("/api/analyze", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ source, line, character }),
    });
    const parsed = zLspCursorResponse.parse(await resp.json());
    return parsed;
  } catch (_e) {
    return null;
  }
};

export const health = async () => {
  try {
    await fetch("/health");
  } catch (_e) {}
};

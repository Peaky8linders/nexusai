import type { Conversation } from "../types";

export function exportAsMarkdown(conversation: Conversation): string {
  const lines: string[] = [
    `# ${conversation.title}`,
    "",
    `*Exported from NexusAI | ${new Date(conversation.createdAt).toLocaleString()}*`,
    `*Model: ${conversation.modelId || "unknown"}*`,
    "",
    "---",
    "",
  ];

  for (const msg of conversation.messages) {
    const role = msg.role === "user" ? "**You**" : "**NexusAI**";
    const time = new Date(msg.timestamp).toLocaleTimeString();
    lines.push(`### ${role} (${time})`);
    lines.push("");
    lines.push(msg.content);
    lines.push("");
  }

  return lines.join("\n");
}

export function exportAsJson(conversation: Conversation): string {
  return JSON.stringify(
    {
      id: conversation.id,
      title: conversation.title,
      model: conversation.modelId,
      created: new Date(conversation.createdAt).toISOString(),
      messages: conversation.messages.map((m) => ({
        role: m.role,
        content: m.content,
        timestamp: new Date(m.timestamp).toISOString(),
      })),
    },
    null,
    2,
  );
}

export function downloadFile(content: string, filename: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

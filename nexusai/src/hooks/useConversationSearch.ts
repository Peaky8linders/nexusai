import { useMemo, useState } from "react";
import { useAppStore } from "../stores/appStore";

export function useConversationSearch() {
  const [query, setQuery] = useState("");
  const conversations = useAppStore((s) => s.conversations);

  const results = useMemo(() => {
    if (!query.trim()) return conversations;

    const lowerQuery = query.toLowerCase();
    return conversations.filter((conv) => {
      // Search title
      if (conv.title.toLowerCase().includes(lowerQuery)) return true;
      // Search messages
      return conv.messages.some((msg) =>
        msg.content.toLowerCase().includes(lowerQuery),
      );
    });
  }, [query, conversations]);

  return { query, setQuery, results };
}

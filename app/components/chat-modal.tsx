"use client";

import { useState, useEffect, useRef } from "react";
import { getAgent, chatWithAgent, type Agent } from "@/app/actions";

interface Message {
  role: "user" | "agent";
  content: string;
  timestamp: Date;
}

interface ChatModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function ChatModal({ isOpen, onClose }: ChatModalProps) {
  const [agent, setAgent] = useState<Agent | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isInitializing, setIsInitializing] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Fetch agent on mount
  useEffect(() => {
    if (isOpen) {
      setIsInitializing(true);
      setError(null);
      getAgent()
        .then((result) => {
          if (result.success && result.agent) {
            setAgent(result.agent);
            // Initialize with a welcome message
            setMessages([
              {
                role: "agent",
                content: `Hello! I'm ${result.agent.name}. How can I help you today?`,
                timestamp: new Date(),
              },
            ]);
          } else {
            setError(result.error || "Failed to load agent");
          }
        })
        .catch((err) => {
          setError("Failed to load agent");
          console.error(err);
        })
        .finally(() => {
          setIsInitializing(false);
        });
    } else {
      // Reset when modal closes
      setMessages([]);
      setInput("");
      setError(null);
    }
  }, [isOpen]);

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Focus input when modal opens
  useEffect(() => {
    if (isOpen && !isInitializing) {
      setTimeout(() => {
        inputRef.current?.focus();
      }, 100);
    }
  }, [isOpen, isInitializing]);

  const handleSend = async () => {
    if (!input.trim() || !agent || isLoading) return;

    const userMessage: Message = {
      role: "user",
      content: input.trim(),
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput("");
    setIsLoading(true);
    setError(null);

    try {
      const response = await chatWithAgent(agent.id, userMessage.content);
      
      if (response.success && response.response) {
        const agentMessage: Message = {
          role: "agent",
          content: response.response,
          timestamp: new Date(),
        };
        setMessages((prev) => [...prev, agentMessage]);
      } else {
        setError(response.error || "Failed to get response");
        // Remove the user message if it failed
        setMessages((prev) => prev.slice(0, -1));
      }
    } catch (err) {
      setError("Failed to send message");
      console.error(err);
      // Remove the user message if it failed
      setMessages((prev) => prev.slice(0, -1));
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative z-10 flex h-full max-h-[90vh] w-full max-w-2xl flex-col rounded-2xl border border-border bg-bg-light shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-border p-4">
          <div className="flex items-center gap-3">
            {agent && (
              <>
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-action-primary/20 text-action-primary">
                  <span className="text-lg font-semibold">
                    {agent.name.charAt(0).toUpperCase()}
                  </span>
                </div>
                <div>
                  <h2 className="text-lg font-semibold text-text">{agent.name}</h2>
                  <p className="text-sm text-text-muted">
                    {agent.status === "online" ? (
                      <span className="flex items-center gap-1">
                        <span className="h-2 w-2 rounded-full bg-alert-success"></span>
                        Online
                      </span>
                    ) : (
                      <span className="flex items-center gap-1">
                        <span className="h-2 w-2 rounded-full bg-text-muted"></span>
                        {agent.status}
                      </span>
                    )}
                  </p>
                </div>
              </>
            )}
            {!agent && isInitializing && (
              <div className="text-text-muted">Loading agent...</div>
            )}
          </div>
          <button
            onClick={onClose}
            className="rounded-lg p-2 text-text-muted transition-colors hover:bg-bg hover:text-text"
            aria-label="Close chat"
          >
            <svg
              className="h-5 w-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Messages */}
        <div className="flex-1 overflow-y-auto p-4">
          {error && (
            <div className="mb-4 rounded-lg bg-alert-danger/10 border border-alert-danger/20 p-3 text-sm text-alert-danger">
              {error}
            </div>
          )}
          
          {isInitializing ? (
            <div className="flex items-center justify-center py-8 text-text-muted">
              <div className="flex items-center gap-2">
                <div className="h-4 w-4 animate-spin rounded-full border-2 border-action-primary border-t-transparent"></div>
                <span>Connecting to agent...</span>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              {messages.map((message, index) => (
                <div
                  key={index}
                  className={`flex ${
                    message.role === "user" ? "justify-end" : "justify-start"
                  }`}
                >
                  <div
                    className={`max-w-[80%] rounded-2xl px-4 py-2 ${
                      message.role === "user"
                        ? "bg-action-primary text-bg-light"
                        : "bg-bg border border-border text-text"
                    }`}
                  >
                    <p className="whitespace-pre-wrap break-words">
                      {message.content}
                    </p>
                    <p
                      className={`mt-1 text-xs ${
                        message.role === "user"
                          ? "text-bg-light/70"
                          : "text-text-muted"
                      }`}
                    >
                      {message.timestamp.toLocaleTimeString([], {
                        hour: "2-digit",
                        minute: "2-digit",
                      })}
                    </p>
                  </div>
                </div>
              ))}
              {isLoading && (
                <div className="flex justify-start">
                  <div className="rounded-2xl bg-bg border border-border px-4 py-2">
                    <div className="flex items-center gap-2 text-text-muted">
                      <div className="h-2 w-2 animate-bounce rounded-full bg-action-primary"></div>
                      <div className="h-2 w-2 animate-bounce rounded-full bg-action-primary [animation-delay:0.2s]"></div>
                      <div className="h-2 w-2 animate-bounce rounded-full bg-action-primary [animation-delay:0.4s]"></div>
                    </div>
                  </div>
                </div>
              )}
              <div ref={messagesEndRef} />
            </div>
          )}
        </div>

        {/* Input */}
        {agent && !isInitializing && (
          <div className="border-t border-border p-4">
            <div className="flex gap-2">
              <textarea
                ref={inputRef}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Type your message..."
                disabled={isLoading}
                rows={2}
                className="flex-1 resize-none rounded-lg border border-border bg-bg px-4 py-2 text-text placeholder:text-text-muted focus:border-action-primary focus:outline-none disabled:opacity-50"
              />
              <button
                onClick={handleSend}
                disabled={!input.trim() || isLoading}
                className="rounded-lg bg-action-primary px-6 py-2 font-semibold text-bg-light transition-colors hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? (
                  <div className="h-5 w-5 animate-spin rounded-full border-2 border-bg-light border-t-transparent"></div>
                ) : (
                  "Send"
                )}
              </button>
            </div>
            <p className="mt-2 text-xs text-text-muted">
              Press Enter to send, Shift+Enter for new line
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

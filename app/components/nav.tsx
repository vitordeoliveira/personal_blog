"use client";

import Link from "next/link";
import { useState } from "react";
import { Bot } from "lucide-react";
import ThemeToggle from "./theme-toggle";
import { Button } from "./ui/button";
import ChatModal from "./chat-modal";

export default function Nav() {
  const [isChatOpen, setIsChatOpen] = useState(false);

  return (
    <>
      <header className="sticky top-0 z-50 w-full border-b border-border/50 bg-bg/80 backdrop-blur-lg">
        <div className="mx-auto flex h-16 items-center justify-between px-6 max-w-6xl">
          <Link href="/" className="flex items-center gap-2 font-bold text-xl">
            <span className="text-gradient">vitor.ws</span>
          </Link>

          <nav className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsChatOpen(true)}
              className="gap-2 text-text-muted hover:text-text"
            >
              <Bot className="h-4 w-4" />
              <span className="hidden sm:inline">Talk to AI</span>
            </Button>
            <ThemeToggle />
          </nav>
        </div>
      </header>
      <ChatModal isOpen={isChatOpen} onClose={() => setIsChatOpen(false)} />
    </>
  );
}


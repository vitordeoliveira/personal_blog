"use client";

import Link from "next/link";
import { Bot, AlertCircle } from "lucide-react";
import ThemeToggle from "./theme-toggle";
import { Button } from "./ui/button";

export default function Nav() {
  return (
    <header className="sticky top-0 z-50 w-full border-b border-border/50 bg-bg/80 backdrop-blur-lg">
      <div className="mx-auto flex h-16 items-center justify-between px-6 max-w-6xl">
        <Link href="/" className="flex items-center gap-2 font-bold text-xl">
          <span className="text-gradient">vitor.ws</span>
        </Link>

        <nav className="flex items-center gap-2">
          <div className="relative group">
            <Button
              variant="ghost"
              size="sm"
              disabled
              className="gap-2 text-text-muted opacity-60 cursor-not-allowed"
            >
              <Bot className="h-4 w-4" />
              <span className="hidden sm:inline">Talk to AI</span>
            </Button>
            <div className="absolute right-0 top-full mt-2 px-3 py-2 bg-bg-light border border-border rounded-lg shadow-lg text-xs text-text-muted whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
              <div className="flex items-center gap-2">
                <AlertCircle className="w-3 h-3 text-action-primary" />
                <span>Under maintenance</span>
              </div>
            </div>
          </div>
          <ThemeToggle />
        </nav>
      </div>
    </header>
  );
}


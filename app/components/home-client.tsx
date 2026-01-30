'use client';

import { useState } from "react";
import { Hero } from "./hero";
import BlogList from "./blog-list";
import ChatModal from "./chat-modal";
import type { Post } from "@/lib/posts";

interface HomeClientProps {
  recentPosts: Post[];
  allPostsCount: number;
}

export default function HomeClient({ recentPosts, allPostsCount }: HomeClientProps) {
  const [isChatOpen, setIsChatOpen] = useState(false);

  return (
    <>
      <main className="relative min-h-screen bg-bg">
        {/* Background decorative elements */}
        <div className="absolute inset-0 overflow-hidden">
          <div className="absolute -top-40 -right-40 h-96 w-96 rounded-full bg-action-primary/12 blur-3xl"></div>
          <div className="absolute -bottom-40 -left-40 h-96 w-96 rounded-full bg-action-primary/10 blur-3xl"></div>
          <div className="absolute top-1/3 right-1/4 h-72 w-72 rounded-full bg-action-primary/8 blur-3xl"></div>
          <div className="absolute bottom-1/4 left-1/3 h-64 w-64 rounded-full bg-action-primary/6 blur-3xl"></div>
          <div className="absolute top-1/2 left-1/2 h-[500px] w-[500px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-action-primary/5 blur-3xl"></div>
        </div>
        
        {/* Grid pattern overlay */}
        <div className="bg-pattern"></div>

        <Hero onOpenChat={() => setIsChatOpen(true)} />
        <div className="relative mx-auto max-w-5xl pb-12">
          <BlogList posts={recentPosts} allPostsCount={allPostsCount} />
        </div>
      </main>
      <ChatModal isOpen={isChatOpen} onClose={() => setIsChatOpen(false)} />
    </>
  );
}

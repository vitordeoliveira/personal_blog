"use client";

import { useEffect, useState, useRef } from "react";
import { trackPostView } from "@/app/actions";

export default function PostViewTracker({
  slug,
  initialViews,
}: {
  slug: string;
  initialViews: number;
}) {
  const [views, setViews] = useState(initialViews);
  const hasTrackedRef = useRef(false);

  useEffect(() => {
    // Only track once per page load, even if useEffect runs twice (React Strict Mode)
    if (!hasTrackedRef.current) {
      hasTrackedRef.current = true;
      trackPostView(slug).then((result) => {
        if (result.success) {
          setViews(result.views);
        }
      });
    }
  }, [slug]);

  return (
    <span className="flex items-center gap-1">
      <svg
        className="h-4 w-4"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
        />
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
        />
      </svg>
      {views.toLocaleString()} {views === 1 ? "view" : "views"}
    </span>
  );
}


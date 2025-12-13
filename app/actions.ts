"use server";

import { incrementViews, getPostMetadata } from "@/lib/db";

export async function trackPostView(slug: string) {
  try {
    incrementViews(slug);
    const metadata = getPostMetadata(slug);
    return { success: true, views: metadata?.views || 0 };
  } catch (error) {
    console.error("Failed to track view:", error);
    return { success: false, views: 0 };
  }
}


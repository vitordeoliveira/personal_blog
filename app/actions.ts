"use server";

import { incrementViews, getPostMetadata, updatePostViews } from "@/lib/db";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

const ADMIN_USERNAME = process.env.ADMIN_USERNAME || "admin";
const ADMIN_PASSWORD = process.env.ADMIN_PASSWORD || "admin";
const SESSION_COOKIE_NAME = "admin_session";

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

// Admin authentication
export async function loginAdmin(username: string, password: string) {
  if (username === ADMIN_USERNAME && password === ADMIN_PASSWORD) {
    const cookieStore = await cookies();
    cookieStore.set(SESSION_COOKIE_NAME, "authenticated", {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      maxAge: 60 * 60 * 24 * 7, // 7 days
    });
    return { success: true };
  }
  return { success: false, error: "Invalid credentials" };
}

export async function logoutAdmin() {
  const cookieStore = await cookies();
  cookieStore.delete(SESSION_COOKIE_NAME);
  redirect("/admin");
}

export async function isAdminAuthenticated(): Promise<boolean> {
  const cookieStore = await cookies();
  const session = cookieStore.get(SESSION_COOKIE_NAME);
  return session?.value === "authenticated";
}

export async function requireAdmin() {
  const isAuthenticated = await isAdminAuthenticated();
  if (!isAuthenticated) {
    redirect("/admin");
  }
}

// Get post metadata (for admin)
export async function getPostMetadataForAdmin(slug: string) {
  try {
    await requireAdmin();
    const metadata = getPostMetadata(slug);
    return { success: true, views: metadata?.views || 0 };
  } catch (error) {
    return { success: false, views: 0, error: "Unauthorized" };
  }
}

// Update post metadata views
export async function updatePostMetadataViews(slug: string, views: number) {
  try {
    await requireAdmin();
    updatePostViews(slug, views);
    return { success: true };
  } catch (error) {
    console.error("Failed to update post views:", error);
    return { success: false, error: "Failed to update views" };
  }
}


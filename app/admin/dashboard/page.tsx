import { getAllPostsForAdmin } from "@/lib/posts";
import { requireAdmin, logoutAdmin } from "@/app/actions";
import { redirect } from "next/navigation";
import PostsTable from "./posts-table";

export const dynamic = 'force-dynamic';

export default async function AdminDashboardPage() {
  // Check authentication
  try {
    await requireAdmin();
  } catch {
    redirect("/admin");
  }

  const posts = await getAllPostsForAdmin();

  return (
    <div className="relative min-h-screen bg-bg">
      <div className="bg-pattern"></div>
      
      <main className="relative mx-auto max-w-6xl px-6 py-16">
        <div className="flex items-center justify-between mb-8">
          <h1 className="text-4xl font-bold text-text">Admin Dashboard</h1>
          <form action={logoutAdmin}>
            <button
              type="submit"
              className="px-4 py-2 rounded-lg bg-action-secondary text-white font-medium hover:opacity-90 transition-opacity"
            >
              Logout
            </button>
          </form>
        </div>
        
        <PostsTable posts={posts} />
      </main>
    </div>
  );
}


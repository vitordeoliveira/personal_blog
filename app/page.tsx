import { getAllPosts } from "@/lib/posts";
import HomeClient from "@/app/components/home-client";

// Force dynamic rendering to ensure views are always up-to-date
export const dynamic = 'force-dynamic';

export default async function Home() {
  const allPosts = await getAllPosts();
  const recentPosts = allPosts.slice(0, 5);

  return <HomeClient recentPosts={recentPosts} allPostsCount={allPosts.length} />;
}

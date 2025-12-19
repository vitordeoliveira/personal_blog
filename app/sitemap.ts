import { MetadataRoute } from "next";
import { getAllPosts } from "@/lib/posts";

export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  const baseUrl = process.env.NEXT_PUBLIC_SITE_URL || "https://vitor.ws";
  
  // Get all blog posts
  const posts = await getAllPosts();
  
  // Home page
  const home: MetadataRoute.Sitemap[0] = {
    url: baseUrl,
    lastModified: new Date(),
    changeFrequency: "weekly",
    priority: 1,
  };
  
  // Blog listing page
  const blog: MetadataRoute.Sitemap[0] = {
    url: `${baseUrl}/blog`,
    lastModified: posts.length > 0 
      ? new Date(posts[0].date) 
      : new Date(),
    changeFrequency: "weekly",
    priority: 0.8,
  };
  
  // Individual blog posts
  const postUrls: MetadataRoute.Sitemap = posts.map((post) => ({
    url: `${baseUrl}/blog/${post.slug}`,
    lastModified: new Date(post.date),
    changeFrequency: "monthly" as const,
    priority: 0.6,
  }));
  
  return [home, blog, ...postUrls];
}



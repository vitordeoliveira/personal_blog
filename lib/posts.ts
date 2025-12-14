import fs from "fs";
import path from "path";
import matter from "gray-matter";
import { remark } from "remark";
import html from "remark-html";
import { getPostMetadata, incrementViews, getAllPostMetadata } from "./db";

const postsDirectory = path.join(process.cwd(), "posts");

export interface PostMetadata {
  title: string;
  subtitle?: string;
  description?: string;
  date: string;
  tags?: string[];
  ready?: boolean;
}

export interface Post extends PostMetadata {
  slug: string;
  content: string;
  contentHtml: string;
  views: number;
}

export function getPostSlugs(): string[] {
  if (!fs.existsSync(postsDirectory)) {
    return [];
  }
  return fs
    .readdirSync(postsDirectory)
    .filter((file) => file.endsWith(".md"))
    .map((file) => file.replace(/\.md$/, ""));
}

export async function getPostBySlug(
  slug: string,
  trackView: boolean = false
): Promise<Post | null> {
  const fullPath = path.join(postsDirectory, `${slug}.md`);
  
  if (!fs.existsSync(fullPath)) {
    return null;
  }

  const fileContents = fs.readFileSync(fullPath, "utf8");
  const { data, content } = matter(fileContents);

  const processedContent = await remark().use(html).process(content);
  const contentHtml = processedContent.toString();

  // Get or initialize metadata
  const metadata = getPostMetadata(slug);
  const views = metadata?.views || 0;

  // If ready is undefined, treat it as false (not published)
  const ready = data.ready !== undefined ? data.ready : false;

  // Don't return unpublished posts
  if (ready !== true) {
    return null;
  }

  // Track view if requested
  if (trackView) {
    incrementViews(slug);
  }

  return {
    slug,
    title: data.title || "",
    subtitle: data.subtitle,
    description: data.description,
    date: data.date || "",
    tags: data.tags || [],
    ready,
    content,
    contentHtml,
    views: trackView ? views + 1 : views,
  };
}

export async function getAllPosts(): Promise<Post[]> {
  const slugs = getPostSlugs();
  const allMetadata = getAllPostMetadata();
  
  const posts = await Promise.all(
    slugs.map(async (slug) => {
      const post = await getPostBySlug(slug);
      if (!post) return null;
      
      // Add views from metadata
      const metadata = allMetadata[slug];
      return {
        ...post,
        views: metadata?.views || 0,
      };
    })
  );

  return posts
    .filter((post): post is Post => post !== null)
    .filter((post) => post.ready === true) // Only show published posts
    .sort((a, b) => {
      if (a.date < b.date) {
        return 1;
      } else {
        return -1;
      }
    });
}


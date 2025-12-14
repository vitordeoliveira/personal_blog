import Link from "next/link";
import { notFound } from "next/navigation";
import { getAllPosts, getPostBySlug } from "@/lib/posts";
import PostViewTracker from "./post-view-tracker";

export async function generateStaticParams() {
  const posts = await getAllPosts();
  return posts.map((post) => ({
    slug: post.slug,
  }));
}

export default async function PostPage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const post = await getPostBySlug(slug);

  if (!post) {
    notFound();
  }

  return (
    <div className="relative min-h-screen bg-bg">
      {/* Background decorative elements */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute -top-40 -right-40 h-96 w-96 rounded-full bg-action-primary/12 blur-3xl"></div>
        <div className="absolute -bottom-40 -left-40 h-96 w-96 rounded-full bg-action-primary/10 blur-3xl"></div>
        <div className="absolute top-1/3 right-1/4 h-72 w-72 rounded-full bg-action-primary/8 blur-3xl"></div>
        <div className="absolute bottom-1/4 left-1/3 h-64 w-64 rounded-full bg-action-primary/6 blur-3xl"></div>
      </div>
      
      {/* Grid pattern overlay */}
      <div className="bg-pattern"></div>

      <main className="relative mx-auto max-w-4xl px-6 py-16">
        <Link
          href="/blog"
          className="mb-8 inline-block text-action-primary hover:underline"
        >
          ← Back to Blog
        </Link>

        <article className="prose prose-lg max-w-none rounded-lg bg-bg-light px-12 py-8 shadow-sm">
          <header className="mb-8">
            <h1 className="mb-2 text-4xl font-bold text-text">{post.title}</h1>
            {post.subtitle && (
              <p className="mb-4 text-xl font-medium text-text-muted">{post.subtitle}</p>
            )}
            {post.description && (
              <p className="mb-4 text-lg text-text-muted">{post.description}</p>
            )}
            <div className="mb-4 flex flex-wrap items-center gap-4 text-sm text-text-muted">
              <time dateTime={post.date}>
                {new Date(post.date).toLocaleDateString("en-US", {
                  year: "numeric",
                  month: "long",
                  day: "numeric",
                })}
              </time>
              <span>•</span>
              <PostViewTracker slug={slug} initialViews={post.views} />
            </div>
            {post.tags && post.tags.length > 0 && (
              <div className="flex flex-wrap gap-2">
                {post.tags.map((tag) => (
                  <span
                    key={tag}
                    className="rounded-full bg-action-primary/10 px-3 py-1 text-xs font-medium text-action-primary"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            )}
          </header>

          <div
            className="prose prose-lg max-w-none text-text"
            dangerouslySetInnerHTML={{ __html: post.contentHtml }}
            style={{
              color: "var(--text)",
            }}
          />
        </article>
      </main>
    </div>
  );
}


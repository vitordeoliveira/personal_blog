import Link from "next/link";
import { getAllPosts } from "@/lib/posts";

// Force dynamic rendering to ensure views are always up-to-date
export const dynamic = 'force-dynamic';

export default async function BlogPage() {
  const posts = await getAllPosts();

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
        <h1 className="mb-12 text-4xl font-bold text-text">Blog</h1>
        
        {posts.length === 0 ? (
          <p className="text-text-muted">No posts yet. Check back soon!</p>
        ) : (
          <div className="space-y-8">
            {posts.map((post) => (
              <article
                key={post.slug}
                className="rounded-lg border border-border bg-bg-light p-6 transition-colors hover:border-action-primary/30"
              >
                <Link href={`/blog/${post.slug}`}>
                  <h2 className="mb-1 text-2xl font-semibold text-text hover:text-action-primary transition-colors">
                    {post.title}
                  </h2>
                </Link>
                {post.subtitle && (
                  <p className="mb-3 text-lg font-medium text-text-muted">{post.subtitle}</p>
                )}
                <div className="flex items-center gap-3 text-sm text-text-muted">
                  <time dateTime={post.date}>
                    {new Date(post.date).toLocaleDateString("en-US", {
                      year: "numeric",
                      month: "long",
                      day: "numeric",
                    })}
                  </time>
                  <span>•</span>
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
                    {post.views.toLocaleString()} {post.views === 1 ? "view" : "views"}
                  </span>
                </div>
                {post.tags && post.tags.length > 0 && (
                  <div className="mt-3 flex flex-wrap gap-2">
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
                {post.description && (
                  <p className="mt-3 text-text-muted">{post.description}</p>
                )}
                <Link
                  href={`/blog/${post.slug}`}
                  className="mt-4 inline-block text-action-primary hover:underline"
                >
                  Read more →
                </Link>
              </article>
            ))}
          </div>
        )}
      </main>
    </div>
  );
}


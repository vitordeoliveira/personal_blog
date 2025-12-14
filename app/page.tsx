import Link from "next/link";
import { getAllPosts } from "@/lib/posts";

// Force dynamic rendering to ensure views are always up-to-date
export const dynamic = 'force-dynamic';

export default async function Home() {
  const allPosts = await getAllPosts();
  const recentPosts = allPosts.slice(0, 5);

  return (
    <div className="relative min-h-screen bg-bg">
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

      <main className="relative mx-auto max-w-5xl px-6 py-12">
        <section className="mb-20 overflow-hidden rounded-2xl border border-border bg-bg-light shadow-lg">
          <div className="relative p-10 md:p-12">
            <div className="mb-8">
              <h1 className="mb-4 text-5xl font-bold tracking-tight text-text md:text-6xl">
                Hi, I'm{" "}
                <span className="text-action-primary">Vitor de Oliveira</span>
              </h1>
              <div className="mt-6 flex flex-wrap gap-3">
                <span className="rounded-full bg-action-primary/10 px-4 py-1.5 text-sm font-semibold text-action-primary">
                  10+ Years Experience
                </span>
                <span className="rounded-full bg-action-primary/10 px-4 py-1.5 text-sm font-semibold text-action-primary">
                  JavaScript • Rust • AI
                </span>
                <span className="rounded-full bg-action-primary/10 px-4 py-1.5 text-sm font-semibold text-action-primary">
                  Founder & Partner
                </span>
              </div>
            </div>

            <div className="space-y-6 text-lg leading-relaxed text-text-muted">
              <p>
                With over <span className="font-semibold text-text">10+ years of experience</span> in the tech industry, 
                I've had the privilege of working with leading companies like{" "}
                <span className="font-semibold text-text">Volkswagen</span>, 
                where I've contributed to innovative projects and cutting-edge solutions.
              </p>
              <p>
                I specialize in <span className="font-semibold text-action-primary">JavaScript</span>,{" "}
                <span className="font-semibold text-action-primary">Rust</span>, and{" "}
                <span className="font-semibold text-action-primary">AI</span>, combining these technologies 
                to build robust, performant, and intelligent applications.
              </p>
              <p>
                Beyond my technical expertise, I'm also an entrepreneur—<span className="font-semibold text-text">founder of 3 companies</span> and{" "}
                <span className="font-semibold text-text">partner in another</span>. This entrepreneurial experience has given me 
                a unique perspective on building products, leading teams, and solving complex business challenges.
              </p>
              <p className="text-base">
                Through this blog, I share insights, experiences, and knowledge from my journey in software development, 
                AI, entrepreneurship, and modern web technologies.
              </p>
            </div>

            <div className="mt-12 flex justify-center">
              <Link
                href="/blog"
                className="group relative rounded-full bg-action-primary px-10 py-4 text-lg font-semibold text-bg-light shadow-lg transition-all hover:scale-105 hover:shadow-xl hover:opacity-95"
              >
                <span className="relative z-10">View All Blog Posts</span>
                <span className="ml-2 inline-block transition-transform group-hover:translate-x-1">
                  →
                </span>
              </Link>
            </div>
          </div>
        </section>

        {recentPosts.length > 0 && (
          <section className="mt-16">
            <h2 className="mb-8 text-3xl font-bold text-text">Recent Posts</h2>
            <div className="space-y-6">
              {recentPosts.map((post) => (
                <article
                  key={post.slug}
                  className="rounded-lg border border-border bg-bg-light p-6 transition-colors hover:border-action-primary/30"
                >
                  <Link href={`/blog/${post.slug}`}>
                    <h3 className="mb-1 text-2xl font-semibold text-text hover:text-action-primary transition-colors">
                      {post.title}
                    </h3>
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
            {allPosts.length > 5 && (
              <div className="mt-8 text-center">
                <Link
                  href="/blog"
                  className="text-action-primary hover:underline"
                >
                  View all {allPosts.length} posts →
                </Link>
              </div>
            )}
          </section>
        )}
      </main>
    </div>
  );
}

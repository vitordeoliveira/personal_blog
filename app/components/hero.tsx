import Link from "next/link";
import ChatButton from "./chat-button";

export default function Hero() {
  return (
    <div className="relative mx-auto max-w-5xl px-6 py-12">
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

            <div className="mt-12 flex flex-wrap justify-center gap-4">
              <ChatButton />
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
    </div>
  );
}

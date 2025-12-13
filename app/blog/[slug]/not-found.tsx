import Link from "next/link";

export default function NotFound() {
  return (
    <div className="min-h-screen bg-bg flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-bold text-text mb-4">Post Not Found</h1>
        <p className="text-text-muted mb-8">
          The blog post you're looking for doesn't exist.
        </p>
        <Link
          href="/blog"
          className="text-action-primary hover:underline"
        >
          ← Back to Blog
        </Link>
      </div>
    </div>
  );
}


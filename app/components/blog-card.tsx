import Link from "next/link";
import { motion } from "framer-motion";
import { Eye } from "lucide-react";
import { Post } from "@/lib/posts";

interface BlogCardProps {
  post: Post;
  index: number;
}

export function BlogCard({ post, index }: BlogCardProps) {
  return (
    <motion.article
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.1 }}
    >
      <Link
        href={`/blog/${post.slug}`}
        className="group block p-6 -mx-6 rounded-xl transition-all duration-300 hover:bg-bg-dark hover:shadow-md"
      >
        <div className="flex flex-col gap-3">
          {/* Meta info */}
          <div className="flex items-center gap-3 text-sm text-text-muted">
            <time>
              {new Date(post.date).toLocaleDateString("en-US", {
                year: "numeric",
                month: "long",
                day: "numeric",
              })}
            </time>
            <span className="text-text-muted">•</span>
            <span className="flex items-center gap-1">
              <Eye className="h-3.5 w-3.5" />
              {post.views} views
            </span>
          </div>

          {/* Title */}
          <h2 className="text-xl md:text-2xl font-bold group-hover:text-action-primary transition-colors">
            {post.title}
          </h2>

          {/* Description */}
          <p className="text-text-muted leading-relaxed">
            {post.description}
          </p>

          {/* Tags */}
          <div className="flex flex-wrap gap-2 mt-2">
            {post.tags && post.tags.map((tag) => (
              <span
                key={tag}
                className="px-3 py-1 text-xs font-medium rounded-full bg-action-secondary text-text"
              >
                {tag}
              </span>
            ))}
          </div>
        </div>
      </Link>
    </motion.article>
  );
}

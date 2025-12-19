"use client";

import { useState, useMemo } from "react";
import Link from "next/link";
import { Post } from "@/lib/posts";

type SortField = "date" | "status" | "views" | null;
type SortDirection = "asc" | "desc";

interface PostsTableProps {
  posts: Post[];
}

export default function PostsTable({ posts }: PostsTableProps) {
  const [sortField, setSortField] = useState<SortField>(null);
  const [sortDirection, setSortDirection] = useState<SortDirection>("asc");

  const sortedPosts = useMemo(() => {
    if (!sortField) {
      return posts;
    }

    const sorted = [...posts].sort((a, b) => {
      let comparison = 0;

      switch (sortField) {
        case "date":
          const dateA = a.date ? new Date(a.date).getTime() : 0;
          const dateB = b.date ? new Date(b.date).getTime() : 0;
          comparison = dateA - dateB;
          break;
        case "status":
          // Published (true) comes before Draft (false)
          comparison = (a.ready ? 1 : 0) - (b.ready ? 1 : 0);
          break;
        case "views":
          comparison = a.views - b.views;
          break;
      }

      return sortDirection === "asc" ? comparison : -comparison;
    });

    return sorted;
  }, [posts, sortField, sortDirection]);

  function handleSort(field: SortField) {
    if (sortField === field) {
      // Toggle direction if clicking the same field
      setSortDirection(sortDirection === "asc" ? "desc" : "asc");
    } else {
      // Set new field and default to ascending
      setSortField(field);
      setSortDirection("asc");
    }
  }

  function SortIcon({ field }: { field: SortField }) {
    if (sortField !== field) {
      return (
        <span className="ml-1 text-text-muted opacity-50">
          <svg className="w-4 h-4 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
          </svg>
        </span>
      );
    }

    return (
      <span className="ml-1 text-action-primary">
        {sortDirection === "asc" ? (
          <svg className="w-4 h-4 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 15l7-7 7 7" />
          </svg>
        ) : (
          <svg className="w-4 h-4 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        )}
      </span>
    );
  }

  return (
    <div className="rounded-lg border border-border bg-bg-light overflow-hidden">
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead className="bg-bg-dark border-b border-border">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
                Title
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
                Slug
              </th>
              <th
                className="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider cursor-pointer hover:bg-bg/50 transition-colors select-none"
                onClick={() => handleSort("date")}
              >
                <span className="flex items-center">
                  Date
                  <SortIcon field="date" />
                </span>
              </th>
              <th
                className="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider cursor-pointer hover:bg-bg/50 transition-colors select-none"
                onClick={() => handleSort("status")}
              >
                <span className="flex items-center">
                  Status
                  <SortIcon field="status" />
                </span>
              </th>
              <th
                className="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider cursor-pointer hover:bg-bg/50 transition-colors select-none"
                onClick={() => handleSort("views")}
              >
                <span className="flex items-center">
                  Views
                  <SortIcon field="views" />
                </span>
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {sortedPosts.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-6 py-8 text-center text-text-muted">
                  No posts found
                </td>
              </tr>
            ) : (
              sortedPosts.map((post) => (
                <tr key={post.slug} className="hover:bg-bg-dark/50 transition-colors">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm font-medium text-text">{post.title}</div>
                    {post.subtitle && (
                      <div className="text-sm text-text-muted">{post.subtitle}</div>
                    )}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <code className="text-xs text-text-muted bg-bg-dark px-2 py-1 rounded">
                      {post.slug}
                    </code>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-text-muted">
                    {post.date ? new Date(post.date).toLocaleDateString() : "—"}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span
                      className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${
                        post.ready
                          ? "bg-alert-success/10 text-alert-success"
                          : "bg-alert-warning/10 text-alert-warning"
                      }`}
                    >
                      {post.ready ? "Published" : "Draft"}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-text">
                    {post.views.toLocaleString()}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm">
                    <Link
                      href={`/admin/posts/${post.slug}`}
                      className="text-action-primary hover:underline"
                    >
                      Edit Views
                    </Link>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}


